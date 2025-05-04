use egui::{Color32, Ui};

pub mod input;
pub mod output;

pub const NUMBER_COLOR: Color32 = Color32::from_rgb(0xb0, 0x00, 0x00);
pub const STRING_COLOR: Color32 = Color32::from_rgb(0x00, 0xb0, 0x00);
pub const VECTOR_COLOR: Color32 = Color32::from_rgb(0x00, 0x00, 0xb0);
pub const MATERIAL_COLOR: Color32 = Color32::from_rgb(0xb0, 0x00, 0xb0);
pub const UNTYPED_COLOR: Color32 = Color32::from_rgb(0xb0, 0xb0, 0xb0);

pub fn horizontal(ui: &mut Ui, label: &str, controls: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        if !label.is_empty() {
            ui.label(label);
        }
        controls(ui);
    });
}
