use eframe::wgpu::naga::FastIndexSet;
use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin, Snarl};

use super::Node;
use super::item::{OutputNode, RenderNode};
use super::subscribtion::{Event, EventCallback};
use crate::tabs::Tab;

pub enum InputMessage<'a> {
    Connect {
        from: &'a OutPin,
        to: &'a InPin,
    },
    Disconnect {
        from: &'a OutPin,
        to: &'a InPin,
    },
    CollectIds {
        predicate: &'a dyn Fn(&Node) -> bool,
        destination: &'a mut FastIndexSet<NodeId>,
    },
}

pub enum InputResponse {}

pub enum EventMessage {
    HasSubscription {
        node_id: NodeId,
        event: Event,
    },
    Subscribe {
        node_id: NodeId,
        event: Event,
        callback: EventCallback,
    },
    Unsubscribe {
        node_id: NodeId,
        event: Event,
    },
}

pub enum EventResponse {
    HasSubscription(bool),
}

pub enum DisplayMessage<'a> {
    Input {
        pin: &'a InPin,
        ui: &'a mut Ui,
    },
    Body {
        inputs: &'a [InPin],
        outputs: &'a [OutPin],
        ui: &'a mut Ui,
    },
}

#[derive(Debug, Copy, Clone)]
pub struct SelectedTab<'a> {
    pub title: &'a str,
    pub node_id: NodeId,
}

pub enum DisplayResponse<'a> {
    Info(PinInfo),
    Selected(SelectedTab<'a>),
}

pub enum InterfaceMessage<'a> {
    OpenTab(&'a Tab),
    CloseTab(&'a Tab),
}

pub enum InterfaceResponse {}

pub enum CommonNodeMessage<'a> {
    Input(InputMessage<'a>),
    Event(EventMessage),
    Display(DisplayMessage<'a>),
    Interface(InterfaceMessage<'a>),
}

impl<'a> From<InputMessage<'a>> for CommonNodeMessage<'a> {
    fn from(msg: InputMessage<'a>) -> Self {
        Self::Input(msg)
    }
}

impl From<EventMessage> for CommonNodeMessage<'_> {
    fn from(msg: EventMessage) -> Self {
        Self::Event(msg)
    }
}

impl<'a> From<DisplayMessage<'a>> for CommonNodeMessage<'a> {
    fn from(msg: DisplayMessage<'a>) -> Self {
        Self::Display(msg)
    }
}

impl<'a> From<InterfaceMessage<'a>> for CommonNodeMessage<'a> {
    fn from(msg: InterfaceMessage<'a>) -> Self {
        Self::Interface(msg)
    }
}

pub enum CommonNodeResponse<'a> {
    Input(InputResponse),
    Event(EventResponse),
    Display(DisplayResponse<'a>),
    Interface(InterfaceResponse),
}

pub struct SelfNodeMut<'a> {
    pub id: NodeId,
    pub snarl: &'a mut Snarl<Node>,
}

impl<'a> SelfNodeMut<'a> {
    pub fn new(id: NodeId, snarl: &'a mut Snarl<Node>) -> Self {
        Self { id, snarl }
    }
}

impl SelfNodeMut<'_> {
    pub fn node_by_id_ref(&self, id: NodeId) -> &Node {
        &self.snarl[id]
    }

    pub fn node_by_id_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.snarl[id]
    }

    pub fn node_ref(&self) -> &Node {
        self.node_by_id_ref(self.id)
    }

    pub fn node_mut(&mut self) -> &mut Node {
        self.node_by_id_mut(self.id)
    }

    pub fn as_render_node_ref(&self) -> &RenderNode {
        self.node_ref().as_render_ref()
    }

    pub fn as_render_node_mut(&mut self) -> &mut RenderNode {
        self.node_mut().as_render_mut()
    }

    pub fn as_output_node_mut(&mut self) -> &mut OutputNode {
        self.node_mut().as_output_mut()
    }
}

pub trait MessageHandling {
    fn handle_input(self_node: SelfNodeMut, input_msg: InputMessage) -> Option<InputResponse> {
        match input_msg {
            InputMessage::Connect { from, to } => {
                Self::handle_input_connect(self_node, from, to);
                None
            },
            InputMessage::Disconnect { from, to } => {
                Self::handle_input_disconnect(self_node, from, to);
                None
            },
            InputMessage::CollectIds { predicate, destination } => {
                Self::handle_input_collect_ids(self_node, predicate, destination);
                None
            },
        }
    }

