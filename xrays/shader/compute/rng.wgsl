#define_import_path rng
#import consts::PI

// Initialize RNG for given pixel, and frame number (Xorshift-based version)
fn init(pixel: vec2<u32>, resolution: vec2<u32>, frame: u32) -> u32 {
    // Adapted from https://github.com/boksajak/referencePT
    let seed = dot(pixel, vec2<u32>(1u, resolution.x)) ^ jenkins_hash(frame);
    return jenkins_hash(seed);
}

fn next_int(state: ptr<function, u32>) -> u32 {
    // PCG random number generator
    // Based on https://www.shadertoy.com/view/XlGcRh
    let new_state = *state * 747796405u + 2891336453u;
    *state = new_state;
    let word = ((new_state >> ((new_state >> 28u) + 4u)) ^ new_state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn next_float(state: ptr<function, u32>) -> f32 {
    let x = next_int(state);
    return f32(x) / f32(0xffffffffu);
}

fn next_uint_in_range(state: ptr<function, u32>, min: u32, max: u32) -> u32 {
    let x = next_int(state);
    return min + (x) % (max - min);
}

fn next_in_cosine_weighted_hemisphere(state: ptr<function, u32>) -> vec3<f32> {
    let r1 = next_float(state);
    let r2 = next_float(state);
    let sqrt_r2 = sqrt(r2);

    let z = sqrt(1f - r2);
    let phi = 2f * PI * r1;
    let x = cos(phi) * sqrt_r2;
    let y = sin(phi) * sqrt_r2;

    return vec3<f32>(x, y, z);
}

fn next_in_unit_hemisphere(state: ptr<function, u32>) -> vec3<f32> {
    let r1 = next_float(state);
    let r2 = next_float(state);

    let phi = 2f * PI * r1;
    let sin_theta = sqrt(1f - r2 * r2);

    let x = cos(phi) * sin_theta;
    let y = sin(phi) * sin_theta;
    let z = r2;

    return vec3(x, y, z);
}

fn next_vec3_in_unit_disk(state: ptr<function, u32>) -> vec3<f32> {
    // Generate numbers uniformly in a disk:
    // https://stats.stackexchange.com/a/481559

    // r^2 is distributed as U(0, 1).
    let r = sqrt(next_float(state));
    let alpha = 2f * PI * next_float(state);

    let x = r * cos(alpha);
    let y = r * sin(alpha);

    return vec3(x, y, 0f);
}

fn next_vec3_in_unit_sphere(state: ptr<function, u32>) -> vec3<f32> {
    let r = pow(next_float(state), 0.33333f);
    let cos_theta = 1f - 2f * next_float(state);
    let sin_theta = sqrt(1f - cos_theta * cos_theta);
    let phi = 2f * PI * next_float(state);

    let x = r * sin_theta * cos(phi);
    let y = r * sin_theta * sin(phi);
    let z = cos_theta;

    return vec3(x, y, z);
}

fn jenkins_hash(input: u32) -> u32 {
    var x = input;
    x += x << 10u;
    x ^= x >> 6u;
    x += x << 3u;
    x ^= x >> 11u;
    x += x << 15u;
    return x;
}
