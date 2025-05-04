use eframe::wgpu::naga::FastIndexSet;
use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin};
use reactor_derives::Noded;
use reactor_types::{NodePin, Vector};
use serde::{Deserialize, Serialize};

use crate::node::message::{MessageHandling, SelfNodeMut};
use crate::node::subscribtion::Subscription;
use crate::node::viewer::ui::{input, output};
use crate::node::{Node, NodeFlags, Noded, collect_for_node};

#[derive(Clone, Serialize, Deserialize, Noded, PartialEq)]
pub struct EmissiveNode {
    emit: NodePin<Vector>,
    texture: NodePin<Option<NodeId>>,

    #[serde(skip)]
    subscription: Subscription,
}

impl Default for EmissiveNode {
    fn default() -> Self {
        Self {
            emit: NodePin::new(Vector::Dim3(Default::default())),
            texture: Default::default(),
            subscription: Default::default(),
        }
    }
}

impl EmissiveNode {
    pub const NAME: &str = "Emissive Material";
    pub const INPUTS: [u64; 2] = [NodeFlags::TYPICAL_VECTOR_INPUT.bits(), NodeFlags::TEXTURE.bits()];
    pub const OUTPUTS: [u64; 1] = [NodeFlags::MATERIAL_EMISSIVE.bits()];

    pub fn texture(&self) -> Option<NodeId> {
        self.texture.get()
    }
}

impl MessageHandling for EmissiveNode {
    fn handle_display_input(self_node: SelfNodeMut, pin: &InPin, ui: &mut Ui) -> Option<PinInfo> {
        match pin.id.input {
            0 => Some(input::display_vector_field(ui, pin, self_node, "Emit", |node| {
                &mut node.as_material_mut().as_emissive_mut().emit
            })),
            1 => Some(input::display_texture_field(ui, pin, self_node, "Texture", |node| {
                &mut node.as_material_mut().as_emissive_mut().texture
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
        destination: &mut FastIndexSet<NodeId>,
    ) {
        collect_for_node(
            self_node.node_ref().as_material_ref().get_texture_node_id(),
            predicate,
            destination,
            self_node.snarl,
        );
    }
}
