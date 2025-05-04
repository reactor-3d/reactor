use enum_dispatch::enum_dispatch;
use reactor_derives::EnumAs;
use serde::{Deserialize, Serialize};

use self::sphere::SphereNode;
use crate::node::message::{CommonNodeMessage, CommonNodeResponse, MessageHandling, SelfNodeMut};

pub mod sphere;

#[derive(Clone, EnumAs, Serialize, Deserialize)]
#[enum_dispatch(Noded)]
pub enum PrimitiveNode {
    Sphere(SphereNode),
}

impl PrimitiveNode {
    pub fn handle_msg<'a>(self_node: SelfNodeMut<'a>, msg: CommonNodeMessage) -> Option<CommonNodeResponse<'a>> {
        match self_node.node_ref().as_primitive_ref() {
            Self::Sphere(_) => SphereNode::handle_msg(self_node, msg),
        }
    }
}
