use reactor_types::rect::RectSize;
use serde::{Deserialize, Serialize};

use crate::{Angle, Vector3};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    pub eye_pos: Vector3,
    pub eye_dir: Vector3,
    pub up: Vector3,
    /// Angle must be between 0..=90 degrees.
    pub vfov: Angle,
    /// Aperture must be between 0..=1.
    pub aperture: f32,
    /// Focus distance must be a positive number.
    pub focus_distance: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuCamera {
    pub eye: Vector3,
    _padding1: f32,
    pub horizontal: Vector3,
    _padding2: f32,
    pub vertical: Vector3,
    _padding3: f32,
    pub u: Vector3,
    _padding4: f32,
    pub v: Vector3,
    pub lens_radius: f32,
    pub lower_left_corner: Vector3,
    _padding5: f32,
}

impl GpuCamera {
    pub fn new(camera: &Camera, viewport_size: RectSize<u32>) -> Self {
        let lens_radius = 0.5 * camera.aperture;
        let aspect = viewport_size.width as f32 / viewport_size.height as f32;
        let theta = camera.vfov.as_radians();
        let half_height = camera.focus_distance * (0.5 * theta).tan();
        let half_width = aspect * half_height;

        let w = camera.eye_dir.normalize();
        let v = camera.up.normalize();
        let u = w.cross(&v);

        let lower_left_corner = camera.eye_pos + camera.focus_distance * w - half_width * u - half_height * v;
        let horizontal = 2.0 * half_width * u;
        let vertical = 2.0 * half_height * v;

        Self {
            eye: camera.eye_pos,
            _padding1: 0.0,
            horizontal,
            _padding2: 0.0,
            vertical,
            _padding3: 0.0,
            u,
            _padding4: 0.0,
            v,
            lens_radius,
            lower_left_corner,
            _padding5: 0.0,
        }
    }
}
