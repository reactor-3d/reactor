use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SamplingParams {
    pub max_samples_per_pixel: u32,
    pub num_samples_per_pixel: u32,
    pub num_bounces: u32,
}

impl Default for SamplingParams {
    fn default() -> Self {
        Self {
            max_samples_per_pixel: 256,
            num_samples_per_pixel: 1,
            num_bounces: 8,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuSamplingParams {
    pub num_samples_per_pixel: u32,
    pub num_bounces: u32,
    pub accumulated_samples_per_pixel: u32,
    pub clear_accumulated_samples: u32,
}
