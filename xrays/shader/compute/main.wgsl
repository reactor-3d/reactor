#import consts::{EPSILON, PI, FRAC_1_PI, CHANNEL_R, CHANNEL_G, CHANNEL_B}
#import object::{intersection, Intersection, Sphere, spheres}
#import rng
#import sampling::SamplingParams
#import types::Ray

@group(1) @binding(0) var<uniform> frame_data: vec4<u32>;
@group(1) @binding(1) var<storage, read_write> image_buffer: array<array<f32, 3>>;

@group(2) @binding(0) var<uniform> sampling_params: SamplingParams;
@group(2) @binding(1) var<uniform> camera: Camera;
@group(2) @binding(2) var<storage, read> sky_state: SkyState;

// @group(3) @binding(0) var<storage, read> spheres: array<Sphere>;
@group(3) @binding(1) var<storage, read> materials: array<Material>;
@group(3) @binding(2) var<storage, read> textures: array<array<f32, 3>>;
@group(3) @binding(3) var<storage, read> lights: array<u32>;

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let image_width = frame_data.x;
    let image_height = frame_data.y;
    let frame_number = frame_data.z;

    let x = global_id.x;
    let y = global_id.y;

    if (x >= image_width || y >= image_height) {
        return;
    }
    let idx = image_width * y + x;

    var rng_state = rng::init(vec2(x, y), vec2(image_width, image_height), frame_number);
    var pixel = vec3(image_buffer[idx][0], image_buffer[idx][1], image_buffer[idx][2]);
    {
        if sampling_params.clear_accumulated_samples == 1 {
            pixel = vec3(0f);
        }

        let rgb = sample_pixel(x, y, &rng_state);
        pixel += rgb;
    }

    image_buffer[idx] = array<f32, 3>(pixel.r, pixel.g, pixel.b);
}

fn sample_pixel(x: u32, y: u32, rng_state: ptr<function, u32>) -> vec3<f32> {
    let image_width = frame_data.x;
    let image_height = frame_data.y;
    let inv_width = 1f / f32(image_width);
    let inv_height = 1f / f32(image_height);

    let num_samples = sampling_params.num_samples_per_pixel;
    var pixel_color = vec3(0f);
    for (var i = 0u; i < num_samples; i += 1u) {
        let u = (f32(x) + rng::next_float(rng_state)) * inv_width;
        let v = (f32(y) + rng::next_float(rng_state)) * inv_height;

        let primary_ray = camera_make_ray(camera, rng_state, u, 1f - v);
        pixel_color += ray_color(primary_ray, rng_state);
    }

    return pixel_color;
}

fn ray_color(primary_ray: Ray, rng_state: ptr<function, u32>) -> vec3<f32> {
    var ray = primary_ray;

    var color = vec3(0f);
    var throughput = vec3(1f);

    for (var bounce = 0u; bounce < sampling_params.num_bounces; bounce += 1u) {
        var intersection = Intersection();

        if intersection(ray, &intersection) {
            let material = materials[intersection.material_idx];

            if material.id == 4u {
                let emission_texture = material.desc1;
                let emission_color = texture_lookup(emission_texture, intersection.u, intersection.v);
                color += throughput * emission_color;
                break;
            }

            var scatter = scatter_ray(ray, intersection, material, rng_state);
            ray = scatter.ray;
            throughput *= scatter.throughput;
        } else {
            // The ray missed. Output background color.
            let v = normalize(ray.direction);
            let s = sky_state.sun_direction;

            let theta = acos(v.y);
            let gamma = acos(clamp(dot(v, s), -1f, 1f));

            color += throughput * vec3(
                radiance(theta, gamma, CHANNEL_R),
                radiance(theta, gamma, CHANNEL_G),
                radiance(theta, gamma, CHANNEL_B)
            );

            break;
        }
    }

    return color;
}

fn scatter_ray(wo: Ray, hit: Intersection, material: Material, rng_state: ptr<function, u32>) -> Scatter {
    switch material.id {
        case 0u: {
            let texture = material.desc1;
            return scatter_mixture_density(hit, texture, rng_state);
        }

        case 1u: {
            let texture = material.desc1;
            let fuzz = material.x;
            return scatter_metal(wo, hit, texture, fuzz, rng_state);
        }

        case 2u: {
            let refraction_index = material.x;
            return scatter_dielectric(wo, hit, refraction_index, rng_state);
        }

        case 3u: {
            let texture1 = material.desc1;
            let texture2 = material.desc2;
            return scatter_checkerboard(hit, texture1, texture2, rng_state);
        }

        default: {
            return scatter_missing_material(hit, rng_state);
        }
    }
}

fn scatter_mixture_density(hit: Intersection, albedo: TextureDescriptor, rng_state: ptr<function, u32>) -> Scatter {
    let scatter_direction = sample_mixture_density(hit, rng_state);
    let material_value = eval_lambertian(hit, albedo, scatter_direction);
    let material_pdf = pdf_lambertian(hit, scatter_direction);
    let light_pdf = pdf_light(hit, scatter_direction);
    let throughput = material_value / max(EPSILON, (0.5f * material_pdf + 0.5f * light_pdf));
    return Scatter(Ray(hit.point, scatter_direction), throughput);
}

