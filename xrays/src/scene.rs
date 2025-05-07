use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::buffer::StorageBuffer;
use crate::texture::TextureId;
use crate::{Texture, Vector3, Vector4};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextureData {
    pub texture: Texture,
    pub key: Option<Cow<'static, str>>,
    pub scale: f32,
}

impl TextureData {
    pub fn new(texture: Texture) -> Self {
        Self {
            texture,
            key: None,
            scale: 1.0,
        }
    }

    pub fn load_scaled(path: impl Into<Cow<'static, str>>, scale: f32) -> Self {
        let path = path.into();
        let texture = Texture::new_from_scaled_image(&path, scale).expect("Failed to load texture from file");
        Self {
            texture,
            key: Some(path),
            scale,
        }
    }

    pub fn load(path: impl Into<Cow<'static, str>>) -> Self {
        Self::load_scaled(path, 1.0)
    }
}

impl From<Texture> for TextureData {
    fn from(texture: Texture) -> Self {
        Self::new(texture)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub materials: Vec<Material>,
    pub textures: Vec<TextureData>,
}

impl Scene {
    pub fn stub() -> Self {
        let textures = vec![Texture::new_from_color(Vector3::new(0.0, 0.0, 0.0)).into()];
        let materials = vec![Material::Lambertian { albedo: 0 }, Material::Emissive { emit: 0 }];
        let spheres = vec![
            Sphere::new(Vector3::new(0.0, 0.0, 0.0), 0.0, 0),
            Sphere::new(Vector3::new(0.0, 0.0, 0.0), 0.0, 1),
        ];

        Self {
            spheres,
            materials,
            textures,
        }
    }

    pub fn test() -> Self {
        let textures = vec![
            TextureData::new(Texture::new_from_color(Vector3::new(0.5, 0.7, 0.8))),
            TextureData::new(Texture::new_from_color(Vector3::new(0.9, 0.9, 0.9))),
            TextureData::load("assets/moon.jpeg"),
            TextureData::new(Texture::new_from_color(Vector3::new(1.0, 0.85, 0.57))),
            TextureData::load("assets/earthmap.jpeg"),
            TextureData::load_scaled("assets/sun.jpeg", 50.0),
            TextureData::new(Texture::new_from_color(Vector3::new(0.3, 0.9, 0.9))),
            TextureData::new(Texture::new_from_color(Vector3::new(50.0, 0.0, 0.0))),
            TextureData::new(Texture::new_from_color(Vector3::new(0.0, 50.0, 0.0))),
            TextureData::new(Texture::new_from_color(Vector3::new(0.0, 0.0, 50.0))),
        ];

        let materials = vec![
            Material::Checkerboard { even: 0, odd: 1 },
            Material::Lambertian { albedo: 2 },
            Material::Metal { albedo: 3, fuzz: 0.4 },
            Material::Dielectric { refraction_index: 1.5 },
            Material::Lambertian { albedo: 4 },
            Material::Emissive { emit: 5 },
            Material::Lambertian { albedo: 6 },
            Material::Emissive { emit: 7 },
            Material::Emissive { emit: 8 },
            Material::Emissive { emit: 9 },
        ];

        let spheres = vec![
            Sphere::new(Vector3::new(0.0, -500.0, -1.0), 500.0, 0),
            // left row
            Sphere::new(Vector3::new(-5.0, 1.0, -4.0), 1.0, 7),
            Sphere::new(Vector3::new(0.0, 1.0, -4.0), 1.0, 8),
            Sphere::new(Vector3::new(5.0, 1.0, -4.0), 1.0, 9),
            // middle row
            Sphere::new(Vector3::new(-5.0, 1.0, 0.0), 1.0, 2),
            Sphere::new(Vector3::new(0.0, 1.0, 0.0), 1.0, 3),
            Sphere::new(Vector3::new(5.0, 1.0, 0.0), 1.0, 6),
            // right row
            Sphere::new(Vector3::new(-5.0, 0.8, 4.0), 0.8, 1),
            Sphere::new(Vector3::new(0.0, 1.2, 4.0), 1.2, 4),
            Sphere::new(Vector3::new(5.0, 2.0, 4.0), 2.0, 5),
        ];

        Self {
            spheres,
            materials,
            textures,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::NoUninit, Serialize, Deserialize)]
pub struct Sphere {
    // NOTE: naga memory alignment issue, see discussion at
    // https://github.com/gfx-rs/naga/issues/2000
    // It's safer to just use Vec4 instead of Vec3.
    pub center: Vector4,   // 0 byte offset
    pub radius: f32,       // 16 byte offset
    pub material_idx: u32, // 20 byte offset
    _padding: [u32; 2],    // 24 byte offset, 8 bytes size
}

impl Sphere {
    pub fn new(center: Vector3, radius: f64, material_idx: u32) -> Self {
        Self {
            center: Vector4::new(center.x as _, center.y as _, center.z as _, 0.0),
            radius: radius as _,
            material_idx,
            _padding: [0; 2],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Material {
    Lambertian { albedo: TextureId },
    Metal { albedo: TextureId, fuzz: f32 },
    Dielectric { refraction_index: f32 },
    Checkerboard { even: TextureId, odd: TextureId },
    Emissive { emit: TextureId },
}

pub struct GroupData {
    sphere_buffer: StorageBuffer,
    material_buffer: StorageBuffer,
    texture_buffer: StorageBuffer,
    light_buffer: StorageBuffer,
    layout: wgpu::BindGroupLayout,
}

pub struct SceneBuffersGroup {
    data: GroupData,
    bind_group: wgpu::BindGroup,
}

impl GroupData {
    pub fn from_scene(scene: &Scene, device: &wgpu::Device) -> Self {
        let sphere_buffer = StorageBuffer::new_from_bytes(
            device,
            bytemuck::cast_slice(scene.spheres.as_slice()),
            0,
            Some("scene buffer"),
        );

        let mut global_texture_data = Vec::new();
        let mut texture_descriptors = Vec::new();
        let mut material_data = Vec::with_capacity(scene.materials.len());

        for texture in &scene.textures {
            texture_descriptors.push(append_to_global_texture_data(
                &texture.texture,
                &mut global_texture_data,
            ));
        }

        for material in &scene.materials {
            let gpu_material = match material {
                Material::Lambertian { albedo } => GpuMaterial::lambertian(texture_descriptors[*albedo]),
                Material::Metal { albedo, fuzz } => GpuMaterial::metal(texture_descriptors[*albedo], *fuzz),
                Material::Dielectric { refraction_index } => GpuMaterial::dielectric(*refraction_index),
                Material::Checkerboard { odd, even } => {
                    GpuMaterial::checkerboard(texture_descriptors[*odd], texture_descriptors[*even])
                },
                Material::Emissive { emit } => GpuMaterial::emissive(texture_descriptors[*emit]),
            };

            material_data.push(gpu_material);
        }

        let material_buffer = StorageBuffer::new_from_bytes(
            device,
            bytemuck::cast_slice(material_data.as_slice()),
            1,
            Some("materials buffer"),
        );

        let texture_buffer = StorageBuffer::new_from_bytes(
            device,
            bytemuck::cast_slice(global_texture_data.as_slice()),
            2,
            Some("textures buffer"),
        );

        let light_indices: Vec<u32> = scene
            .spheres
            .iter()
            .enumerate()
            .filter(|(_, s)| matches!(scene.materials[s.material_idx as usize], Material::Emissive { .. }))
            .map(|(idx, _)| idx as u32)
            .collect();

        let light_buffer = StorageBuffer::new_from_bytes(
            device,
            bytemuck::cast_slice(light_indices.as_slice()),
            3,
            Some("lights buffer"),
        );

        let scene_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                sphere_buffer.layout(wgpu::ShaderStages::COMPUTE, true),
                material_buffer.layout(wgpu::ShaderStages::COMPUTE, true),
                texture_buffer.layout(wgpu::ShaderStages::COMPUTE, true),
                light_buffer.layout(wgpu::ShaderStages::COMPUTE, true),
            ],
            label: Some("scene layout"),
        });

        Self {
            sphere_buffer,
            material_buffer,
            texture_buffer,
            light_buffer,
            layout: scene_bind_group_layout,
        }
    }

