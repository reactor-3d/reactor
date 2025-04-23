use eframe::egui_wgpu::RenderState;
use enum_dispatch::enum_dispatch;
use reactor_derives::EnumAs;
use serde::{Deserialize, Serialize};

use self::triangle::TriangleRenderNode;
use crate::node::message::{CommonNodeMessage, CommonNodeResponse, MessageHandling, SelfNodeMut};

pub mod triangle;

#[derive(Clone, EnumAs, Serialize, Deserialize)]
#[enum_dispatch(Noded)]
pub enum RenderNode {
    TriangleRender(TriangleRenderNode),
}

impl RenderNode {
    pub const NAME: &str = "Render";

    pub fn handle_msg<'a>(self_node: SelfNodeMut<'a>, msg: CommonNodeMessage) -> Option<CommonNodeResponse<'a>> {
        match self_node.as_render_node_ref() {
            Self::TriangleRender(_) => TriangleRenderNode::handle_msg(self_node, msg),
        }
    }

    pub fn register(&self, render_state: &RenderState) {
        match self {
            Self::TriangleRender(render) => render.register(render_state),
        }
    }

    pub fn unregister(&self, render_state: &RenderState) {
        match self {
            Self::TriangleRender(render) => render.unregister(render_state),
        }
    }
}
