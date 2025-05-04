use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin};
use reactor_derives::Noded;
use reactor_types::{Color, Float, NodePin};
use serde::{Deserialize, Serialize};

use crate::node::message::{MessageHandling, SelfNodeMut};
use crate::node::subscribtion::Subscription;
use crate::node::viewer::ui::{input, output};
use crate::node::{Node, NodeFlags, Noded, collect_for_node};

#[derive(Clone, Default, Serialize, Deserialize, Noded, PartialEq)]
pub struct MetalNode {
    albedo: NodePin<Color>,
    fuzz: NodePin<Float>,
    texture: NodePin<Option<NodeId>>,

    #[serde(skip)]
    subscription: Subscription,
}

impl MetalNode {
    pub const NAME: &str = "Metal Material";
    pub const INPUTS: [u64; 3] = [
        NodeFlags::TYPICAL_VECTOR_INPUT.bits(),
        NodeFlags::TYPICAL_NUMBER_INPUT.bits(),
        NodeFlags::TEXTURE.bits(),
    ];
    pub const OUTPUTS: [u64; 1] = [NodeFlags::MATERIAL_METAL.bits()];

    pub fn albedo(&self) -> Color {
        self.albedo.get()
    }

    pub fn fuzz(&self) -> Float {
        self.fuzz.get()
    }

    pub fn texture(&self) -> Option<NodeId> {
        self.texture.get()
    }
}

impl MessageHandling for MetalNode {
    fn handle_display_input(self_node: SelfNodeMut, pin: &InPin, ui: &mut Ui) -> Option<PinInfo> {
        match pin.id.input {
            0 => Some(input::display_color_field(ui, pin, self_node, "Albedo", |node| {
                &mut node.as_material_mut().as_metal_mut().albedo
            })),
            1 => Some(input::display_number_field(ui, pin, self_node, "Fuzz", |node| {
                &mut node.as_material_mut().as_metal_mut().fuzz
            })),
            2 => Some(input::display_texture_field(ui, pin, self_node, "Texture", |node| {
                &mut node.as_material_mut().as_metal_mut().texture
            })),
            _ => None,
        }
    }

    fn handle_display_output(_self_node: SelfNodeMut, _pin: &OutPin, _ui: &mut Ui) -> Option<PinInfo> {
        Some(output::empty_view())
    }

    fn handle_input_collect_ids(
        self_node: SelfNodeMut,
        predicate: &dyn Fn(&Node) -> bool,
        destination: &mut eframe::wgpu::naga::FastIndexSet<NodeId>,
    ) {
        collect_for_node(
            self_node.node_ref().as_material_ref().get_texture_node_id(),
            predicate,
            destination,
            self_node.snarl,
        );
    }
}