    pub fn create_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.layout,
            entries: &[
                self.sphere_buffer.binding(),
                self.material_buffer.binding(),
                self.texture_buffer.binding(),
                self.light_buffer.binding(),
            ],
            label: Some("scene bind group"),
        })
    }
}

impl SceneBuffersGroup {
    pub fn new(scene: &Scene, device: &wgpu::Device) -> Self {
        let data = GroupData::from_scene(scene, device);
        let scene_bind_group = data.create_bind_group(device);

        Self {
            data,
            bind_group: scene_bind_group,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue, scene: &Scene) {
        // if new_spheres.len() > self.data.sphere_count {
        //     self.data.sphere_count = new_spheres.len();
        // let new_size = (self.capacity * std::mem::size_of::<T>()) as u64;

        // self.data.need_recreate
        self.data = GroupData::from_scene(scene, device);
        // self.data.sphere_buffer =
        //     StorageBuffer::new_from_bytes(device, bytemuck::cast_slice(new_spheres), 0, Some("scene buffer"));

        self.bind_group = self.data.create_bind_group(device);
        // } else {
        //     queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(new_data));
        // }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.data.layout
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuMaterial {
    id: u32,
    desc1: TextureDescriptor,
    desc2: TextureDescriptor,
    x: f32,
}

impl GpuMaterial {
    pub fn lambertian(albedo: TextureDescriptor) -> Self {
        Self {
            id: 0,
            desc1: albedo,
            desc2: TextureDescriptor::empty(),
            x: 0.0,
        }
    }

    pub fn metal(albedo: TextureDescriptor, fuzz: f32) -> Self {
        Self {
            id: 1,
            desc1: albedo,
            desc2: TextureDescriptor::empty(),
            x: fuzz,
        }
    }

    pub fn dielectric(refraction_index: f32) -> Self {
        Self {
            id: 2,
            desc1: TextureDescriptor::empty(),
            desc2: TextureDescriptor::empty(),
            x: refraction_index,
        }
    }

    pub fn checkerboard(even: TextureDescriptor, odd: TextureDescriptor) -> Self {
        Self {
            id: 3,
            desc1: even,
            desc2: odd,
            x: 0.0,
        }
    }

    pub fn emissive(emit: TextureDescriptor) -> Self {
        Self {
            id: 4,
            desc1: emit,
            desc2: TextureDescriptor::empty(),
            x: 0.0,
        }
    }
}

fn append_to_global_texture_data(texture: &Texture, global_texture_data: &mut Vec<[f32; 3]>) -> TextureDescriptor {
    let dimensions = texture.dimensions();
    let offset = global_texture_data.len() as u32;
    global_texture_data.extend_from_slice(texture.as_slice());
    TextureDescriptor {
        width: dimensions.0,
        height: dimensions.1,
        offset,
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextureDescriptor {
    width: u32,
    height: u32,
    offset: u32,
}

impl TextureDescriptor {
    pub fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            offset: 0xffffffff,
        }
    }
}
