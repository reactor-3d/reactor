use std::collections::HashMap;
use std::mem;

use bitflags::bitflags;
use eframe::wgpu::naga::FastIndexSet;
use egui::Ui;
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin};
use reactor_types::NodePin;
use serde::{Deserialize, Serialize};
use xrays::scene::{Scene, TextureData};

use crate::node::item::material::InputMaterial;
use crate::node::item::primitive::PrimitiveNode;
use crate::node::message::{
    CommonNodeMessage, CommonNodeResponse, EventMessage, EventResponse, InputMessage, MessageHandling, SelfNodeMut,
};
use crate::node::subscribtion::Event;
use crate::node::viewer::remote;
use crate::node::viewer::ui::{input, output};
use crate::node::{Node, NodeFlags, Noded, collect_for_node};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,Serialize, Deserialize)]
    pub struct SceneDirtyFlags: u32 {
        const NONE = 0;

        const TEXTURE_VALUE = 1;
        const TEXTURE_LAYOUT = Self::TEXTURE_VALUE.bits() << 1;

        const MATERIAL_VALUE = Self::TEXTURE_LAYOUT.bits() << 1;
        const MATERIAL_LAYOUT = Self::MATERIAL_VALUE.bits() << 1;

        const PRIMITIVE_VALUE = Self::MATERIAL_LAYOUT.bits() << 1;
        const PRIMITIVE_LAYOUT = Self::PRIMITIVE_VALUE.bits() << 1;

        const ALL = u32::MAX;
        const INIT = Self::ALL.bits() - 1;
    }
}

