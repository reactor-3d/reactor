use bitflags::bitflags;
use eframe::wgpu::naga::FastIndexSet;
use egui_snarl::{NodeId, Snarl};
use message::InputMessage;
use serde::{Deserialize, Serialize};

use self::message::{CommonNodeMessage, CommonNodeResponse, MessageHandling, SelfNodeMut};
use self::output::OutputNode;
use self::render::RenderNode;
use self::render::triangle::TriangleRenderNode;
use self::viewer::NodeConfig;

pub mod message;
pub mod output;
pub mod render;
pub mod subscribtion;
pub mod viewer;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct NodeFlags: u64 {
        const RENDER_TRIANGLE = 0b00000001;
        const RENDERS = Self::RENDER_TRIANGLE.bits();

        const OUTPUT = Self::RENDER_TRIANGLE.bits() << 1;
        const NUMBER = Self::OUTPUT.bits() << 1;

        const ALL = u64::MAX;
        const TYPICAL_NUMBER_INPUT = NodeFlags::NUMBER.bits();
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Node {
    Render(RenderNode),
    Output(OutputNode),
}

impl Node {
    pub fn fabrics() -> impl IntoIterator<Item = (&'static str, fn(&NodeConfig) -> Node, &'static [u64], &'static [u64])>
    {
        [
            (
                TriangleRenderNode::NAME,
                (|_| Node::Render(RenderNode::Triangle(TriangleRenderNode::default()))) as fn(&NodeConfig) -> Node,
                TriangleRenderNode::INPUTS.as_slice(),
                TriangleRenderNode::OUTPUTS.as_slice(),
            ),
            (
                OutputNode::NAME,
                |config| {
                    Node::Output(OutputNode::new(
                        config.viewport_tab_titles.iter().cloned().collect(),
                        None,
                    ))
                },
                OutputNode::INPUTS.as_slice(),
                OutputNode::OUTPUTS.as_slice(),
            ),
        ]
    }

    pub const fn name(&self) -> &str {
        match self {
            Self::Render(RenderNode::Triangle(_)) => TriangleRenderNode::NAME,
            Self::Output(_) => OutputNode::NAME,
        }
    }

    pub fn inputs(&self) -> &[u64] {
        match self {
            Self::Render(render) => render.inputs(),
            Self::Output(output) => output.inputs(),
        }
    }

    pub fn outputs(&self) -> &[u64] {
        match self {
            Self::Render(render) => render.outputs(),
            Self::Output(output) => output.outputs(),
        }
    }

    pub fn call_handle_msg<'a>(
        self_id: NodeId,
        snarl: &mut Snarl<Node>,
        msg: impl Into<CommonNodeMessage<'a>>,
    ) -> Option<CommonNodeResponse> {
        let self_node = SelfNodeMut::new(self_id, snarl);
        Self::handle_msg(self_node, msg)
    }

    pub fn handle_msg<'a>(self_node: SelfNodeMut, msg: impl Into<CommonNodeMessage<'a>>) -> Option<CommonNodeResponse> {
        let msg = msg.into();

        match self_node.node_ref() {
            Self::Render(_) => RenderNode::handle_msg(self_node, msg),
            Self::Output(_) => OutputNode::handle_msg(self_node, msg),
        }
    }

    pub fn render_node_ref(&self) -> Option<&RenderNode> {
        match self {
            Self::Render(render_node) => Some(render_node),
            _ => None,
        }
    }

    pub fn as_render_node_ref(&self) -> &RenderNode {
        self.render_node_ref()
            .unwrap_or_else(|| panic!("Node `{}` is not a `{}`", self.name(), RenderNode::NAME))
    }

    pub fn as_render_node_mut(&mut self) -> &mut RenderNode {
        match self {
            Self::Render(render_node) => render_node,
            node => panic!("Node `{}` is not an `{}`", node.name(), RenderNode::NAME),
        }
    }

    pub fn output_node_ref(&self) -> Option<&OutputNode> {
        match self {
            Self::Output(output_node) => Some(output_node),
            _ => None,
        }
    }

    pub fn output_node_mut(&mut self) -> Option<&mut OutputNode> {
        match self {
            Self::Output(output_node) => Some(output_node),
            _ => None,
        }
    }

    pub fn as_output_node_mut(&mut self) -> &mut OutputNode {
        match self {
            Self::Output(output_node) => output_node,
            node => panic!("Node `{}` is not an `{}`", node.name(), OutputNode::NAME),
        }
    }
}

pub fn collect_for_node(
    node_id: Option<NodeId>,
    predicate: &dyn Fn(&Node) -> bool,
    destination: &mut FastIndexSet<NodeId>,
    snarl: &mut Snarl<Node>,
) {
    if let Some(node_id) = node_id {
        let self_node = SelfNodeMut::new(node_id, snarl);
        let need_insert = predicate(self_node.node_ref());

        Node::handle_msg(
            self_node,
            CommonNodeMessage::Input(InputMessage::CollectIds { predicate, destination }),
        );
        if need_insert {
            destination.insert(node_id);
        }
    }
}
