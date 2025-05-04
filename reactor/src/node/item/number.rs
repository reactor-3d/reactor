use egui::Ui;
use egui_snarl::OutPin;
use egui_snarl::ui::PinInfo;
use reactor_derives::Noded;
use reactor_types::Float;
use serde::{Deserialize, Serialize};

use crate::node::message::{MessageHandling, SelfNodeMut};
use crate::node::viewer::ui::output;
use crate::node::{NodeFlags, Noded};

#[derive(Clone, Copy, Default, Serialize, Deserialize, Noded)]
pub struct NumberNode {
    value: Float,
}

impl NumberNode {
    pub const NAME: &str = "Number";
    pub const INPUTS: [u64; 0] = [];
    pub const OUTPUTS: [u64; 1] = [NodeFlags::NUMBER.bits()];

    pub fn value(&self) -> Float {
        self.value
    }
}

impl MessageHandling for NumberNode {
    fn handle_display_output(mut self_node: SelfNodeMut, pin: &OutPin, ui: &mut Ui) -> Option<PinInfo> {
        if pin.id.output == 0 {
            let node = self_node.node_mut().as_number_mut();

            Some(output::number_view(ui, "", &mut node.value))
        } else {
            None
        }
    }
}