impl Default for SceneDirtyFlags {
    fn default() -> Self {
        SceneDirtyFlags::INIT
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SceneNodeResponse {
    Recalculated,
    Nothing,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct SceneNode {
    pub scene_data: NodePin<Option<NodeId>>,

    inner_scene: Scene,

    #[serde(skip)]
    tracked_nodes: FastIndexSet<NodeId>,

    #[serde(skip)]
    dirty: SceneDirtyFlags,
}

impl SceneNode {
    pub const NAME: &str = "Scene";
    pub const INPUTS: [u64; 1] = [NodeFlags::PRIMITIVES.bits() | NodeFlags::COLLECTION.bits()];
    pub const OUTPUTS: [u64; 1] = [NodeFlags::SCENE.bits()];
}

impl Noded for SceneNode {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn inputs(&self) -> &[u64] {
        &Self::INPUTS
    }

    fn outputs(&self) -> &[u64] {
        &Self::OUTPUTS
    }
}

impl MessageHandling for SceneNode {
    fn handle_display_input(mut self_node: SelfNodeMut, pin: &InPin, ui: &mut Ui) -> Option<PinInfo> {
        if pin.id.input == 0 {
            const LABEL: &str = "Scene Data";

            let remote_value = remote::node(pin, LABEL, self_node.snarl, |remote_node| {
                matches!(remote_node, Node::Primitive(_) | Node::Collection(_))
            });

            if let Some(node_id) = remote_value {
                let node = self_node.node_mut().as_scene_mut();
                node.scene_data.set(Some(node_id));
            }

            Some(input::empty_view(ui, LABEL))
        } else {
            None
        }
    }

    fn handle_display_output(_self_node: SelfNodeMut, _pin: &OutPin, _ui: &mut Ui) -> Option<PinInfo> {
        Some(output::empty_view())
    }

    fn handle_input_connect(mut self_node: SelfNodeMut, from: &OutPin, _to: &InPin) {
        let node = self_node.node_mut().as_scene_mut();
        node.scene_data.set(Some(from.id.node));
        node.dirty = SceneDirtyFlags::ALL;
    }

    fn handle_input_disconnect(mut self_node: SelfNodeMut, _from: &OutPin, to: &InPin) {
        let node = self_node.node_mut().as_scene_mut();
        match to.id.input {
            0 => {
                node.scene_data.reset();
                node.dirty = SceneDirtyFlags::ALL;
            },
            _ => (),
        }
    }

    fn handle_input_collect_ids(
        mut self_node: SelfNodeMut,
        predicate: &dyn Fn(&Node) -> bool,
        destination: &mut FastIndexSet<NodeId>,
    ) {
        let node = self_node.node_mut().as_scene_mut();
        collect_for_node(node.scene_data.get(), predicate, destination, self_node.snarl);
    }
}

impl SceneNode {
    pub fn as_scene(&self) -> &Scene {
        &self.inner_scene
    }

    pub fn register_in_render(&mut self) {
        self.dirty = SceneDirtyFlags::ALL;
    }

    pub fn handle_recalculate(mut self_node: SelfNodeMut) -> SceneNodeResponse {
        let old_data = {
            let node = self_node.node_mut().as_scene_mut();
            if node.dirty != SceneDirtyFlags::NONE {
                Some((mem::take(&mut node.inner_scene), mem::take(&mut node.tracked_nodes)))
            } else {
                None
            }
        };

        if let Some((mut old_scene, old_nodes)) = old_data {
            let mut nodes = FastIndexSet::default();
            Self::handle_msg(
                SelfNodeMut::new(self_node.id, self_node.snarl),
                CommonNodeMessage::Input(InputMessage::CollectIds {
                    predicate: &|node| {
                        matches!(
                            node,
                            Node::Primitive(_) | Node::Material(_) | Node::Texture(_) | Node::Collection(_)
                        )
                    },
                    destination: &mut nodes,
                }),
            );

            for node_id in &nodes {
                let has_subscription_response = Node::handle_msg(
                    SelfNodeMut::new(*node_id, self_node.snarl),
                    EventMessage::HasSubscription {
                        node_id: self_node.id,
                        event: Event::OnChange,
                    },
                );

                if let Some(CommonNodeResponse::Event(EventResponse::HasSubscription(false))) =
                    has_subscription_response
                {
                    Node::handle_msg(SelfNodeMut::new(*node_id, self_node.snarl), EventMessage::Subscribe {
                        node_id: self_node.id,
                        event: Event::OnChange,
                        callback: |self_node: SelfNodeMut, subscriber_id: NodeId| match self_node
                            .snarl
                            .get_node_mut(subscriber_id)
                        {
                            Some(Node::Scene(node)) => {
                                node.dirty = SceneDirtyFlags::ALL;
                            },
                            _ => {
                                Node::handle_msg(self_node, EventMessage::Unsubscribe {
                                    node_id: subscriber_id,
                                    event: Event::OnChange,
                                });
                            },
                        },
                    });
                }
            }

            for old_node_id in &old_nodes {
                if !nodes.contains(old_node_id) {
                    Node::handle_msg(
                        SelfNodeMut::new(*old_node_id, self_node.snarl),
                        EventMessage::Unsubscribe {
                            node_id: self_node.id,
                            event: Event::OnChange,
                        },
                    );
                }
            }

            let mut textures: Vec<TextureData> = Vec::new();
            let mut texture_indices = HashMap::new();

            let mut materials = Vec::new();
            let mut material_indices = HashMap::new();

            let mut spheres = Vec::new();

            for node_id in nodes {
                match self_node.node_by_id_ref(node_id) {
                    Node::Texture(texture_node) => {
                        let eq_predicate = |data: &TextureData| {
                            data.key.as_deref() == Some(texture_node.path())
                                && data.scale == texture_node.scale() as f32
                        };

                        if let Some(texture_id) = textures.iter().position(eq_predicate) {
                            texture_indices.insert(node_id, texture_id);
                        } else if let Some(texture_id) = old_scene.textures.iter().position(eq_predicate) {
                            let data = old_scene.textures.remove(texture_id);
                            textures.push(data);
                            texture_indices.insert(node_id, textures.len() - 1);
                        } else {
                            let data =
                                TextureData::load_scaled(texture_node.path().to_string(), texture_node.scale() as _);
                            textures.push(data);
                            texture_indices.insert(node_id, textures.len() - 1);
                        }
                    },
                    Node::Material(material_node) => {
                        let texture_id = material_node
                            .get_texture_node_id()
                            .and_then(|node_id| texture_indices.get(&node_id).copied());
                        let material = material_node.to_xrays_material(texture_id, &mut textures);
                        materials.push(material);
                        material_indices.insert(node_id, materials.len() - 1);
                    },
                    Node::Primitive(PrimitiveNode::Sphere(sphere_node)) => {
                        let material_idx = match sphere_node.material() {
                            InputMaterial::Internal(material_node) => {
                                let texture_id = material_node
                                    .get_texture_node_id()
                                    .and_then(|node_id| texture_indices.get(&node_id).copied());
                                let material = material_node.to_xrays_material(texture_id, &mut textures);
                                materials.push(material);
                                materials.len() - 1
                            },
                            InputMaterial::External(node_id) => material_indices[node_id],
                        };

                        let sphere = sphere_node.to_xrays_sphere(material_idx as u32);
                        spheres.push(sphere);
                    },
                    _ => (),
                }
            }

            let node = self_node.node_mut().as_scene_mut();
            node.inner_scene = Scene {
                spheres,
                materials,
                textures,
            };

            // Самый первый рендер с флагом инициализации не проходит до конца,
            // поэтому нужен будет повторный. В дальнейшем эта ошибка не повторяется.
            if node.dirty == SceneDirtyFlags::INIT {
                node.dirty = SceneDirtyFlags::ALL;
            } else {
                node.dirty = SceneDirtyFlags::NONE;
            }

            SceneNodeResponse::Recalculated
        } else {
            SceneNodeResponse::Nothing
        }
    }
}