fn sample_mixture_density(hit: Intersection, rng_state: ptr<function, u32>) -> vec3<f32> {
    if rng::next_float(rng_state) < 0.5f {
        return sample_lambertian(hit, rng_state);
    } else {
        return sample_light(hit, rng_state);
    }
}

fn eval_lambertian(hit: Intersection, texture: TextureDescriptor, wi: vec3<f32>) -> vec3<f32> {
    return texture_lookup(texture, hit.u, hit.v) * FRAC_1_PI * max(EPSILON, dot(hit.normal, wi));
}

fn sample_lambertian(hit: Intersection, rng_state: ptr<function, u32>) -> vec3<f32> {
    let v = rng::next_in_cosine_weighted_hemisphere(rng_state);
    let onb = pixar_onb(hit.normal);
    return onb * v;
}

fn pdf_lambertian(hit: Intersection, wi: vec3<f32>) -> f32 {
    return max(EPSILON, dot(hit.normal, wi) * FRAC_1_PI);
}

fn sample_light(hit: Intersection, rng_state: ptr<function, u32>) -> vec3<f32> {
    // Select a random light using a uniform distribution.
    let num_lights = arrayLength(&lights);   // TODO: what about when there are no lights?
    let light_idx = rng::next_uint_in_range(rng_state, 0u, num_lights - 1u);
    let sphere_idx = lights[light_idx];
    let sphere = spheres[sphere_idx];

    return sample_hemisphere(hit, sphere, rng_state);
}

fn sample_hemisphere(hit: Intersection, sphere: Sphere, rng_state: ptr<function, u32>) -> vec3<f32> {
    let v = rng::next_in_unit_hemisphere(rng_state);

    // Sample the hemisphere facing the intersection point.
    let dir = normalize(hit.point - sphere.center_and_pad.xyz);
    let onb = pixar_onb(dir);

    let point_on_sphere = sphere.center_and_pad.xyz + onb * sphere.radius * v;
    let to_point_on_sphere = point_on_sphere - hit.point;

    return normalize(to_point_on_sphere);
}

fn pdf_light(hit: Intersection, wi: vec3<f32>) -> f32 {
    let ray = Ray(hit.point, wi);
    var light_hit = Intersection();
    var pdf = 0f;

    if intersection(ray, &light_hit) {
        let sphere_idx = light_hit.sphere_idx;
        let sphere = spheres[sphere_idx];
        let num_spheres = arrayLength(&spheres);
        let to_light = light_hit.point - hit.point;
        let length_sqr = dot(to_light, to_light);
        let cosine = abs(dot(wi, light_hit.normal));
        let area_half_sphere = 2f * PI * sphere.radius * sphere.radius;

        // length_sqr / cosine is the inverse of the geometric factor, as defined in
        // "MULTIPLE IMPORTANCE SAMPLING 101".
        pdf = length_sqr / max(EPSILON, cosine * area_half_sphere * f32(num_spheres));
    }

    return pdf;
}

fn pixar_onb(n: vec3<f32>) -> mat3x3<f32> {
    // https://www.jcgt.org/published/0006/01/01/paper-lowres.pdf
    let s = select(-1f, 1f, n.z >= 0f);
    let a = -1f / (s + n.z);
    let b = n.x * n.y * a;
    let u = vec3<f32>(1f + s * n.x * n.x * a, s * b, -s * n.x);
    let v = vec3<f32>(b, s + n.y * n.y * a, -n.y);

    return mat3x3<f32>(u, v, n);
}

fn scatter_metal(wo: Ray, hit: Intersection, texture: TextureDescriptor, fuzz: f32, rng_state: ptr<function, u32>) -> Scatter {
    let scatter_direction = reflect(wo.direction, hit.normal) + fuzz * rng::next_vec3_in_unit_sphere(rng_state);
    let albedo = texture_lookup(texture, hit.u, hit.v);
    return Scatter(Ray(hit.point, scatter_direction), albedo);
}

fn scatter_dielectric(rayIn: Ray, hit: Intersection, refraction_index: f32, rng_state: ptr<function, u32>) -> Scatter {
    let wo = rayIn.direction;
    var outward_normal = vec3(0f);
    var ni_over_nt = 0f;
    var cosine = 0f;
    if dot(wo, hit.normal) > 0f {
        outward_normal = -hit.normal;
        ni_over_nt = refraction_index;
        cosine = refraction_index * dot(normalize(wo), hit.normal);
    } else {
        outward_normal = hit.normal;
        ni_over_nt = 1f / refraction_index;
        cosine = dot(normalize(-wo), hit.normal);
    };

    var refracted_direction = vec3(0f);
    if refract(wo, outward_normal, ni_over_nt, &refracted_direction) {
        let reflection_prob = schlick(cosine, refraction_index);
        var wi = refracted_direction;
        if rng::next_float(rng_state) < reflection_prob {
            reflect(wo, hit.normal);
        }

        return Scatter(Ray(hit.point, wi), vec3(1f));
    }

    let wi = reflect(wo, hit.normal);
    return Scatter(Ray(hit.point, wi), vec3(1f));
}

