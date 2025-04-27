use eframe::egui_wgpu::RenderState;
use eframe::wgpu::naga::{FastHashSet, FastIndexSet};
use egui::emath::Numeric;
use egui::{Color32, Ui, WidgetText};
use egui_snarl::ui::{AnyPins, PinInfo, SnarlViewer};
use egui_snarl::{InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
use reactor_types::NodePin;

use super::Noded;
use crate::node::message::{CommonNodeResponse, DisplayMessage, DisplayResponse, InputMessage, InterfaceMessage};
use crate::node::{Node, RenderNode};
use crate::tabs::{Tab, ViewportTab};

pub const STRING_COLOR: Color32 = Color32::from_rgb(0x00, 0xb0, 0x00);
pub const NUMBER_COLOR: Color32 = Color32::from_rgb(0xb0, 0x00, 0x00);
pub const VECTOR_COLOR: Color32 = Color32::from_rgb(0x00, 0x00, 0xb0);
pub const MATERIAL_COLOR: Color32 = Color32::from_rgb(0xb0, 0x00, 0xb0);
pub const UNTYPED_COLOR: Color32 = Color32::from_rgb(0xb0, 0xb0, 0xb0);

pub struct NodeConfig {
    pub render_state: RenderState,
    pub max_viewport_resolution: u32,
    pub viewport_tab_titles: FastIndexSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RenderTarget {
    Viewport(String),
}

impl RenderTarget {
    pub fn title(&self) -> &str {
        match self {
            Self::Viewport(title) => title.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderNodeData {
    pub id: NodeId,
    pub output_id: NodeId,
    pub target: RenderTarget,
}

impl RenderNodeData {
    pub fn select(&self, selector: RenderSelector) -> bool {
        match selector {
            RenderSelector::All => true,
            RenderSelector::ById(node_id) => self.id == node_id,
            RenderSelector::ByOutputId(node_id) => self.output_id == node_id,
            RenderSelector::ByTargetTitle(title) => self.target.title() == title,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RenderSelector<'a> {
    All,
    ById(NodeId),
    ByOutputId(NodeId),
    ByTargetTitle(&'a str),
}

pub struct NodeViewer {
    config: NodeConfig,
    output_nodes: FastHashSet<NodeId>,
    render_nodes: FastIndexSet<RenderNodeData>,
}

impl NodeViewer {
    pub fn new<'a>(
        render_state: RenderState,
        max_viewport_resolution: u32,
        open_tabs: impl Iterator<Item = &'a Tab>,
        snarl: &mut Snarl<Node>,
    ) -> Self {
        let open_tabs = open_tabs.collect::<Vec<_>>();
        let mut output_nodes = FastHashSet::default();

        for (node_id, node) in snarl.nodes_ids_mut() {
            if let Some(output_node) = node.output_mut() {
                output_node.set_open_tabs(open_tabs.iter().copied());
                output_nodes.insert(node_id);
            }
        }

        let mut viewer = Self {
            output_nodes,
            render_nodes: Default::default(),
            config: NodeConfig {
                render_state,
                max_viewport_resolution,
                viewport_tab_titles: open_tabs
                    .into_iter()
                    .filter_map(|tab| {
                        if let Tab::Viewport(tab) = tab {
                            Some(tab.title().to_string())
                        } else {
                            None
                        }
                    })
                    .collect(),
            },
        };

        for (from_pin, to_pin) in snarl.wires().collect::<Vec<_>>() {
            viewer.register_render_if_needed(from_pin.node, to_pin.node, snarl);
        }

        viewer
    }

    pub fn open_tab(&mut self, tab: &Tab, snarl: &mut Snarl<Node>) {
        if let Tab::Viewport(tab) = tab {
            self.config.viewport_tab_titles.insert(tab.title().to_string());
        }

        for node_id in &self.output_nodes {
            Node::call_handle_msg(*node_id, snarl, InterfaceMessage::OpenTab(tab));
        }
    }

    pub fn close_tab(&mut self, tab: &Tab, snarl: &mut Snarl<Node>) {
        if let Tab::Viewport(tab) = tab {
            self.config.viewport_tab_titles.shift_remove(tab.title());
        }

        for node_id in &self.output_nodes {
            Node::call_handle_msg(*node_id, snarl, InterfaceMessage::CloseTab(tab));
        }
    }

    pub fn draw(&mut self, tab: &ViewportTab, viewport: &egui::Rect, painter: &egui::Painter, snarl: &mut Snarl<Node>) {
        let selector = RenderSelector::ByTargetTitle(tab.title());
        for render_node_data in &self.render_nodes {
            if render_node_data.select(selector) {
                match snarl.get_node(render_node_data.id).and_then(Node::render_ref) {
                    Some(RenderNode::TriangleRender(render)) => {
                        render.draw(*viewport, painter);
                    },
                    None => (),
                }
            }
        }
    }

    pub fn after_show(&mut self, tab: &ViewportTab, _ui: &mut Ui, response: &egui::Response, snarl: &mut Snarl<Node>) {
        let selector = RenderSelector::ByTargetTitle(tab.title());
        for render_node_data in &self.render_nodes {
            if render_node_data.select(selector) {
                match snarl[render_node_data.id].as_render_mut() {
                    RenderNode::TriangleRender(render) => {
                        let drag = response.drag_delta().x;
                        render.recalc_angle(drag as _);
                    },
                }
            }
        }
    }

    fn register_render_if_needed(&mut self, from_node_id: NodeId, to_node_id: NodeId, snarl: &mut Snarl<Node>) {
        if let Some(output_node) = snarl[to_node_id].output_ref() {
            if let Some(render_node) = snarl[from_node_id].render_ref() {
                if let Some(title) = output_node.selected_title().cloned() {
                    let render_node_data = RenderNodeData {
                        id: from_node_id,
                        output_id: to_node_id,
                        target: RenderTarget::Viewport(title),
                    };
                    if !self.render_nodes.contains(&render_node_data) {
                        render_node.register(&self.config.render_state);
                        self.render_nodes.insert(render_node_data);
                    }
                }
            }
        }
    }

    fn unregister_render_if_needed(&mut self, selector: RenderSelector, snarl: &mut Snarl<Node>) {
        let mut to_unregister_render_nodes = Vec::new();
        let mut to_remove_render_node_idxs = Vec::new();

        for (idx, render_node_data) in self.render_nodes.iter().enumerate() {
            if render_node_data.select(selector) {
                to_unregister_render_nodes.push(render_node_data.id);
                to_remove_render_node_idxs.push(idx);
            }
        }

        for idx in to_remove_render_node_idxs.iter().rev() {
            self.render_nodes.swap_remove_index(*idx);
        }

        for node_id in to_unregister_render_nodes {
            if !self
                .render_nodes
                .iter()
                .any(|render_node_data| render_node_data.id == node_id)
            {
                if let Some(render_node) = snarl.get_node(node_id).and_then(Node::render_ref) {
                    render_node.unregister(&self.config.render_state);
                }
            }
        }
    }

    fn create_node(&mut self, pos: egui::Pos2, factory: fn(&NodeConfig) -> Node, snarl: &mut Snarl<Node>) -> NodeId {
        let node = factory(&self.config);
        let is_output_node = node.output_ref().is_some();
        let node_id = snarl.insert_node(pos, node);

        if is_output_node {
            self.output_nodes.insert(node_id);
        }
        node_id
    }

    fn remove_node(&mut self, node_id: NodeId, snarl: &mut Snarl<Node>) -> Node {
        if snarl[node_id].output_ref().is_some() {
            self.output_nodes.remove(&node_id);
            self.unregister_render_if_needed(RenderSelector::ByOutputId(node_id), snarl);
        }
        snarl.remove_node(node_id)
    }
}

impl SnarlViewer<Node> for NodeViewer {
    #[inline]
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<Node>) {
        // Validate connection
        if snarl[from.id.node].outputs()[from.id.output] & snarl[to.id.node].inputs()[to.id.input] != 0 {
            for &remote in &to.remotes {
                let out_pin = snarl.out_pin(remote);
                self.disconnect(&out_pin, to, snarl);
            }

            snarl.connect(from.id, to.id);
            Node::call_handle_msg(to.id.node, snarl, InputMessage::Connect { from, to });
            // snarl[from.id.node].connect_output(from, to);

            self.register_render_if_needed(from.id.node, to.id.node, snarl);
        }
    }

    #[inline]
    fn disconnect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<Node>) {
        Node::call_handle_msg(to.id.node, snarl, InputMessage::Disconnect { from, to });
        snarl.disconnect(from.id, to.id);
        self.unregister_render_if_needed(RenderSelector::ById(from.id.node), snarl);
    }

    #[inline]
    fn drop_inputs(&mut self, pin: &InPin, snarl: &mut Snarl<Node>) {
        snarl.drop_inputs(pin.id);
    }

    fn title(&mut self, node: &Node) -> String {
        node.name().to_owned()
    }

    fn inputs(&mut self, node: &Node) -> usize {
        node.inputs().len()
    }

    fn outputs(&mut self, node: &Node) -> usize {
        node.outputs().len()
    }

    #[allow(refining_impl_trait)]
    fn show_input(&mut self, pin: &InPin, ui: &mut Ui, snarl: &mut Snarl<Node>) -> PinInfo {
        let response = Node::call_handle_msg(pin.id.node, snarl, DisplayMessage::Input { pin, ui });
        match response {
            Some(CommonNodeResponse::Display(DisplayResponse::Info(pin_info))) => pin_info,
            _ => unreachable!("{} node has no inputs", snarl[pin.id.node].name()),
        }
    }

    #[allow(refining_impl_trait)]
    fn show_output(&mut self, pin: &OutPin, ui: &mut Ui, snarl: &mut Snarl<Node>) -> PinInfo {
        match &mut snarl[pin.id.node] {
            Node::Output(_) => {
                unreachable!("Output node has no outputs")
            },
            _ => PinInfo::circle().with_fill(UNTYPED_COLOR),
        }
    }

    fn has_body(&mut self, node: &Node) -> bool {
        node.output_ref().is_some()
    }

    fn show_body(&mut self, node: NodeId, inputs: &[InPin], outputs: &[OutPin], ui: &mut Ui, snarl: &mut Snarl<Node>) {
        match Node::call_handle_msg(node, snarl, DisplayMessage::Body { inputs, outputs, ui }) {
            Some(CommonNodeResponse::Display(DisplayResponse::Selected(selected_tab))) => {
                let output_node_id = selected_tab.node_id;

                self.unregister_render_if_needed(RenderSelector::ByOutputId(output_node_id), snarl);

                for (from_pin, to_pin) in snarl
                    .wires()
                    .filter(|&(_, to_pin)| to_pin.node == output_node_id)
                    .collect::<Vec<_>>()
                {
                    self.register_render_if_needed(from_pin.node, to_pin.node, snarl);
                }
            },
            _ => (),
        }
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<Node>) -> bool {
        true
    }

    fn show_graph_menu(&mut self, pos: egui::Pos2, ui: &mut Ui, snarl: &mut Snarl<Node>) {
        ui.label("Add node");
        for (name, factory, ..) in Node::fabrics() {
            if ui.button(name).clicked() {
                self.create_node(pos, factory, snarl);
                ui.close_menu();
            }
        }
    }

    fn has_dropped_wire_menu(&mut self, _src_pins: AnyPins, _snarl: &mut Snarl<Node>) -> bool {
        true
    }

    fn show_dropped_wire_menu(&mut self, pos: egui::Pos2, ui: &mut Ui, src_pins: AnyPins, snarl: &mut Snarl<Node>) {
        ui.label("Add node");
        match src_pins {
            AnyPins::Out(src_pin_ids) => {
                for src_pin_id in src_pin_ids {
                    let src_out = snarl[src_pin_id.node].outputs()[src_pin_id.output];
                    let dst_in_candidates = Node::fabrics().into_iter().filter_map(|(name, factory, inputs, _)| {
                        inputs
                            .iter()
                            .position(|input| *input & src_out != 0)
                            .map(|idx| (name, factory, idx))
                    });

                    for (name, factory, idx) in dst_in_candidates {
                        if ui.button(name).clicked() {
                            // Create new node.
                            let node_id = self.create_node(pos, factory, snarl);

                            // Connect the wire.
                            let src_pin = snarl.out_pin(*src_pin_id);
                            let dst_pin = InPin {
                                id: InPinId {
                                    node: node_id,
                                    input: idx,
                                },
                                remotes: Default::default(),
                            };
                            self.connect(&src_pin, &dst_pin, snarl);

                            ui.close_menu();
                        }
                    }
                }
            },
            AnyPins::In(src_pin_ids) => {
                for src_pin_id in src_pin_ids {
                    let src_in = snarl[src_pin_id.node].inputs()[src_pin_id.input];
                    let dst_out_candidates = Node::fabrics().into_iter().filter_map(|(name, factory, _, outputs)| {
                        outputs
                            .iter()
                            .position(|output| *output & src_in != 0)
                            .map(|idx| (name, factory, idx))
                    });

                    for (name, factory, idx) in dst_out_candidates {
                        if ui.button(name).clicked() {
                            // Create new node.
                            let node_id = self.create_node(pos, factory, snarl);

                            // Connect the wire.
                            let dst_pin = OutPin {
                                id: OutPinId {
                                    node: node_id,
                                    output: idx,
                                },
                                remotes: Default::default(),
                            };
                            let src_pin = snarl.in_pin(*src_pin_id);
                            self.connect(&dst_pin, &src_pin, snarl);

                            ui.close_menu();
                        }
                    }
                }
            },
        };
    }

    fn has_node_menu(&mut self, _node: &Node) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node_id: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<Node>,
    ) {
        ui.label("Node menu");
        if ui.button("Remove").clicked() {
            for in_pin in inputs {
                for out_pin_id in &in_pin.remotes {
                    let out_pin = snarl.out_pin(*out_pin_id);
                    self.disconnect(&out_pin, in_pin, snarl);
                }
            }

            for out_pin in outputs {
                for in_pin_id in &out_pin.remotes {
                    let in_pin = snarl.in_pin(*in_pin_id);
                    self.disconnect(out_pin, &in_pin, snarl);
                }
            }

            self.remove_node(node_id, snarl);

            ui.close_menu();
        }
    }

    fn has_on_hover_popup(&mut self, _: &Node) -> bool {
        true
    }

    fn show_on_hover_popup(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<Node>,
    ) {
        match snarl[node] {
            Node::Output(_) => {
                ui.label("Displays anything connected to it");
            },
            _ => {
                ui.label("<No description available>");
            },
        }
    }

    fn header_frame(
        &mut self,
        frame: egui::Frame,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        snarl: &Snarl<Node>,
    ) -> egui::Frame {
        match snarl[node] {
            Node::Output(_) => frame.fill(egui::Color32::from_rgb(70, 70, 80)),
            _ => frame.fill(egui::Color32::from_rgb(40, 40, 70)),
        }
    }
}

pub fn format_float(value: f64) -> String {
    let value = (value * 1000.0).round() / 1000.0;
    format!("{value}")
}

pub fn number_input_remote_value(pin: &InPin, snarl: &Snarl<Node>, label: &str) -> Option<(&'static str, f64)> {
    match &*pin.remotes {
        [] => None,
        [remote] => Some(match &snarl[remote.node] {
            node => unreachable!("{label} input not suppor connection with `{}`", node.name()),
        }),
        _ => None,
    }
}

pub fn number_input_view<N>(
    ui: &mut Ui,
    label: &str,
    node_pin: &mut NodePin<N>,
    remote_value: Option<(&'static str, N)>,
) -> PinInfo
where
    N: Numeric,
{
    ui.horizontal(|ui| {
        ui.label(label);
        let enabled = match remote_value {
            None => true,
            Some(remote) => {
                node_pin.set(remote.1);
                false
            },
        };
        ui.add_enabled(enabled, egui::DragValue::new(node_pin.as_mut()));
    });
    PinInfo::circle().with_fill(NUMBER_COLOR)
}

pub fn as_number_input_view<N, M>(
    ui: &mut Ui,
    label: &str,
    node_pin: &mut NodePin<N>,
    remote_value: Option<(&'static str, M)>,
) -> PinInfo
where
    N: AsMut<f64>,
    M: Into<N>,
{
    ui.horizontal(|ui| {
        ui.label(label);
        let enabled = match remote_value {
            None => true,
            Some(remote) => {
                node_pin.set(remote.1.into());
                false
            },
        };
        ui.add_enabled(enabled, egui::DragValue::new(node_pin.as_mut().as_mut()));
    });
    PinInfo::circle().with_fill(NUMBER_COLOR)
}

pub fn empty_input_view(ui: &mut Ui, label: impl Into<WidgetText>) -> PinInfo {
    ui.label(label);
    PinInfo::circle().with_fill(UNTYPED_COLOR)
}
