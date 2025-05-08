use reactor_types::rect::RectSize;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wgpu;
use wgpu::util::DeviceExt;
use world::SkyParams;

use crate::buffer::{StorageBuffer, UniformBuffer};
pub use crate::camera::Camera;
use crate::camera::GpuCamera;
use crate::sampling::GpuSamplingParams;
pub use crate::sampling::SamplingParams;
use crate::scene::SceneBuffersGroup;
pub use crate::scene::{Material, Scene, Sphere};
pub use crate::texture::Texture;
use crate::vertex::{Vertex, VertexUniforms};

pub mod buffer;
pub mod camera;
pub mod sampling;
pub mod scene;
pub mod texture;
pub mod vertex;
pub mod world;

pub type Float = f32;
pub type Color = Vector3;
pub type Vector3 = reactor_types::Vector3<Float>;
pub type Vector4 = reactor_types::Vector4<Float>;
pub type Matrix4 = reactor_types::Matrix4<Float>;
pub type Angle = reactor_types::Angle<Float>;

pub struct Renderer {
    vertex_bind_group: wgpu::BindGroup,
    image_bind_group: wgpu::BindGroup,
    parameter_bind_group: wgpu::BindGroup,
    scene_group: SceneBuffersGroup,

    vertex_buffer: wgpu::Buffer,
    frame_data_buffer: UniformBuffer,
    camera_buffer: UniformBuffer,
    sampling_parameter_buffer: UniformBuffer,
    hw_sky_state_buffer: StorageBuffer,

    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    latest_render_params: RenderParams,
    render_progress: RenderProgress,
    frame_number: u32,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        scene: &Scene,
        render_params: &RenderParams,
        max_viewport_resolution: u32,
    ) -> Result<Self, RenderParamsValidationError> {
        render_params.validate()?;

        let uniforms = VertexUniforms {
            view_projection_matrix: unit_quad_projection_matrix(),
            model_matrix: Matrix4::identity(),
        };
        let vertex_uniform_buffer =
            UniformBuffer::new_from_bytes(device, bytemuck::bytes_of(&uniforms), 0, Some("uniforms"));
        let vertex_uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[vertex_uniform_buffer.layout(wgpu::ShaderStages::VERTEX)],
            label: Some("uniforms layout"),
        });
        let vertex_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &vertex_uniform_bind_group_layout,
            entries: &[vertex_uniform_buffer.binding()],
            label: Some("uniforms bind group"),
        });

        let frame_data_buffer = UniformBuffer::new(device, 16_u64, 0, Some("frame data buffer"));

        let image_buffer = {
            let buffer = vec![[0.0_f32; 3]; max_viewport_resolution as usize];
            StorageBuffer::new_from_bytes(device, bytemuck::cast_slice(buffer.as_slice()), 1, Some("image buffer"))
        };

        let image_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                frame_data_buffer.layout(wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT),
                image_buffer.layout(wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT, false),
            ],
            label: Some("image layout"),
        });
        let image_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &image_bind_group_layout,
            entries: &[frame_data_buffer.binding(), image_buffer.binding()],
            label: Some("image bind group"),
        });

        let sampling_parameter_buffer = UniformBuffer::new(
            device,
            std::mem::size_of::<GpuSamplingParams>() as wgpu::BufferAddress,
            0,
            Some("sampling parameter buffer"),
        );

        let camera_buffer = {
            let camera = GpuCamera::new(&render_params.camera, render_params.viewport_size);

            UniformBuffer::new_from_bytes(device, bytemuck::bytes_of(&camera), 1, Some("camera buffer"))
        };

        let hw_sky_state_buffer = {
            let sky_state = render_params.sky.to_sky_state()?;

            StorageBuffer::new_from_bytes(device, bytemuck::bytes_of(&sky_state), 2, Some("sky state buffer"))
        };

        let parameter_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                sampling_parameter_buffer.layout(wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT),
                camera_buffer.layout(wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT),
                hw_sky_state_buffer.layout(wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT, true),
            ],
            label: Some("parameter layout"),
        });

        let parameter_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &parameter_bind_group_layout,
            entries: &[
                sampling_parameter_buffer.binding(),
                camera_buffer.binding(),
                hw_sky_state_buffer.binding(),
            ],
            label: Some("parameter bind group"),
        });

        let scene_group = SceneBuffersGroup::new(scene, device);

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            source: wgpu::ShaderSource::Wgsl(include_str!(concat!(env!("OUT_DIR"), "/compute_shader.wgsl")).into()),
            label: Some("compute_shader.wgsl"),
        });
        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            source: wgpu::ShaderSource::Wgsl(include_str!(concat!(env!("OUT_DIR"), "/render_shader.wgsl")).into()),
            label: Some("render_shader.wgsl"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[
                &vertex_uniform_bind_group_layout,
                &image_bind_group_layout,
                &parameter_bind_group_layout,
                &scene_group.layout(),
            ],
            push_constant_ranges: &[],
            label: Some("raytracer layout"),
        });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("raytracing compute pipeline"),
            layout: Some(&pipeline_layout),
            module: &compute_shader,
            entry_point: Some("cs_main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                polygon_mode: wgpu::PolygonMode::Fill,
                cull_mode: Some(wgpu::Face::Back),
                // Requires Features::DEPTH_CLAMPING
                conservative: false,
                unclipped_depth: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("raytracing render pipeline"),
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(FRAME_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
            label: Some("VertexInput buffer"),
        });

        let render_progress = RenderProgress::new();
        let frame_number = 1;

        Ok(Self {
            vertex_bind_group: vertex_uniform_bind_group,
            frame_data_buffer,
            image_bind_group,
            camera_buffer,
            sampling_parameter_buffer,
            hw_sky_state_buffer,
            parameter_bind_group,
            scene_group,
            vertex_buffer,
            compute_pipeline,
            render_pipeline,
            latest_render_params: *render_params,
            render_progress,
            frame_number,
        })
    }

    pub fn set_render_params(
        &mut self,
        queue: &wgpu::Queue,
        render_force: bool,
        render_params: &RenderParams,
    ) -> Result<(), RenderParamsValidationError> {
        if !render_force && *render_params == self.latest_render_params {
            return Ok(());
        }

        render_params.validate()?;

        {
            let sky_state = render_params.sky.to_sky_state()?;
            queue.write_buffer(self.hw_sky_state_buffer.handle(), 0, bytemuck::bytes_of(&sky_state));
        }

        {
            let camera = GpuCamera::new(&render_params.camera, render_params.viewport_size);
            queue.write_buffer(self.camera_buffer.handle(), 0, bytemuck::bytes_of(&camera));
        }

        self.latest_render_params = *render_params;

        self.render_progress.reset();

        Ok(())
    }

    pub fn progress(&self) -> f32 {
        self.render_progress.accumulated_samples() as f32
            / self.latest_render_params.sampling.max_samples_per_pixel as f32
    }
}

