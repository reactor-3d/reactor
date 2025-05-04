use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, OutPin};
use reactor_derives::Noded;
use reactor_types::{Color, NodePin};
use serde::{Deserialize, Serialize};

use crate::node::message::{MessageHandling, SelfNodeMut};
use crate::node::subscribtion::Subscription;
use crate::node::viewer::ui::{input, output};
use crate::node::{NodeFlags, Noded};

#[derive(Clone, Serialize, Deserialize, Noded, PartialEq)]
pub struct CheckerboardNode {
    even: NodePin<Color>,
    odd: NodePin<Color>,

    #[serde(skip)]
    subscription: Subscription,
}

impl Default for CheckerboardNode {
    fn default() -> Self {
        Self {
            even: NodePin::new(Color::BLACK),
            odd: NodePin::new(Color::WHITE),
            subscription: Subscription::default(),
        }
    }
}

impl CheckerboardNode {
    pub const NAME: &str = "Checkerboard Material";
    pub const INPUTS: [u64; 2] = [
        NodeFlags::TYPICAL_VECTOR_INPUT.bits(),
        NodeFlags::TYPICAL_VECTOR_INPUT.bits(),
    ];
    pub const OUTPUTS: [u64; 1] = [NodeFlags::MATERIAL_CHECKERBOARD.bits()];
}

impl MessageHandling for CheckerboardNode {
    fn handle_display_input(self_node: SelfNodeMut, pin: &InPin, ui: &mut Ui) -> Option<PinInfo> {
        match pin.id.input {
            0 => Some(input::display_color_field(ui, pin, self_node, "Even", |node| {
                &mut node.as_material_mut().as_checkerboard_mut().even
            })),
            1 => Some(input::display_color_field(ui, pin, self_node, "Odd", |node| {
                &mut node.as_material_mut().as_checkerboard_mut().odd
            })),
            _ => None,
        }
    }

    fn handle_display_output(_self_node: SelfNodeMut, _pin: &OutPin, _ui: &mut Ui) -> Option<PinInfo> {
        Some(output::empty_view())
    }
}
