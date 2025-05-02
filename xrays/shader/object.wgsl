#define_import_path object

#import consts::{PI, FRAC_1_PI, MIN_T, MAX_T}
#import types::Ray

@group(3) @binding(0) var<storage, read> spheres: array<Sphere>;

struct Sphere {
    center_and_pad: vec4<f32>,
    radius: f32,
    material_idx: u32,
}

struct Intersection {
    point: vec3<f32>,
    normal: vec3<f32>,
    u: f32,
    v: f32,
    t: f32,
    material_idx: u32,
    sphere_idx: u32,
}

fn intersection(ray: Ray, intersection: ptr<function, Intersection>) -> bool {
    var closest_t = MAX_T;
    var closest_intersection = Intersection();

    for (var idx = 0u; idx < arrayLength(&spheres); idx = idx + 1u) {
        var test_intersect = Intersection();
        if ray_intersect_sphere(ray, idx, MIN_T, closest_t, &test_intersect) {
            closest_t = test_intersect.t;
            closest_intersection = test_intersect;
        }
    }

    if closest_t < MAX_T {
        *intersection = closest_intersection;
        return true;
    }

    return false;
}

fn ray_intersect_sphere(ray: Ray, sphere_idx: u32, tmin: f32, tmax: f32, hit: ptr<function, Intersection>) -> bool {
    let sphere = spheres[sphere_idx];
    let oc = ray.origin - sphere.center_and_pad.xyz;
    let a = dot(ray.direction, ray.direction);
    let b = dot(oc, ray.direction);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = b * b - a * c;

    if discriminant > 0f {
        var t = (-b - sqrt(b * b - a * c)) / a;
        if t < tmax && t > tmin {
            *hit = sphere_intersection(ray, sphere, sphere_idx, t);
            return true;
        }

        t = (-b + sqrt(b * b - a * c)) / a;
        if t < tmax && t > tmin {
            *hit = sphere_intersection(ray, sphere, sphere_idx, t);
            return true;
        }
    }

    return false;
}

fn sphere_intersection(ray: Ray, sphere: Sphere, sphere_idx: u32, t: f32) -> Intersection {
    let p = ray_point_at_parameter(ray, t);
    let n = (1f / sphere.radius) * (p - sphere.center_and_pad.xyz);
    let theta = acos(-n.y);
    let phi = atan2(-n.z, n.x) + PI;
    let u = 0.5 * FRAC_1_PI * phi;
    let v = FRAC_1_PI * theta;

    // TODO: passing sphere_idx in here just to pass it to Intersection
    return Intersection(p, n, u, v, t, sphere.material_idx, sphere_idx);
}

fn ray_point_at_parameter(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + t * ray.direction;
}
