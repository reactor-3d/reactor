use eframe::egui_wgpu::{Callback, CallbackResources, CallbackTrait, RenderState, ScreenDescriptor};
use eframe::wgpu;
use egui::{PaintCallbackInfo, Ui};
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin, Snarl};
use reactor_types::NodePin;
use reactor_types::rect::RectSize;
use serde::{Deserialize, Serialize};
use xrays::scene::Scene;
use xrays::{RenderParams, SamplingParams};

use crate::node::item::camera::{CameraNode, camera_node_by_id};
use crate::node::item::scene::{SceneNode, SceneNodeResponse};
use crate::node::message::{MessageHandling, SelfNodeMut};
use crate::node::viewer::remote;
use crate::node::viewer::ui::{input, output};
use crate::node::{Node, NodeFlags, Noded, collect_for_node};

#[derive(Clone, Serialize, Deserialize)]
pub struct XraysRenderNode {
    max_samples_per_pixel: NodePin<u32>,
    num_samples_per_pixel: NodePin<u32>,
    num_bounces: NodePin<u32>,
    camera: NodePin<Option<NodeId>>,
    scene: Option<NodeId>,

    max_viewport_resolution: u32,
    #[serde(skip)]
    disconnect_scene: bool,
}

impl XraysRenderNode {
    pub fn new(max_viewport_resolution: u32) -> Self {
        let sampling = SamplingParams::default();
        Self {
            max_samples_per_pixel: NodePin::new(sampling.max_samples_per_pixel),
            num_samples_per_pixel: NodePin::new(sampling.num_samples_per_pixel),
            num_bounces: NodePin::new(sampling.num_bounces),
            camera: Default::default(),
            scene: Default::default(),

            max_viewport_resolution,
            disconnect_scene: false,
        }
    }

    pub fn camera_id(&self) -> Option<NodeId> {
        self.camera.get()
    }

    pub fn camera_node<'a>(&self, snarl: &'a Snarl<Node>) -> Option<&'a CameraNode> {
        self.camera
            .get()
            .and_then(|camera_id| camera_node_by_id(camera_id, snarl))
    }

    fn sampling_params(&self) -> SamplingParams {
        SamplingParams {
            max_samples_per_pixel: self.max_samples_per_pixel.get(),
            num_samples_per_pixel: self.num_samples_per_pixel.get(),
            num_bounces: self.num_bounces.get(),
        }
    }
}

impl XraysRenderNode {
    pub const NAME: &str = "Xrays Render";
    pub const INPUTS: [u64; 5] = [
        NodeFlags::TYPICAL_NUMBER_INPUT.bits(),
        NodeFlags::TYPICAL_NUMBER_INPUT.bits(),
        NodeFlags::TYPICAL_NUMBER_INPUT.bits(),
        NodeFlags::CAMERA.bits(),
        NodeFlags::SCENE.bits(),
    ];
    pub const OUTPUTS: [u64; 1] = [NodeFlags::RENDER_XRAYS.bits()];

    pub fn register(&self, render_state: &RenderState) {
        RaytracerRenderResources::register(render_state, self, Default::default());
    }

    pub fn unregister(&self, render_state: &RenderState) {
        RaytracerRenderResources::unregister(render_state);
    }

    pub fn draw(self_node: SelfNodeMut, viewport: egui::Rect, painter: &egui::Painter) {
        let node = self_node.node_ref().as_render_ref().as_xrays_render_ref();
        let render_params = node.camera_node(self_node.snarl).map(|camera_node| RenderParams {
            camera: camera_node.to_xrays_camera(),
            sky: Default::default(),
            sampling: node.sampling_params(),
        });

        let scene = if let Some(scene_node_id) = node.scene {
            if let SceneNodeResponse::Recalculated =
                SceneNode::handle_recalculate(SelfNodeMut::new(scene_node_id, self_node.snarl))
            {
                Some(self_node.snarl[scene_node_id].as_scene_ref().as_scene().clone())
            } else {
                None
            }
        } else if node.disconnect_scene {
            self_node.snarl[self_node.id]
                .as_render_mut()
                .as_xrays_render_mut()
                .disconnect_scene = false;
            Some(Scene::stub())
        } else {
            None
        };

        if let Some(render_params) = render_params {
            let callback = Callback::new_paint_callback(viewport, Drawer { render_params, scene });
            painter.add(callback);
        }
    }
}

impl Noded for XraysRenderNode {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn inputs(&self) -> &[u64] {
        &Self::INPUTS
    }

    fn outputs(&self) -> &[u64] {
        &Self::OUTPUTS
    }

    fn reset_input(&mut self, pin: &InPin) -> bool {
        match pin.id.input {
            0 => self.max_samples_per_pixel.reset(),
            1 => self.num_samples_per_pixel.reset(),
            2 => self.num_bounces.reset(),
            3 => self.camera.reset(),
            4 => {
                self.scene = None;
                self.disconnect_scene = true
            },
            _ => return false,
        }
        true
    }
}

