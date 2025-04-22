use eframe::egui_wgpu::RenderState;
use serde::{Deserialize, Serialize};

use self::triangle::TriangleRenderNode;
use super::message::{CommonNodeMessage, CommonNodeResponse, MessageHandling, SelfNodeMut};

pub mod triangle;

#[derive(Clone, Serialize, Deserialize)]
pub enum RenderNode {
    Triangle(TriangleRenderNode),
}

impl RenderNode {
    pub const NAME: &str = "Render";

    pub fn name(&self) -> &str {
        match self {
            Self::Triangle(_) => TriangleRenderNode::NAME,
        }
    }

    pub fn inputs(&self) -> &[u64] {
        match self {
            Self::Triangle(render) => render.inputs(),
        }
    }

    pub fn outputs(&self) -> &[u64] {
        match self {
            Self::Triangle(render) => render.outputs(),
        }
    }

    pub fn handle_msg<'a>(self_node: SelfNodeMut<'a>, msg: CommonNodeMessage) -> Option<CommonNodeResponse<'a>> {
        match self_node.as_render_node_ref() {
            Self::Triangle(_) => TriangleRenderNode::handle_msg(self_node, msg),
        }
    }

    pub fn as_triangle_render_mut(&mut self) -> &mut TriangleRenderNode {
        match self {
            Self::Triangle(render) => render,
            node => panic!("Node `{}` is not a `{}`", node.name(), TriangleRenderNode::NAME),
        }
    }

    pub fn register(&self, render_state: &RenderState) {
        match self {
            Self::Triangle(render) => render.register(render_state),
        }
    }

    pub fn unregister(&self, render_state: &RenderState) {
        match self {
            Self::Triangle(render) => render.unregister(render_state),
        }
    }
}
