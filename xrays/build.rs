use std::path::Path;
use std::{env, fs};

use naga::back::wgsl::WriterFlags;
use naga::valid::{ValidationFlags, Validator};
use naga_oil::compose::{ComposableModuleDescriptor, Composer, NagaModuleDescriptor};

fn main() {
    let mut composer = Composer::default();

    composer
        .add_composable_module(ComposableModuleDescriptor {
            source: include_str!("shader/consts.wgsl"),
            file_path: "shader/consts.wgsl",
            additional_imports: Default::default(),
            ..Default::default()
        })
        .expect("Failed to add shader");
    composer
        .add_composable_module(ComposableModuleDescriptor {
            source: include_str!("shader/types.wgsl"),
            file_path: "shader/types.wgsl",
            additional_imports: Default::default(),
            ..Default::default()
        })
        .expect("Failed to add shader");
    composer
        .add_composable_module(ComposableModuleDescriptor {
            source: include_str!("shader/object.wgsl"),
            file_path: "shader/object.wgsl",
            additional_imports: Default::default(),
            ..Default::default()
        })
        .expect("Failed to add shader");
    composer
        .add_composable_module(ComposableModuleDescriptor {
            source: include_str!("shader/rng.wgsl"),
            file_path: "shader/rng.wgsl",
            additional_imports: Default::default(),
            ..Default::default()
        })
        .expect("Failed to add shader");
    composer
        .add_composable_module(ComposableModuleDescriptor {
            source: include_str!("shader/tonemap.wgsl"),
            file_path: "shader/tonemap.wgsl",
            additional_imports: Default::default(),
            ..Default::default()
        })
        .expect("Failed to add shader");

    let combined_shader_module = composer
        .make_naga_module(NagaModuleDescriptor {
            source: include_str!("shader/main.wgsl"),
            file_path: "shader/main.wgsl",
            ..Default::default()
        })
        .expect("Failed to compose shader");

    let info = Validator::new(ValidationFlags::all(), composer.capabilities)
        .validate(&combined_shader_module)
        .expect("Failed to validate shader");

    let combined_shader_wgsl =
        naga::back::wgsl::write_string(&combined_shader_module, &info, WriterFlags::EXPLICIT_TYPES).expect("");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_shader_path = Path::new(&out_dir).join("shader_combined.wgsl");
    fs::write(&out_shader_path, combined_shader_wgsl).expect("Failed to write combined shader");

    println!("cargo:rerun-if-changed=shader");
}