impl Renderer {
    pub fn prepare_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        render_params: &RenderParams,
        scene: Option<&Scene>,
    ) {
        self.set_render_params(queue, scene.is_some(), render_params)
            .expect("Render params should be valid");

        if let Some(scene) = scene {
            self.scene_group.update(&device, &queue, scene);
        }

        let gpu_sampling_params = self.render_progress.next_frame(&self.latest_render_params.sampling);

        queue.write_buffer(
            self.sampling_parameter_buffer.handle(),
            0,
            bytemuck::cast_slice(&[gpu_sampling_params]),
        );

        let frame_number = self.frame_number;
        let frame_data = [
            render_params.viewport_size.width,
            render_params.viewport_size.height,
            frame_number,
        ];
        queue.write_buffer(self.frame_data_buffer.handle(), 0, bytemuck::cast_slice(&frame_data));

        self.frame_number += 1;

        {
            let workgroup_size_x = 8;
            let workgroup_size_y = 8;
            let workgroups_x = (render_params.viewport_size.width + workgroup_size_x - 1) / workgroup_size_x;
            let workgroups_y = (render_params.viewport_size.height + workgroup_size_y - 1) / workgroup_size_y;

            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.vertex_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.image_bind_group, &[]);
            compute_pass.set_bind_group(2, &self.parameter_bind_group, &[]);
            compute_pass.set_bind_group(3, self.scene_group.bind_group(), &[]);
            compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
        }
    }

    pub fn render_frame(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.vertex_bind_group, &[]);
        render_pass.set_bind_group(1, &self.image_bind_group, &[]);
        render_pass.set_bind_group(2, &self.parameter_bind_group, &[]);
        render_pass.set_bind_group(3, self.scene_group.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        let num_vertices = FRAME_VERTICES.len() as u32;
        render_pass.draw(0..num_vertices, 0..1);
    }
}

