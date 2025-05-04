use egui::Ui;
use egui_snarl::OutPin;
use egui_snarl::ui::PinInfo;
use reactor_derives::Noded;
use serde::{Deserialize, Serialize};

use crate::node::message::{MessageHandling, SelfNodeMut};
use crate::node::viewer::ui::output;
use crate::node::{NodeFlags, Noded};

#[derive(Clone, Default, Serialize, Deserialize, Noded)]
pub struct StringNode {
    value: String,
}

impl StringNode {
    pub const NAME: &str = "String";
    pub const INPUTS: [u64; 0] = [];
    pub const OUTPUTS: [u64; 1] = [NodeFlags::STRING.bits()];

    pub fn value(&self) -> &String {
        &self.value
    }
}

impl MessageHandling for StringNode {
    fn handle_display_output(mut self_node: SelfNodeMut, pin: &OutPin, ui: &mut Ui) -> Option<PinInfo> {
        if pin.id.output == 0 {
            let node = self_node.node_mut().as_string_mut();

            Some(output::string_view(ui, "", &mut node.value))
        } else {
            None
        }
    }
}
