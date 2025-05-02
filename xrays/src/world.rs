use std::f32::consts;

use serde::{Deserialize, Serialize};

use crate::{Angle, Float};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkyParams {
    // Azimuth must be between 0..=360 degrees
    pub azimuth: Angle,
    // Inclination must be between 0..=90 degrees
    pub zenith: Angle,
    // Turbidity must be between 1..=10
    pub turbidity: Float,
    // Albedo elements must be between 0..=1
    pub albedo: [Float; 3],
}

impl Default for SkyParams {
    fn default() -> Self {
        Self {
            azimuth: Angle::degrees(0.0),
            zenith: Angle::degrees(85.0),
            turbidity: 4.0,
            albedo: [1.0; 3],
        }
    }
}

impl SkyParams {
    pub fn to_sky_state(self: &SkyParams) -> Result<GpuSkyState, hw_skymodel::rgb::Error> {
        let azimuth = self.azimuth.as_radians();
        let zenith = self.zenith.as_radians();
        let sun_direction = [
            zenith.sin() * azimuth.cos(),
            zenith.cos(),
            zenith.sin() * azimuth.sin(),
            0.0,
        ];

        let state = hw_skymodel::rgb::SkyState::new(&hw_skymodel::rgb::SkyParams {
            elevation: consts::FRAC_PI_2 - zenith,
            turbidity: self.turbidity,
            albedo: self.albedo,
        })?;

        let (params_data, radiance_data) = state.raw();

        Ok(GpuSkyState {
            params: params_data,
            radiances: radiance_data,
            _padding: [0, 2],
            sun_direction,
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuSkyState {
    pub params: [f32; 27],       // 0 byte offset, 108 byte size
    pub radiances: [f32; 3],     // 108 byte offset, 12 byte size
    _padding: [u32; 2],          // 120 byte offset, 8 byte size
    pub sun_direction: [f32; 4], // 128 byte offset, 16 byte size
}
