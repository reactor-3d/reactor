use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, OutPin};
use reactor_derives::Noded;
use reactor_types::{Float, NodePin};
use serde::{Deserialize, Serialize};

use crate::node::message::{MessageHandling, SelfNodeMut};
use crate::node::subscribtion::Subscription;
use crate::node::viewer::ui::{input, output};
use crate::node::{NodeFlags, Noded};

#[derive(Clone, Default, Serialize, Deserialize, Noded, PartialEq)]
pub struct DielectricNode {
    ior: NodePin<Float>,

    #[serde(skip)]
    subscription: Subscription,
}

impl DielectricNode {
    pub const NAME: &str = "Dielectric Material";
    pub const INPUTS: [u64; 1] = [NodeFlags::TYPICAL_NUMBER_INPUT.bits()];
    pub const OUTPUTS: [u64; 1] = [NodeFlags::MATERIAL_DIELECTRIC.bits()];

    pub fn ior(&self) -> Float {
        self.ior.get()
    }
}

impl MessageHandling for DielectricNode {
    fn handle_display_input(self_node: SelfNodeMut, pin: &InPin, ui: &mut Ui) -> Option<PinInfo> {
        match pin.id.input {
            0 => Some(input::display_number_field(ui, pin, self_node, "IOR", |node| {
                &mut node.as_material_mut().as_dielectric_mut().ior
            })),
            _ => None,
        }
    }

    fn handle_display_output(_self_node: SelfNodeMut, _pin: &OutPin, _ui: &mut Ui) -> Option<PinInfo> {
        Some(output::empty_view())
    }
}
