use egui::emath::Numeric;
use egui::{Color32, TextBuffer, Ui};
use egui_snarl::ui::{PinInfo, WireStyle};
use reactor_types::Vector;

use super::{NUMBER_COLOR, STRING_COLOR, UNTYPED_COLOR, VECTOR_COLOR, horizontal};
use crate::node::viewer::widget::color_picker::{Alpha, color_edit_button_srgba};

pub fn number_view<N>(ui: &mut Ui, label: &str, value: &mut N) -> PinInfo
where
    N: Numeric,
{
    horizontal(ui, label, |ui| {
        ui.add_enabled(true, egui::DragValue::new(value));
    });
    PinInfo::circle().with_fill(NUMBER_COLOR)
}

pub fn string_view(ui: &mut Ui, label: &str, value: &mut dyn TextBuffer) -> PinInfo {
    horizontal(ui, label, |ui| {
        let edit = egui::TextEdit::singleline(value)
            .clip_text(false)
            .desired_width(0.0)
            .margin(ui.spacing().item_spacing);
        ui.add(edit);
    });

    PinInfo::circle()
        .with_fill(STRING_COLOR)
        .with_wire_style(WireStyle::AxisAligned { corner_radius: 10.0 })
}

pub fn vector_view(ui: &mut Ui, label: &str, vector: &mut Vector) -> PinInfo {
    horizontal(ui, label, |ui| {
        ui.add(egui::DragValue::new(&mut vector[0]));
        ui.add(egui::DragValue::new(&mut vector[1]));

        if vector.len() > 2 {
            ui.add(egui::DragValue::new(&mut vector[2]));
            if vector.len() > 3 {
                ui.add(egui::DragValue::new(&mut vector[3]));
            }
        }
    });
    PinInfo::circle().with_fill(VECTOR_COLOR)
}

pub fn color_view(ui: &mut Ui, label: &str, value: &mut Color32) -> PinInfo {
    horizontal(ui, label, |ui| {
        color_edit_button_srgba(ui, value, Alpha::BlendOrAdditive);
    });
    PinInfo::circle().with_fill(VECTOR_COLOR)
}

pub fn empty_view() -> PinInfo {
    PinInfo::circle().with_fill(UNTYPED_COLOR)
}
