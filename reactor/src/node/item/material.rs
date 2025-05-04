use egui_snarl::NodeId;
use enum_dispatch::enum_dispatch;
use reactor_derives::EnumAs;
use reactor_types::Vector3;
use serde::{Deserialize, Serialize};
use xrays::scene::TextureData;
use xrays::texture::TextureId;

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

    pub fn to_xrays_material(&self, texture_id: Option<TextureId>, textures: &mut Vec<TextureData>) -> xrays::Material {
        match self {
            MaterialNode::Metal(metal_node) => xrays::Material::Metal {
                albedo: texture_id.unwrap_or_else(|| {
                    let color = metal_node.albedo().to_normalized_gamma_f32();
                    let texture = xrays::Texture::new_from_color(Vector3::new(color[0], color[1], color[2]));
                    textures.push(TextureData::new(texture));
                    textures.len() - 1
                }),
                fuzz: metal_node.fuzz() as _,
            },
            MaterialNode::Dielectric(dielectric_node) => xrays::Material::Dielectric {
                refraction_index: dielectric_node.ior() as _,
            },
            MaterialNode::Lambertian(lambertian_node) => xrays::Material::Lambertian {
                albedo: texture_id.unwrap_or_else(|| {
                    let color = lambertian_node.albedo().to_normalized_gamma_f32();
                    let texture = xrays::Texture::new_from_color(Vector3::new(color[0], color[1], color[2]));
                    textures.push(TextureData::new(texture));
                    textures.len() - 1
                }),
            },
            MaterialNode::Emissive(emissive_node) => xrays::Material::Emissive {
                emit: texture_id.unwrap_or_else(|| {
                    let emit = emissive_node.emit();
                    let texture =
                        xrays::Texture::new_from_color(Vector3::new(emit[0] as _, emit[1] as _, emit[2] as _));
                    textures.push(TextureData::new(texture));
                    textures.len() - 1
                }),
            },
            MaterialNode::Checkerboard(checkerboard_node) => xrays::Material::Checkerboard {
                even: {
                    let color = checkerboard_node.even().to_normalized_gamma_f32();
                    let texture = xrays::Texture::new_from_color(Vector3::new(color[0], color[1], color[2]));
                    textures.push(TextureData::new(texture));
                    textures.len() - 1
                },
                odd: {
                    let color = checkerboard_node.odd().to_normalized_gamma_f32();
                    let texture = xrays::Texture::new_from_color(Vector3::new(color[0], color[1], color[2]));
                    textures.push(TextureData::new(texture));
                    textures.len() - 1
                },
            },
        }
    }
}