fn refract(v: vec3<f32>, n: vec3<f32>, ni_over_nt: f32, refract_direction: ptr<function, vec3<f32>>) -> bool {
    // ni * sin(i) = nt * sin(t)
    // sin(t) = sin(i) * (ni / nt)
    let uv = normalize(v);
    let dt = dot(uv, n);
    let discriminant = 1f - ni_over_nt * ni_over_nt * (1f - dt * dt);
    if discriminant > 0f {
        *refract_direction = normalize(ni_over_nt * (uv - dt * n) - sqrt(discriminant) * n);
        return true;
    }

    return false;
}

fn schlick(cosine: f32, refraction_index: f32) -> f32 {
    var r0 = (1f - refraction_index) / (1f + refraction_index);
    r0 = r0 * r0;
    return r0 + pow((1f - r0) * (1f - cosine), 5f);
}

fn scatter_checkerboard(hit: Intersection, texture1: TextureDescriptor, texture2: TextureDescriptor, rng_state: ptr<function, u32>) -> Scatter {
    let sines = sin(5f * hit.point.x) * sin(5f * hit.point.y) * sin(5f * hit.point.z);
    if sines < 0f {
        return scatter_mixture_density(hit, texture1, rng_state);
    } else {
        return scatter_mixture_density(hit, texture2, rng_state);
    }
}

fn scatter_missing_material(hit: Intersection, rng_state: ptr<function, u32>) -> Scatter {
    let scatter_direction = hit.normal + rng::next_vec3_in_unit_sphere(rng_state);
    // An aggressive pink color to indicate an error
    let albedo = vec3(0.9921f, 0.24705f, 0.57254f);
    return Scatter(Ray(hit.point, scatter_direction), albedo);
}

fn radiance(theta: f32, gamma: f32, channel: u32) -> f32 {
    let r = sky_state.radiances[channel];
    let idx = 9u * channel;
    let p0 = sky_state.params[idx + 0u];
    let p1 = sky_state.params[idx + 1u];
    let p2 = sky_state.params[idx + 2u];
    let p3 = sky_state.params[idx + 3u];
    let p4 = sky_state.params[idx + 4u];
    let p5 = sky_state.params[idx + 5u];
    let p6 = sky_state.params[idx + 6u];
    let p7 = sky_state.params[idx + 7u];
    let p8 = sky_state.params[idx + 8u];

    let cos_gamma = cos(gamma);
    let cos_gamma2 = cos_gamma * cos_gamma;
    let cos_theta = abs(cos(theta));

    let exp_m = exp(p4 * gamma);
    let ray_m = cos_gamma2;
    let mie_m_lhs = 1.0 + cos_gamma2;
    let mie_m_rhs = pow(1.0 + p8 * p8 - 2.0 * p8 * cos_gamma, 1.5f);
    let mie_m = mie_m_lhs / mie_m_rhs;
    let zenith = sqrt(cos_theta);
    let radiance_lhs = 1.0 + p0 * exp(p1 / (cos_theta + 0.01));
    let radiance_rhs = p2 + p3 * exp_m + p5 * ray_m + p6 * mie_m + p7 * zenith;
    let radiance_dist = radiance_lhs * radiance_rhs;
    return r * radiance_dist;
}

struct SkyState {
    params: array<f32, 27>,
    radiances: array<f32, 3>,
    sun_direction: vec3<f32>,
};

struct Material {
    id: u32,
    desc1: TextureDescriptor,
    desc2: TextureDescriptor,
    x: f32,
}

struct TextureDescriptor {
    width: u32,
    height: u32,
    offset: u32,
}

fn texture_lookup(desc: TextureDescriptor, arg_u: f32, arg_v: f32) -> vec3<f32> {
    let u = clamp(arg_u, 0f, 1f);
    let v = 1f - clamp(arg_v, 0f, 1f);

    let j = u32(u * f32(desc.width));
    let i = u32(v * f32(desc.height));
    let idx = i * desc.width + j;

    let elem = textures[desc.offset + idx];
    return vec3(elem[0u], elem[1u], elem[2u]);
}

struct Scatter {
    ray: Ray,
    throughput: vec3<f32>,
}

struct Camera {
    eye: vec3<f32>,
    horizontal: vec3<f32>,
    vertical: vec3<f32>,
    u: vec3<f32>,
    v: vec3<f32>,
    lens_radius: f32,
    lower_left_corner: vec3<f32>,
}

fn camera_make_ray(camera: Camera, rng_state: ptr<function, u32>, u: f32, v: f32) -> Ray {
    let random_point_in_lens = camera.lens_radius * rng::next_vec3_in_unit_disk(rng_state);
    let lens_offset = random_point_in_lens.x * camera.u + random_point_in_lens.y * camera.v;

    let origin = camera.eye + lens_offset;
    let direction = camera.lower_left_corner + u * camera.horizontal + v * camera.vertical - origin;

    return Ray(origin, direction);
}
