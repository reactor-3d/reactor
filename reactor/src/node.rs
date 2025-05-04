use bitflags::bitflags;
use const_format::concatcp;
use eframe::wgpu::naga::FastIndexSet;
use egui_snarl::{InPin, NodeId, Snarl};
use enum_dispatch::enum_dispatch;
use message::InputMessage;
use reactor_derives::EnumAs;
use serde::{Deserialize, Serialize};

use self::item::material::{CheckerboardNode, DielectricNode, EmissiveNode, LambertianNode, MetalNode};
use self::item::primitive::SphereNode;
use self::item::render::{TriangleRenderNode, XraysRenderNode};
use self::item::{
    CameraNode, CollectionNode, ColorNode, MaterialNode, NumberNode, OutputNode, PrimitiveNode, RenderNode, SceneNode,
    StringNode, TextureNode, VectorNode,
};
use self::message::{CommonNodeMessage, CommonNodeResponse, MessageHandling, SelfNodeMut};
use self::subscribtion::Subscription;
use self::viewer::NodeConfig;

pub mod item;
pub mod message;
pub mod subscribtion;
pub mod viewer;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct NodeFlags: u64 {
        const NUMBER = 0b00000001;
        const STRING = Self::NUMBER.bits() << 1;
        const VECTOR = Self::STRING.bits() << 1;
        const COLOR = Self::VECTOR.bits() << 1;

        const PRIMITIVE_SPHERE = Self::COLOR.bits() << 1;
        const PRIMITIVES = Self::PRIMITIVE_SPHERE.bits();

        const MATERIAL_METAL = Self::PRIMITIVE_SPHERE.bits() << 1;
        const MATERIAL_DIELECTRIC = Self::MATERIAL_METAL.bits() << 1;
        const MATERIAL_LAMBERT = Self::MATERIAL_DIELECTRIC.bits() << 1;
        const MATERIAL_EMISSIVE = Self::MATERIAL_LAMBERT.bits() << 1;
        const MATERIAL_CHECKERBOARD = Self::MATERIAL_EMISSIVE.bits() << 1;
        const MATERIALS = Self::MATERIAL_METAL.bits() | Self::MATERIAL_DIELECTRIC.bits() | Self::MATERIAL_LAMBERT.bits() | Self::MATERIAL_EMISSIVE.bits() | Self::MATERIAL_CHECKERBOARD.bits();

        const TEXTURE = Self::MATERIAL_CHECKERBOARD.bits() << 1;

        const COLLECTION = Self::TEXTURE.bits() << 1;
        const CAMERA = Self::COLLECTION.bits() << 1;

        const SCENE = Self::CAMERA.bits() << 1;

        const RENDER_TRIANGLE = Self::SCENE.bits() << 1;
        const RENDER_XRAYS = Self::RENDER_TRIANGLE.bits() << 1;
        const RENDERS = Self::RENDER_TRIANGLE.bits() | Self::RENDER_XRAYS.bits();

        const OUTPUT = Self::RENDER_XRAYS.bits() << 1;

        const ALL = u64::MAX;
        const TYPICAL_NUMBER_INPUT = NodeFlags::NUMBER.bits();
        const TYPICAL_VECTOR_INPUT = NodeFlags::VECTOR.bits() | NodeFlags::COLOR.bits() | NodeFlags::NUMBER.bits();
    }
}

#[enum_dispatch]
pub trait Noded {
    fn name(&self) -> &str;
    fn inputs(&self) -> &[u64];
    fn outputs(&self) -> &[u64];
    fn reset_input(&mut self, _pin: &InPin) -> bool {
        false
    }
    fn subscription_ref(&self) -> Option<&Subscription> {
        None
    }
    fn subscription_mut(&mut self) -> Option<&mut Subscription> {
        None
    }
}

#[derive(Clone, EnumAs, Serialize, Deserialize)]
#[enum_dispatch(Noded)]
pub enum Node {
    Number(NumberNode),
    String(StringNode),
    Vector(VectorNode),
    Color(ColorNode),
    Primitive(PrimitiveNode),
    Material(MaterialNode),
    Texture(TextureNode),
    Collection(CollectionNode),
    Scene(SceneNode),
    Camera(CameraNode),
    Render(RenderNode),
    Output(OutputNode),
}

