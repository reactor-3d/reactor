use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin};
use reactor_derives::Noded;
use reactor_types::{Float, NodePin, Vector};
use serde::{Deserialize, Serialize};

use crate::node::item::InputMaterial;
use crate::node::message::{MessageHandling, SelfNodeMut};
use crate::node::subscribtion::Subscription;
use crate::node::viewer::ui::{input, output};
use crate::node::{Node, NodeFlags, Noded, collect_for_node};

#[derive(Clone, Serialize, Deserialize, Noded)]
pub struct SphereNode {
    center: NodePin<Vector>,
    radius: NodePin<Float>,
    material: NodePin<InputMaterial>,

    #[serde(skip)]
    subscription: Subscription,
}

impl Default for SphereNode {
    fn default() -> Self {
        Self {
            center: NodePin::new(Vector::Dim3(Default::default())),
            radius: NodePin::new(1.0),
            material: Default::default(),
            subscription: Subscription::default(),
        }
    }
}

impl SphereNode {
    pub const NAME: &str = "Sphere Primitive";
    pub const INPUTS: [u64; 3] = [
        NodeFlags::TYPICAL_VECTOR_INPUT.bits(),
        NodeFlags::TYPICAL_NUMBER_INPUT.bits(),
        NodeFlags::MATERIALS.bits(),
    ];
    pub const OUTPUTS: [u64; 1] = [NodeFlags::PRIMITIVE_SPHERE.bits()];
}

impl MessageHandling for SphereNode {
    fn handle_display_input(self_node: SelfNodeMut, pin: &InPin, ui: &mut Ui) -> Option<PinInfo> {
        match pin.id.input {
            0 => Some(input::display_vector_field(ui, pin, self_node, "Center", |node| {
                &mut node.as_primitive_mut().as_sphere_mut().center
            })),
            1 => Some(input::display_number_field(ui, pin, self_node, "Radius", |node| {
                &mut node.as_primitive_mut().as_sphere_mut().radius
            })),
            2 => Some(input::display_material_field(ui, pin, self_node, "Material", |node| {
                &mut node.as_primitive_mut().as_sphere_mut().material
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
        let node = self_node.node_ref().as_primitive_ref().as_sphere_ref();
        if let InputMaterial::External(node_id) = node.material.as_ref() {
            collect_for_node(Some(*node_id), predicate, destination, self_node.snarl);
        }
    }
}
