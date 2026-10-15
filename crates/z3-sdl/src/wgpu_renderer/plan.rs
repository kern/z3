//! Explicit CPU descriptions of shader passes, uniforms, and resources.

use std::path::PathBuf;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct ShaderUniforms {
    pub(super) input_size: [f32; 2],
    pub(super) texture_size: [f32; 2],
    pub(super) output_size: [f32; 2],
    pub(super) original_size: [f32; 2],
    pub(super) frame_count: u32,
    pub(super) frame_direction: i32,
    pub(super) pass_index: u32,
    pub(super) _padding: u32,
}

pub(super) const MAX_SHADER_PARAMETERS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum PassScale {
    Default,
    Source(f32),
    Viewport(f32),
    Absolute(f32),
}

#[derive(Debug)]
pub(super) struct ShaderPassPlan {
    pub(super) source: String,
    pub(super) linear: Option<bool>,
    pub(super) scale_x: PassScale,
    pub(super) scale_y: PassScale,
    pub(super) address_mode: wgpu::AddressMode,
    pub(super) mipmap_input: bool,
    pub(super) frame_count_mod: u32,
    pub(super) float_framebuffer: bool,
    pub(super) srgb_framebuffer: bool,
    pub(super) parameters: Vec<ShaderParameter>,
    pub(super) textures: Vec<AuxTexturePlan>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct AuxTexturePlan {
    pub(super) id: String,
    pub(super) filename: Option<PathBuf>,
    pub(super) linear: bool,
    pub(super) mipmap: bool,
    pub(super) address_mode: wgpu::AddressMode,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ShaderParameter {
    pub(super) id: String,
    pub(super) value: f32,
    pub(super) min: f32,
    pub(super) max: f32,
    pub(super) step: f32,
}

pub(super) fn scaled_size(scale: PassScale, source: u32, viewport: u32) -> u32 {
    let value = match scale {
        PassScale::Default => source as f32,
        PassScale::Source(scale) => source as f32 * scale,
        PassScale::Viewport(scale) => viewport as f32 * scale,
        PassScale::Absolute(size) => size,
    };
    value.max(1.0) as u32
}

pub(super) fn referenced_slots(source: &str, prefix: &str, maximum: usize) -> usize {
    (0..maximum)
        .filter(|index| source.contains(&format!("{prefix}{index}")))
        .max()
        .map_or(0, |index| index + 1)
}
