use bitflags::bitflags;
use eframe::wgpu::naga::FastIndexSet;
use egui_snarl::{NodeId, Snarl};
use enum_dispatch::enum_dispatch;
use message::InputMessage;
use reactor_derives::EnumAs;
use serde::{Deserialize, Serialize};

use self::item::{OutputNode, RenderNode, TriangleRenderNode};
use self::message::{CommonNodeMessage, CommonNodeResponse, MessageHandling, SelfNodeMut};
use self::viewer::NodeConfig;

pub mod item;
pub mod message;
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

#[enum_dispatch]
pub trait Noded {
    fn name(&self) -> &str;
    fn inputs(&self) -> &[u64];
    fn outputs(&self) -> &[u64];
}

#[derive(Clone, EnumAs, Serialize, Deserialize)]
#[enum_dispatch(Noded)]
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
                (|_| Node::Render(RenderNode::TriangleRender(TriangleRenderNode::default())))
                    as fn(&NodeConfig) -> Node,
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
