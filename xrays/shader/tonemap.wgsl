#define_import_path tonemap

fn uncharted2(x: vec3<f32>) -> vec3<f32> {
    // Based on uncharted2 tonemapping function
    // https://dmnsgn.github.io/glsl-tone-map/
    let exposure_bias = 0.246;   // determined experimentally for the scene
    let curr = uncharted2_tonemap(exposure_bias * x);

    let w = 11.2;
    let white_scale = 1f / uncharted2_tonemap(vec3(w));
    return white_scale * curr;
}

fn uncharted2_tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 0.15;
    let b = 0.50;
    let c = 0.10;
    let d = 0.20;
    let e = 0.02;
    let f = 0.30;
    let w = 11.2;
    return ((x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f)) - e / f;
}
