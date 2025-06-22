use std::error::Error;
use std::sync::Arc;

use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew};
use eframe::wgpu;

use crate::app::ReactorApp;
use crate::logger::LoggerConfig;

mod app;
mod logger;
mod node;
mod settings;
mod tabs;

fn main() -> Result<(), Box<dyn Error>> {
    let logger_config = LoggerConfig::load(None)?;
    logger::init(&logger_config)?;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([700.0, 520.0]),
        wgpu_options: WgpuConfiguration {
            wgpu_setup: WgpuSetup::CreateNew(WgpuSetupCreateNew {
                device_descriptor: Arc::new(|adapter| {
                    let mut base_limits = if adapter.get_info().backend == wgpu::Backend::Gl {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    };
                    base_limits.max_storage_buffer_binding_size = 512 << 20;

                    wgpu::DeviceDescriptor {
                        label: Some("egui wgpu device"),
                        required_features: wgpu::Features::default(),
                        required_limits: wgpu::Limits {
                            // When using a depth buffer, we have to be able to create a texture
                            // large enough for the entire surface, and we want to support 4k+ displays.
                            max_texture_dimension_2d: 8192,
                            ..base_limits
                        },
                        memory_hints: wgpu::MemoryHints::default(),
                    }
                }),
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    };

    eframe::run_native(
        "reactor",
        native_options,
        Box::new(|cx| Ok(Box::new(ReactorApp::new(cx)))),
    )?;

    Ok(())
}
