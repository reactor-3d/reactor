use egui_snarl::NodeId;
use enum_dispatch::enum_dispatch;
use reactor_derives::EnumAs;
use serde::{Deserialize, Serialize};

pub use self::checkerboard::CheckerboardNode;
pub use self::dielectric::DielectricNode;
pub use self::emissive::EmissiveNode;
pub use self::lambertian::LambertianNode;
pub use self::metal::MetalNode;
use crate::node::message::{CommonNodeMessage, CommonNodeResponse, MessageHandling, SelfNodeMut};

pub mod checkerboard;
pub mod dielectric;
pub mod emissive;
pub mod lambertian;
pub mod metal;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum InputMaterial {
    Internal(MaterialNode),
    External(NodeId),
}

impl Default for InputMaterial {
    fn default() -> Self {
        Self::Internal(MaterialNode::default())
    }
}

#[derive(Clone, EnumAs, Serialize, Deserialize, PartialEq)]
#[enum_dispatch(Noded)]
pub enum MaterialNode {
    Metal(MetalNode),
    Dielectric(DielectricNode),
    Lambertian(LambertianNode),
    Emissive(EmissiveNode),
    Checkerboard(CheckerboardNode),
}

impl Default for MaterialNode {
    fn default() -> Self {
        Self::Lambertian(Default::default())
    }
}

impl MaterialNode {
    pub fn handle_msg<'a>(self_node: SelfNodeMut<'a>, msg: CommonNodeMessage) -> Option<CommonNodeResponse<'a>> {
        match self_node.node_ref().as_material_ref() {
            Self::Metal(_) => MetalNode::handle_msg(self_node, msg),
            Self::Dielectric(_) => DielectricNode::handle_msg(self_node, msg),
            Self::Lambertian(_) => LambertianNode::handle_msg(self_node, msg),
            Self::Emissive(_) => EmissiveNode::handle_msg(self_node, msg),
            Self::Checkerboard(_) => CheckerboardNode::handle_msg(self_node, msg),
        }
    }

    pub fn get_texture_node_id(&self) -> Option<NodeId> {
        match self {
            Self::Metal(metal) => metal.texture(),
            Self::Dielectric(_) => None,
            Self::Lambertian(lambert) => lambert.texture(),
            Self::Emissive(emissive) => emissive.texture(),
            Self::Checkerboard(_) => None,
        }
    }
}