impl Node {
    pub fn fabrics() -> impl IntoIterator<Item = (&'static str, fn(&NodeConfig) -> Node, &'static [u64], &'static [u64])>
    {
        [
            (
                NumberNode::NAME,
                (|_| Node::Number(NumberNode::default())) as fn(&NodeConfig) -> Node,
                NumberNode::INPUTS.as_slice(),
                NumberNode::OUTPUTS.as_slice(),
            ),
            (
                StringNode::NAME,
                |_| Node::String(StringNode::default()),
                StringNode::INPUTS.as_slice(),
                StringNode::OUTPUTS.as_slice(),
            ),
            (
                concatcp!(VectorNode::NAME, " 2D"),
                |_| Node::Vector(VectorNode::new_dim2()),
                VectorNode::INPUTS.as_slice(),
                VectorNode::OUTPUTS.as_slice(),
            ),
            (
                concatcp!(VectorNode::NAME, " 3D"),
                |_| Node::Vector(VectorNode::new_dim3()),
                VectorNode::INPUTS.as_slice(),
                VectorNode::OUTPUTS.as_slice(),
            ),
            (
                concatcp!(VectorNode::NAME, " 4D"),
                |_| Node::Vector(VectorNode::new_dim4()),
                VectorNode::INPUTS.as_slice(),
                VectorNode::OUTPUTS.as_slice(),
            ),
            (
                ColorNode::NAME,
                |_| Node::Color(ColorNode::default()),
                ColorNode::INPUTS.as_slice(),
                ColorNode::OUTPUTS.as_slice(),
            ),
            (
                SphereNode::NAME,
                |_| Node::Primitive(PrimitiveNode::Sphere(SphereNode::default())),
                SphereNode::INPUTS.as_slice(),
                SphereNode::OUTPUTS.as_slice(),
            ),
            (
                MetalNode::NAME,
                |_| Node::Material(MaterialNode::Metal(Default::default())),
                MetalNode::INPUTS.as_slice(),
                MetalNode::OUTPUTS.as_slice(),
            ),
            (
                DielectricNode::NAME,
                |_| Node::Material(MaterialNode::Dielectric(Default::default())),
                DielectricNode::INPUTS.as_slice(),
                DielectricNode::OUTPUTS.as_slice(),
            ),
            (
                LambertianNode::NAME,
                |_| Node::Material(MaterialNode::Lambertian(Default::default())),
                LambertianNode::INPUTS.as_slice(),
                LambertianNode::OUTPUTS.as_slice(),
            ),
            (
                EmissiveNode::NAME,
                |_| Node::Material(MaterialNode::Emissive(Default::default())),
                EmissiveNode::INPUTS.as_slice(),
                EmissiveNode::OUTPUTS.as_slice(),
            ),
            (
                CheckerboardNode::NAME,
                |_| Node::Material(MaterialNode::Checkerboard(Default::default())),
                CheckerboardNode::INPUTS.as_slice(),
                CheckerboardNode::OUTPUTS.as_slice(),
            ),
            (
                TextureNode::NAME,
                |_| Node::Texture(TextureNode::default()),
                TextureNode::INPUTS.as_slice(),
                TextureNode::OUTPUTS.as_slice(),
            ),
            (
                CollectionNode::NAME,
                |_| Node::Collection(CollectionNode::default()),
                [CollectionNode::INPUT].as_slice(),
                CollectionNode::OUTPUTS.as_slice(),
            ),
            (
                SceneNode::NAME,
                |_| Node::Scene(SceneNode::default()),
                SceneNode::INPUTS.as_slice(),
                SceneNode::OUTPUTS.as_slice(),
            ),
            (
                CameraNode::NAME,
                |_| Node::Camera(CameraNode::default()),
                CameraNode::INPUTS.as_slice(),
                CameraNode::OUTPUTS.as_slice(),
            ),
            (
                TriangleRenderNode::NAME,
                |_| Node::Render(RenderNode::TriangleRender(TriangleRenderNode::default())),
                TriangleRenderNode::INPUTS.as_slice(),
                TriangleRenderNode::OUTPUTS.as_slice(),
            ),
            (
                XraysRenderNode::NAME,
                |config| {
                    Node::Render(RenderNode::XraysRender(XraysRenderNode::new(
                        config.max_viewport_resolution,
                    )))
                },
                XraysRenderNode::INPUTS.as_slice(),
                XraysRenderNode::OUTPUTS.as_slice(),
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
            Self::Number(_) => NumberNode::handle_msg(self_node, msg),
            Self::String(_) => StringNode::handle_msg(self_node, msg),
            Self::Vector(_) => VectorNode::handle_msg(self_node, msg),
            Self::Color(_) => ColorNode::handle_msg(self_node, msg),
            Self::Primitive(_) => PrimitiveNode::handle_msg(self_node, msg),
            Self::Material(_) => MaterialNode::handle_msg(self_node, msg),
            Self::Texture(_) => TextureNode::handle_msg(self_node, msg),
            Self::Collection(_) => CollectionNode::handle_msg(self_node, msg),
            Self::Scene(_) => SceneNode::handle_msg(self_node, msg),
            Self::Camera(_) => CameraNode::handle_msg(self_node, msg),
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
