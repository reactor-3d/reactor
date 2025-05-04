use egui::Ui;
use egui_snarl::OutPin;
use egui_snarl::ui::PinInfo;
use reactor_derives::Noded;
use reactor_types::{Vector, Vector2, Vector3, Vector4};
use serde::{Deserialize, Serialize};

use crate::node::message::{MessageHandling, SelfNodeMut};
use crate::node::viewer::ui::output;
use crate::node::{NodeFlags, Noded};

#[derive(Clone, Copy, Serialize, Deserialize, Noded)]
pub struct VectorNode {
    value: Vector,
}

impl VectorNode {
    pub const NAME: &str = "Vector";
    pub const INPUTS: [u64; 0] = [];
    pub const OUTPUTS: [u64; 1] = [NodeFlags::VECTOR.bits()];

    pub fn new_dim2() -> Self {
        Self {
            value: Vector::Dim2(Vector2::default()),
        }
    }

    pub fn new_dim3() -> Self {
        Self {
            value: Vector::Dim3(Vector3::default()),
        }
    }

    pub fn new_dim4() -> Self {
        Self {
            value: Vector::Dim4(Vector4::default()),
        }
    }

    pub fn value(&self) -> Vector {
        self.value
    }
}

impl MessageHandling for VectorNode {
    fn handle_display_output(mut self_node: SelfNodeMut, pin: &OutPin, ui: &mut Ui) -> Option<PinInfo> {
        if pin.id.output == 0 {
            let node = self_node.node_mut().as_vector_mut();

            Some(output::vector_view(ui, "", &mut node.value))
        } else {
            None
        }
    }
}
