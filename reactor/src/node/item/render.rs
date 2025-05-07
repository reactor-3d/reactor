use eframe::egui_wgpu::RenderState;
use enum_dispatch::enum_dispatch;
use reactor_derives::EnumAs;
use serde::{Deserialize, Serialize};

pub use self::triangle::TriangleRenderNode;
pub use self::xrays::XraysRenderNode;
use crate::node::message::{CommonNodeMessage, CommonNodeResponse, MessageHandling, SelfNodeMut};

pub mod triangle;
pub mod xrays;

#[derive(Clone, EnumAs, Serialize, Deserialize)]
#[enum_dispatch(Noded)]
pub enum RenderNode {
    TriangleRender(TriangleRenderNode),
    XraysRender(XraysRenderNode),
}

impl RenderNode {
    pub fn handle_msg<'a>(self_node: SelfNodeMut<'a>, msg: CommonNodeMessage) -> Option<CommonNodeResponse<'a>> {
        match self_node.node_ref().as_render_ref() {
            Self::TriangleRender(_) => TriangleRenderNode::handle_msg(self_node, msg),
            Self::XraysRender(_) => XraysRenderNode::handle_msg(self_node, msg),
        }
    }

    pub fn register(&mut self, render_state: &RenderState) {
        match self {
            Self::TriangleRender(render) => render.register(render_state),
            Self::XraysRender(render) => render.register(render_state),
        }
    }

    pub fn unregister(&self, render_state: &RenderState) {
        match self {
            Self::TriangleRender(render) => render.unregister(render_state),
            Self::XraysRender(render) => render.unregister(render_state),
        }
    }
}
