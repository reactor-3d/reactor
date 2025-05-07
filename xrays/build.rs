use std::path::Path;
use std::{env, fs};

use naga::Module;
use naga::back::wgsl::WriterFlags;
use naga::valid::{ValidationFlags, Validator};
use naga_oil::compose::{ComposableModuleDescriptor, Composer, NagaModuleDescriptor};

fn main() {
    let mut composer = Composer::default();

    composer
        .add_composable_module(ComposableModuleDescriptor {
            source: include_str!("shader/sampling.wgsl"),
            file_path: "shader/sampling.wgsl",
            additional_imports: Default::default(),
            ..Default::default()
        })
        .expect("Failed to add shader");
    composer
        .add_composable_module(ComposableModuleDescriptor {
            source: include_str!("shader/compute/consts.wgsl"),
            file_path: "shader/compute/consts.wgsl",
            additional_imports: Default::default(),
            ..Default::default()
        })
        .expect("Failed to add shader");
    composer
        .add_composable_module(ComposableModuleDescriptor {
            source: include_str!("shader/compute/types.wgsl"),
            file_path: "shader/compute/types.wgsl",
            additional_imports: Default::default(),
            ..Default::default()
        })
        .expect("Failed to add shader");
    composer
        .add_composable_module(ComposableModuleDescriptor {
            source: include_str!("shader/compute/object.wgsl"),
            file_path: "shader/compute/object.wgsl",
            additional_imports: Default::default(),
            ..Default::default()
        })
        .expect("Failed to add shader");
    composer
        .add_composable_module(ComposableModuleDescriptor {
            source: include_str!("shader/compute/rng.wgsl"),
            file_path: "shader/compute/rng.wgsl",
            additional_imports: Default::default(),
            ..Default::default()
        })
        .expect("Failed to add shader");
    composer
        .add_composable_module(ComposableModuleDescriptor {
            source: include_str!("shader/render/tonemap.wgsl"),
            file_path: "shader/render/tonemap.wgsl",
            additional_imports: Default::default(),
            ..Default::default()
        })
        .expect("Failed to add shader");

    let compute_shader_module = composer
        .make_naga_module(NagaModuleDescriptor {
            source: include_str!("shader/compute/main.wgsl"),
            file_path: "shader/compute/main.wgsl",
            ..Default::default()
        })
        .expect("Failed to compose compute shader");

    let render_shader_module = composer
        .make_naga_module(NagaModuleDescriptor {
            source: include_str!("shader/render/main.wgsl"),
            file_path: "shader/render/main.wgsl",
            ..Default::default()
        })
        .expect("Failed to compose render shader");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR must be set");
    generate_shader_file(
        &mut composer,
        &compute_shader_module,
        Path::new(&out_dir).join("compute_shader.wgsl"),
    );
    generate_shader_file(
        &mut composer,
        &render_shader_module,
        Path::new(&out_dir).join("render_shader.wgsl"),
    );

    println!("cargo:rerun-if-changed=shader");
}

fn generate_shader_file(composer: &mut Composer, shader_module: &Module, output_path: impl AsRef<Path>) {
    let info = Validator::new(ValidationFlags::all(), composer.capabilities)
        .validate(shader_module)
        .expect("Failed to validate shader");
    let shader_wgsl = naga::back::wgsl::write_string(shader_module, &info, WriterFlags::EXPLICIT_TYPES).expect("");

    fs::write(output_path, shader_wgsl).expect("Failed to write combined shader");
}