impl MessageHandling for XraysRenderNode {
    fn handle_display_input(mut self_node: SelfNodeMut, pin: &InPin, ui: &mut Ui) -> Option<PinInfo> {
        match pin.id.input {
            0 => Some(input::display_number_field(
                ui,
                pin,
                self_node,
                "Total samples per pixel",
                |node| &mut node.as_render_mut().as_xrays_render_mut().max_samples_per_pixel,
            )),
            1 => Some(input::display_number_field(
                ui,
                pin,
                self_node,
                "Samples per pixel",
                |node| &mut node.as_render_mut().as_xrays_render_mut().num_samples_per_pixel,
            )),
            2 => Some(input::display_number_field(
                ui,
                pin,
                self_node,
                "Bounces per ray",
                |node| &mut node.as_render_mut().as_xrays_render_mut().num_bounces,
            )),
            3 => Some(input::display_node_field(
                ui,
                pin,
                self_node,
                "Camera",
                |remote_node| matches!(remote_node, Node::Camera(_)),
                |node| &mut node.as_render_mut().as_xrays_render_mut().camera,
            )),
            4 => Some({
                const LABEL: &str = "Scene";

                let remote_value = remote::node(pin, LABEL, self_node.snarl, |remote_node| {
                    matches!(remote_node, Node::Scene(_))
                });
                let node = self_node.node_mut().as_render_mut().as_xrays_render_mut();

                let old_value = node.scene;
                node.scene = remote_value;

                if old_value != node.scene {
                    if let Some(scene_id) = node.scene {
                        self_node.node_by_id_mut(scene_id).as_scene_mut().register_in_render();
                    }
                }
                input::empty_view(ui, LABEL)
            }),
            _ => None,
        }
    }

    fn handle_display_output(_self_node: SelfNodeMut, _pin: &OutPin, _ui: &mut Ui) -> Option<PinInfo> {
        Some(output::empty_view())
    }

    fn handle_input_collect_ids(
        self_node: SelfNodeMut,
        predicate: &dyn Fn(&Node) -> bool,
        destination: &mut wgpu::naga::FastIndexSet<NodeId>,
    ) {
        let camera_node_id = self_node.node_ref().as_render_ref().as_xrays_render_ref().camera.get();
        let scene_node_id = self_node.node_ref().as_render_ref().as_xrays_render_ref().scene;

        collect_for_node(camera_node_id, predicate, destination, self_node.snarl);
        collect_for_node(scene_node_id, predicate, destination, self_node.snarl);
    }
}

struct Drawer {
    render_params: RenderParams,
    scene: Option<Scene>,
}

impl CallbackTrait for Drawer {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(resources) = callback_resources.get_mut::<RaytracerRenderResources>() {
            let viewport_size = RectSize {
                width: screen_descriptor.size_in_pixels[0],
                height: screen_descriptor.size_in_pixels[1],
            };
            resources.prepare(device, queue, &self.render_params, self.scene.as_ref(), viewport_size);
        }
        Vec::new()
    }

    fn paint(
        &self,
        _info: PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &CallbackResources,
    ) {
        if let Some(resources) = callback_resources.get::<RaytracerRenderResources>() {
            resources.paint(render_pass);
        }
    }
}

pub struct RaytracerRenderResources {
    renderer: xrays::Renderer,
}

impl RaytracerRenderResources {
    pub fn new(
        render_state: &RenderState,
        render_params: &RenderParams,
        viewport_size: RectSize<u32>,
        max_viewport_resolution: u32,
    ) -> Self {
        let device = &render_state.device;
        let target_format = render_state.target_format;
        let scene = Scene::stub();

        Self {
            renderer: xrays::Renderer::new(
                device,
                target_format,
                &scene,
                render_params,
                viewport_size,
                max_viewport_resolution,
            )
            .expect("Xrays renderer creation failed"),
        }
    }

    pub fn register(render_state: &RenderState, node: &XraysRenderNode, viewport_size: RectSize<u32>) {
        let render_params = RenderParams {
            camera: Default::default(),
            sky: Default::default(),
            sampling: node.sampling_params(),
        };

        render_state.renderer.write().callback_resources.insert(Self::new(
            render_state,
            &render_params,
            viewport_size,
            node.max_viewport_resolution,
        ));
    }

    pub fn unregister(render_state: &RenderState) {
        render_state.renderer.write().callback_resources.remove::<Self>();
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_params: &RenderParams,
        scene: Option<&Scene>,
        viewport_size: RectSize<u32>,
    ) {
        self.renderer
            .prepare_frame(device, queue, render_params, scene, viewport_size);
    }

    pub fn paint(&self, rpass: &mut wgpu::RenderPass<'static>) {
        self.renderer.render_frame(rpass);
    }
}
