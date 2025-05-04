use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, OutPin};
use reactor_derives::Noded;
use reactor_types::NodePin;
use serde::{Deserialize, Serialize};

use crate::node::message::{MessageHandling, SelfNodeMut};
use crate::node::subscribtion::{Event, Subscription};
use crate::node::viewer::ui::{input, output};
use crate::node::{NodeFlags, Noded};

#[derive(Clone, Serialize, Deserialize, Noded)]
pub struct TextureNode {
    path: String,
    scale: NodePin<f64>,

    #[serde(skip)]
    subscription: Subscription,
}

impl Default for TextureNode {
    fn default() -> Self {
        Self {
            path: Default::default(),
            scale: NodePin::new(1.0),
            subscription: Default::default(),
        }
    }
}

impl TextureNode {
    pub const NAME: &str = "Texture";
    pub const INPUTS: [u64; 1] = [NodeFlags::TYPICAL_NUMBER_INPUT.bits()];
    pub const OUTPUTS: [u64; 1] = [NodeFlags::TEXTURE.bits() | NodeFlags::STRING.bits()];
}

impl MessageHandling for TextureNode {
    fn handle_display_input(self_node: SelfNodeMut, pin: &InPin, ui: &mut Ui) -> Option<PinInfo> {
        match pin.id.input {
            0 => Some(input::display_number_field(ui, pin, self_node, "Scale", |node| {
                &mut node.as_texture_mut().scale
            })),
            _ => None,
        }
    }

    fn handle_display_output(mut self_node: SelfNodeMut, pin: &OutPin, ui: &mut Ui) -> Option<PinInfo> {
        if pin.id.output == 0 {
            let node = self_node.node_mut().as_texture_mut();

            let old_value = node.path.clone();
            let info = output::string_view(ui, "", &mut node.path);

            if old_value != node.path {
                if let Some(caller) = node.subscription.event_caller(Event::OnChange) {
                    caller(self_node);
                }
            }

            Some(info)
        } else {
            None
        }
    }
}
