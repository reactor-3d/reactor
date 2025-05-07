use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin};
use serde::{Deserialize, Serialize};

use crate::node::message::{MessageHandling, SelfNodeMut};
use crate::node::subscribtion::{Event, Subscription};
use crate::node::viewer::ui::{input, output};
use crate::node::{Node, NodeFlags, Noded, collect_for_node};

#[derive(Clone, Serialize, Deserialize)]
pub struct CollectionNode {
    nodes: Vec<NodeId>,
    inputs: Vec<u64>,

    #[serde(skip)]
    subscription: Subscription,
}

impl Default for CollectionNode {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            inputs: vec![NodeFlags::ALL.bits()],
            subscription: Subscription::default(),
        }
    }
}

impl CollectionNode {
    pub const NAME: &str = "Collection";
    pub const INPUT: u64 = NodeFlags::ALL.bits();
    pub const OUTPUTS: [u64; 1] = [NodeFlags::COLLECTION.bits()];

    pub fn insert(&mut self, idx: usize, node: NodeId) {
        self.nodes.insert(idx, node);
        self.inputs.insert(idx, NodeFlags::ALL.bits());
    }

    pub fn remove(&mut self, idx: usize) {
        self.nodes.remove(idx);
        self.inputs.remove(idx);
    }

    pub fn to_node_ids(&self) -> Vec<NodeId> {
        self.nodes.clone()
    }
}

impl Noded for CollectionNode {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn inputs(&self) -> &[u64] {
        &self.inputs
    }

    fn outputs(&self) -> &[u64] {
        &Self::OUTPUTS
    }

    fn subscription_ref(&self) -> Option<&Subscription> {
        Some(&self.subscription)
    }

    fn subscription_mut(&mut self) -> Option<&mut Subscription> {
        Some(&mut self.subscription)
    }
}

impl MessageHandling for CollectionNode {
    fn handle_display_input(self_node: SelfNodeMut, pin: &InPin, ui: &mut Ui) -> Option<PinInfo> {
        let name = pin
            .remotes
            .first()
            .map(|out_pin| self_node.snarl[out_pin.node].name())
            .unwrap_or_default();

        Some(input::empty_view(ui, format!("{} {name}", pin.id.input + 1)))
    }

    fn handle_display_output(_self_node: SelfNodeMut, _pin: &OutPin, _ui: &mut Ui) -> Option<PinInfo> {
        Some(output::empty_view())
    }

    fn handle_input_connect(mut self_node: SelfNodeMut, from: &OutPin, to: &InPin) {
        let node = self_node.node_mut().as_collection_mut();
        node.insert(to.id.input, from.id.node);

        if let Some(caller) = node.subscription.event_caller(Event::OnChange) {
            caller(self_node)
        }
    }

    fn handle_input_disconnect(mut self_node: SelfNodeMut, _from: &OutPin, to: &InPin) {
        let node = self_node.node_mut().as_collection_mut();
        node.remove(to.id.input);

        if let Some(caller) = node.subscription.event_caller(Event::OnChange) {
            caller(self_node)
        }
    }

    fn handle_input_collect_ids(
        self_node: SelfNodeMut,
        predicate: &dyn Fn(&Node) -> bool,
        destination: &mut eframe::wgpu::naga::FastIndexSet<NodeId>,
    ) {
        let nodes = self_node.node_ref().as_collection_ref().to_node_ids();
        for node_id in nodes {
            collect_for_node(Some(node_id), predicate, destination, self_node.snarl)
        }
    }
}
