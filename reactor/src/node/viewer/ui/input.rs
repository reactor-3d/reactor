use egui::emath::Numeric;
use egui::epaint::Hsva;
use egui::{Ui, WidgetText};
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId};
use reactor_types::{Color, Float, NodePin, Vector};

use super::{MATERIAL_COLOR, NUMBER_COLOR, UNTYPED_COLOR, VECTOR_COLOR, horizontal};
use crate::node::item::InputMaterial;
use crate::node::message::SelfNodeMut;
use crate::node::subscribtion::Event;
use crate::node::viewer::remote;
use crate::node::viewer::widget::color_picker::{Alpha, color_button, color_edit_button_srgba};
use crate::node::{Node, Noded};

pub fn number_view<N>(ui: &mut Ui, label: &str, node_pin: &mut NodePin<N>, remote_value: Option<N>) -> PinInfo
where
    N: Numeric,
{
    horizontal(ui, label, |ui| {
        let enabled = match remote_value {
            None => true,
            Some(remote) => {
                node_pin.set(remote);
                false
            },
        };
        ui.add_enabled(enabled, egui::DragValue::new(node_pin.as_mut()));
    });
    PinInfo::circle().with_fill(NUMBER_COLOR)
}

pub fn as_number_view<N, M>(
    ui: &mut Ui,
    label: &str,
    node_pin: &mut NodePin<N>,
    remote_value: Option<(&'static str, M)>,
) -> PinInfo
where
    N: AsMut<f64>,
    M: Into<N>,
{
    horizontal(ui, label, |ui| {
        let enabled = match remote_value {
            None => true,
            Some(remote) => {
                node_pin.set(remote.1.into());
                false
            },
        };
        ui.add_enabled(enabled, egui::DragValue::new(node_pin.as_mut().as_mut()));
    });
    PinInfo::circle().with_fill(NUMBER_COLOR)
}

pub fn vector_view(ui: &mut Ui, label: &str, node_pin: &mut NodePin<Vector>, remote_value: Option<Vector>) -> PinInfo {
    horizontal(ui, label, |ui| {
        let enabled = match remote_value {
            None => true,
            Some(remote) => {
                node_pin.set(remote);
                false
            },
        };
        let vector = node_pin.as_mut();
        ui.add_enabled(enabled, egui::DragValue::new(&mut vector[0]));
        ui.add_enabled(enabled, egui::DragValue::new(&mut vector[1]));
        if vector.len() > 2 {
            ui.add_enabled(enabled, egui::DragValue::new(&mut vector[2]));
            if vector.len() > 3 {
                ui.add_enabled(enabled, egui::DragValue::new(&mut vector[3]));
            }
        }
    });
    PinInfo::circle().with_fill(VECTOR_COLOR)
}

pub fn color_view(ui: &mut Ui, label: &str, node_pin: &mut NodePin<Color>, remote_value: Option<Color>) -> PinInfo {
    horizontal(ui, label, |ui| match remote_value {
        None => {
            color_edit_button_srgba(ui, node_pin.as_mut(), Alpha::BlendOrAdditive);
        },
        Some(remote) => {
            node_pin.set(remote);
            color_button(ui, Hsva::from(node_pin.get()).into(), false);
        },
    });
    PinInfo::circle().with_fill(node_pin.get())
}

pub fn material_view(
    ui: &mut Ui,
    label: &str,
    node_pin: &mut NodePin<InputMaterial>,
    remote_value: Option<InputMaterial>,
) -> PinInfo {
    horizontal(ui, label, |_ui| {
        if let Some(value) = remote_value {
            node_pin.set(value);
        }
    });
    PinInfo::circle().with_fill(MATERIAL_COLOR)
}

pub fn empty_view(ui: &mut Ui, label: impl Into<WidgetText>) -> PinInfo {
    ui.label(label);
    PinInfo::circle().with_fill(UNTYPED_COLOR)
}

pub fn display_number_field(
    ui: &mut Ui,
    pin: &InPin,
    mut self_node: SelfNodeMut,
    label: &str,
    field_accessor: impl FnOnce(&mut Node) -> &mut NodePin<Float>,
) -> PinInfo {
    let remote_value = remote::number(pin, label, self_node.snarl);
    let node = self_node.node_mut();
    let field = field_accessor(node);

    let old_value = field.get();
    let info = number_view(ui, label, field, remote_value);

    if old_value != field.get() {
        if let Some(caller) = node
            .subscription_ref()
            .and_then(|subscription| subscription.event_caller(Event::OnChange))
        {
            caller(self_node)
        }
    }
    info
}

pub fn display_vector_field(
    ui: &mut Ui,
    pin: &InPin,
    mut self_node: SelfNodeMut,
    label: &str,
    field_accessor: impl FnOnce(&mut Node) -> &mut NodePin<Vector>,
) -> PinInfo {
    let remote_value = remote::vector(pin, label, self_node.snarl);
    let node = self_node.node_mut();
    let field = field_accessor(node);

    let old_value = field.get();
    let info = vector_view(ui, label, field, remote_value);

    if old_value != field.get() {
        if let Some(caller) = node
            .subscription_ref()
            .and_then(|subscription| subscription.event_caller(Event::OnChange))
        {
            caller(self_node)
        }
    }
    info
}

pub fn display_color_field(
    ui: &mut Ui,
    pin: &InPin,
    mut self_node: SelfNodeMut,
    label: &str,
    field_accessor: impl FnOnce(&mut Node) -> &mut NodePin<Color>,
) -> PinInfo {
    let remote_value = remote::color(pin, label, self_node.snarl);
    let node = self_node.node_mut();
    let field = field_accessor(node);

    let old_value = field.get();
    let info = color_view(ui, label, field, remote_value);

    if old_value != field.get() {
        if let Some(caller) = node
            .subscription_ref()
            .and_then(|subscription| subscription.event_caller(Event::OnChange))
        {
            caller(self_node)
        }
    }
    info
}

pub fn display_material_field(
    ui: &mut Ui,
    pin: &InPin,
    mut self_node: SelfNodeMut,
    label: &str,
    field_accessor: impl FnOnce(&mut Node) -> &mut NodePin<InputMaterial>,
) -> PinInfo {
    let remote_value = remote::node(pin, label, self_node.snarl, |node| matches!(node, Node::Material(_)))
        .map(InputMaterial::External);
    let node = self_node.node_mut();
    let field = field_accessor(node);

    let old_value = field.as_ref().clone();
    let info = material_view(ui, label, field, remote_value);

    if old_value != *field.as_ref() {
        if let Some(caller) = node
            .subscription_ref()
            .and_then(|subscription| subscription.event_caller(Event::OnChange))
        {
            caller(self_node)
        }
    }
    info
}

pub fn display_texture_field(
    ui: &mut Ui,
    pin: &InPin,
    mut self_node: SelfNodeMut,
    label: &str,
    field_accessor: impl FnOnce(&mut Node) -> &mut NodePin<Option<NodeId>>,
) -> PinInfo {
    let remote_value = remote::node(pin, label, self_node.snarl, |remote_node| {
        matches!(remote_node, Node::Texture(_))
    });
    let node = self_node.node_mut();
    let field = field_accessor(node);

    let old_value = field.get();
    field.set(remote_value);

    if old_value != field.get() {
        if let Some(caller) = node
            .subscription_ref()
            .and_then(|subscription| subscription.event_caller(Event::OnChange))
        {
            caller(self_node)
        }
    }
    empty_view(ui, label)
}