    #[allow(unused_variables)]
    fn handle_input_connect(self_node: SelfNodeMut, from: &OutPin, to: &InPin) {}

    #[allow(unused_variables)]
    fn handle_input_disconnect(self_node: SelfNodeMut, from: &OutPin, to: &InPin) {}

    #[allow(unused_variables)]
    fn handle_input_collect_ids(
        self_node: SelfNodeMut,
        predicate: &dyn Fn(&Node) -> bool,
        destination: &mut FastIndexSet<NodeId>,
    ) {
    }

    fn handle_event(self_node: SelfNodeMut, event_msg: EventMessage) -> Option<EventResponse> {
        match event_msg {
            EventMessage::HasSubscription { node_id, event } => {
                let response = Self::handle_event_has_subscription(self_node, node_id, event);
                Some(EventResponse::HasSubscription(response))
            },
            EventMessage::Subscribe {
                node_id,
                event,
                callback,
            } => {
                Self::handle_event_subscribe(self_node, node_id, event, callback);
                None
            },
            EventMessage::Unsubscribe { node_id, event } => {
                Self::handle_event_unsubscribe(self_node, node_id, event);
                None
            },
        }
    }

    #[allow(unused_variables)]
    fn handle_event_has_subscription(self_node: SelfNodeMut, node_id: NodeId, event: Event) -> bool {
        false
    }

    #[allow(unused_variables)]
    fn handle_event_subscribe(self_node: SelfNodeMut, node_id: NodeId, event: Event, callback: EventCallback) {}

    #[allow(unused_variables)]
    fn handle_event_unsubscribe(self_node: SelfNodeMut, node_id: NodeId, event: Event) {}

    fn handle_display<'a>(self_node: SelfNodeMut<'a>, display_msg: DisplayMessage) -> Option<DisplayResponse<'a>> {
        match display_msg {
            DisplayMessage::Input { pin, ui } => {
                Self::handle_display_input(self_node, pin, ui).map(DisplayResponse::Info)
            },
            DisplayMessage::Body { inputs, outputs, ui } => {
                Self::handle_display_body(self_node, inputs, outputs, ui).map(DisplayResponse::Selected)
            },
        }
    }

    #[allow(unused_variables)]
    fn handle_display_input(self_node: SelfNodeMut, pin: &InPin, ui: &mut Ui) -> Option<PinInfo> {
        None
    }

    #[allow(unused_variables)]
    fn handle_display_body<'a>(
        self_node: SelfNodeMut<'a>,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
    ) -> Option<SelectedTab<'a>> {
        None
    }

    fn handle_interface(self_node: SelfNodeMut, interface_msg: InterfaceMessage) -> Option<InterfaceResponse> {
        match interface_msg {
            InterfaceMessage::OpenTab(tab) => {
                Self::handle_interface_open_tab(self_node, tab);
                None
            },
            InterfaceMessage::CloseTab(tab) => {
                Self::handle_interface_close_tab(self_node, tab);
                None
            },
        }
    }

    #[allow(unused_variables)]
    fn handle_interface_open_tab(self_node: SelfNodeMut, tab: &Tab) {}

    #[allow(unused_variables)]
    fn handle_interface_close_tab(self_node: SelfNodeMut, tab: &Tab) {}

    fn handle_msg<'a>(self_node: SelfNodeMut<'a>, msg: CommonNodeMessage) -> Option<CommonNodeResponse<'a>> {
        match msg {
            CommonNodeMessage::Input(input_msg) => {
                Self::handle_input(self_node, input_msg).map(CommonNodeResponse::Input)
            },
            CommonNodeMessage::Event(event_msg) => {
                Self::handle_event(self_node, event_msg).map(CommonNodeResponse::Event)
            },
            CommonNodeMessage::Display(display_msg) => {
                Self::handle_display(self_node, display_msg).map(CommonNodeResponse::Display)
            },
            CommonNodeMessage::Interface(interface_msg) => {
                Self::handle_interface(self_node, interface_msg).map(CommonNodeResponse::Interface)
            },
        }
    }
}