#[derive(Error, Debug)]
pub enum RenderParamsValidationError {
    #[error("max_samples_per_pixel ({0}) is not a multiple of num_samples_per_pixel ({1})")]
    MaxSampleCountNotMultiple(u32, u32),
    #[error("viewport_size elements cannot be zero: ({0}, {1})")]
    ViewportSize(u32, u32),
    #[error("vfov must be between 0..=90 degrees")]
    VfovOutOfRange(Float),
    #[error("aperture must be between 0..=1")]
    ApertureOutOfRange(Float),
    #[error("focus_distance must be greater than zero")]
    FocusDistanceOutOfRange(Float),
    #[error(transparent)]
    HwSkyModelValidationError(#[from] hw_skymodel::rgb::Error),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderParams {
    pub camera: Camera,
    pub viewport_size: RectSize<u32>,
    pub sky: SkyParams,
    pub sampling: SamplingParams,
}

impl RenderParams {
    fn validate(&self) -> Result<(), RenderParamsValidationError> {
        if self.sampling.max_samples_per_pixel % self.sampling.num_samples_per_pixel != 0 {
            return Err(RenderParamsValidationError::MaxSampleCountNotMultiple(
                self.sampling.max_samples_per_pixel,
                self.sampling.num_samples_per_pixel,
            ));
        }

        if !(Angle::degrees(0.0)..=Angle::degrees(90.0)).contains(&self.camera.vfov) {
            return Err(RenderParamsValidationError::VfovOutOfRange(
                self.camera.vfov.as_degrees(),
            ));
        }

        if !(0.0..=1.0).contains(&self.camera.aperture) {
            return Err(RenderParamsValidationError::ApertureOutOfRange(self.camera.aperture));
        }

        if self.camera.focus_distance < 0.0 {
            return Err(RenderParamsValidationError::FocusDistanceOutOfRange(
                self.camera.focus_distance,
            ));
        }

        if self.viewport_size.width == 0 || self.viewport_size.height == 0 {
            return Err(RenderParamsValidationError::ViewportSize(
                self.viewport_size.width,
                self.viewport_size.height,
            ));
        }

        Ok(())
    }
}

struct RenderProgress {
    accumulated_samples_per_pixel: u32,
}

impl RenderProgress {
    pub fn new() -> Self {
        Self {
            accumulated_samples_per_pixel: 0,
        }
    }

    pub fn next_frame(&mut self, sampling_params: &SamplingParams) -> GpuSamplingParams {
        let current_accumulated_samples = self.accumulated_samples_per_pixel;
        let next_accumulated_samples = sampling_params.num_samples_per_pixel + current_accumulated_samples;

        // Initial state: no samples have been accumulated yet. This is the first frame
        // after a reset. The image buffer's previous samples should be cleared by
        // setting clear_accumulated_samples to 1.
        if current_accumulated_samples == 0 {
            self.accumulated_samples_per_pixel = next_accumulated_samples;
            GpuSamplingParams {
                num_samples_per_pixel: sampling_params.num_samples_per_pixel,
                num_bounces: sampling_params.num_bounces,
                accumulated_samples_per_pixel: next_accumulated_samples,
                clear_accumulated_samples: 1,
            }
        }
        // Progressive render: accumulating samples in the image buffer over multiple
        // frames.
        else if next_accumulated_samples <= sampling_params.max_samples_per_pixel {
            self.accumulated_samples_per_pixel = next_accumulated_samples;
            GpuSamplingParams {
                num_samples_per_pixel: sampling_params.num_samples_per_pixel,
                num_bounces: sampling_params.num_bounces,
                accumulated_samples_per_pixel: next_accumulated_samples,
                clear_accumulated_samples: 0,
            }
        }
        // Completed render: we have accumulated max_samples_per_pixel samples. Stop rendering
        // by setting num_samples_per_pixel to zero.
        else {
            GpuSamplingParams {
                num_samples_per_pixel: 0,
                num_bounces: sampling_params.num_bounces,
                accumulated_samples_per_pixel: current_accumulated_samples,
                clear_accumulated_samples: 0,
            }
        }
    }

    pub fn reset(&mut self) {
        self.accumulated_samples_per_pixel = 0;
    }

    pub fn accumulated_samples(&self) -> u32 {
        self.accumulated_samples_per_pixel
    }
}

fn unit_quad_projection_matrix() -> Matrix4 {
    let sw = 0.5;
    let sh = 0.5;

    // Our ortho camera is just centered at (0, 0)

    let left = -sw;
    let right = sw;
    let bottom = -sh;
    let top = sh;

    // DirectX, Metal, wgpu share the same left-handed coordinate system
    // for their normalized device coordinates:
    // https://github.com/gfx-rs/gfx/tree/master/src/backend/dx12
    ortho_lh_zo(left, right, bottom, top, -1.0, 1.0)
}

/// Creates a matrix for a left hand orthographic-view frustum with a depth range of 0 to 1
///
/// # Parameters
///
/// * `left` - Coordinate for left bound of matrix
/// * `right` - Coordinate for right bound of matrix
/// * `bottom` - Coordinate for bottom bound of matrix
/// * `top` - Coordinate for top bound of matrix
/// * `znear` - Distance from the viewer to the near clipping plane
/// * `zfar` - Distance from the viewer to the far clipping plane
fn ortho_lh_zo(left: f32, right: f32, bottom: f32, top: f32, znear: f32, zfar: f32) -> Matrix4 {
    let one = 1.0;
    let two = 2.0;
    let mut mat = Matrix4::identity();

    mat[(0, 0)] = two / (right - left);
    mat[(0, 3)] = -(right + left) / (right - left);
    mat[(1, 1)] = two / (top - bottom);
    mat[(1, 3)] = -(top + bottom) / (top - bottom);
    mat[(2, 2)] = one / (zfar - znear);
    mat[(2, 3)] = -znear / (zfar - znear);

    mat
}

const FRAME_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, 0.5],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5],
        tex_coords: [1.0, 0.0],
    },
];
