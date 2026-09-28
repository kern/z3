use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use super::{Error, Window};
use wgpu::util::DeviceExt;

const BLIT_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex: u32) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = vec2<f32>(f32((vertex << 1u) & 2u), f32(vertex & 2u));
    out.position = vec4<f32>(out.tex_coords * 2.0 - 1.0, 0.0, 1.0);
    out.tex_coords.y = 1.0 - out.tex_coords.y;
    return out;
}

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(source_texture, source_sampler, input.tex_coords);
}
"#;

const DIM_SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex: u32) -> @builtin(position) vec4<f32> {
    let position = vec2<f32>(f32((vertex << 1u) & 2u), f32(vertex & 2u));
    return vec4<f32>(position * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 159.0 / 255.0);
}
"#;

const MIPMAP_SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((vertex << 1u) & 2u), f32(vertex & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}
@group(0) @binding(0) var source_texture: texture_2d<f32>;
@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let base = vec2<i32>(position.xy) * 2;
    let size = vec2<i32>(textureDimensions(source_texture));
    let p1 = min(base + vec2<i32>(1, 0), size - 1);
    let p2 = min(base + vec2<i32>(0, 1), size - 1);
    let p3 = min(base + vec2<i32>(1, 1), size - 1);
    return (textureLoad(source_texture, base, 0) + textureLoad(source_texture, p1, 0)
          + textureLoad(source_texture, p2, 0) + textureLoad(source_texture, p3, 0)) * 0.25;
}
"#;

/// Cross-platform GPU presentation backend. SDL continues to own the native
/// window while wgpu selects Metal, Vulkan, Direct3D, or GLES as appropriate.
pub struct WgpuRenderer<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    passes: Vec<ShaderPass>,
    dim_pipeline: wgpu::RenderPipeline,
    mipmap_generator: MipmapGenerator,
    bind_group_layout: wgpu::BindGroupLayout,
    history_bind_group_layout: wgpu::BindGroupLayout,
    pass_bind_group_layout: wgpu::BindGroupLayout,
    source: Option<SourceTexture>,
    pass_targets: Vec<Option<SourceTexture>>,
    aux_textures: Vec<AuxTexture>,
    history: Vec<Option<SourceTexture>>,
    history_slots: usize,
    pass_slots: usize,
    pixels: Vec<u8>,
    draw_width: u32,
    draw_height: u32,
    frame_count: u64,
    keep_aspect: bool,
    #[cfg(target_os = "macos")]
    _metal_view: MetalView,
    _window: PhantomData<&'window Window>,
}

struct SourceTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    mip_level_count: u32,
}

struct AuxTexture {
    image: SourceTexture,
    sampler: wgpu::Sampler,
}

struct MipmapGenerator {
    layout: wgpu::BindGroupLayout,
    pipelines: Vec<(wgpu::TextureFormat, wgpu::RenderPipeline)>,
}

struct ShaderPass {
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    scale_x: PassScale,
    scale_y: PassScale,
    mipmap_input: bool,
    frame_count_mod: u32,
    target_format: wgpu::TextureFormat,
    uniforms: wgpu::Buffer,
    parameters: wgpu::Buffer,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderUniforms {
    input_size: [f32; 2],
    texture_size: [f32; 2],
    output_size: [f32; 2],
    original_size: [f32; 2],
    frame_count: u32,
    frame_direction: i32,
    pass_index: u32,
    _padding: u32,
}

const MAX_SHADER_PARAMETERS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq)]
enum PassScale {
    Default,
    Source(f32),
    Viewport(f32),
    Absolute(f32),
}

#[derive(Debug)]
struct ShaderPassPlan {
    source: String,
    linear: Option<bool>,
    scale_x: PassScale,
    scale_y: PassScale,
    address_mode: wgpu::AddressMode,
    mipmap_input: bool,
    frame_count_mod: u32,
    float_framebuffer: bool,
    srgb_framebuffer: bool,
    parameters: Vec<ShaderParameter>,
    textures: Vec<AuxTexturePlan>,
}

#[derive(Clone, Debug, PartialEq)]
struct AuxTexturePlan {
    id: String,
    filename: Option<PathBuf>,
    linear: bool,
    mipmap: bool,
    address_mode: wgpu::AddressMode,
}

#[derive(Clone, Debug, PartialEq)]
struct ShaderParameter {
    id: String,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
}

impl<'window> WgpuRenderer<'window> {
    pub fn new(
        window: &'window Window,
        keep_aspect: bool,
        linear: bool,
        shader_path: Option<&Path>,
    ) -> Result<Self, Error> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        #[cfg(target_os = "macos")]
        let (surface, metal_view) = create_surface(&instance, window)?;
        #[cfg(not(target_os = "macos"))]
        let surface = create_surface(&instance, window)?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .map_err(|error| Error::message(format!("wgpu adapter unavailable: {error}")))?;
        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: adapter.features()
                & (wgpu::Features::FLOAT32_FILTERABLE
                    | wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER),
            ..Default::default()
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&device_descriptor))
            .map_err(|error| Error::message(format!("wgpu device unavailable: {error}")))?;
        let (width, height) = drawable_size(window);
        let config = surface
            .get_default_config(&adapter, width.max(1), height.max(1))
            .ok_or_else(|| Error::message("wgpu surface is unsupported by the selected adapter"))?;
        surface.configure(&device, &config);

        let pass_plans = load_shader_plan(shader_path)?;
        let pass_count = pass_plans.len();
        if pass_plans
            .iter()
            .take(pass_count.saturating_sub(1))
            .any(|pass| pass.float_framebuffer && !pass.srgb_framebuffer)
            && !device
                .features()
                .contains(wgpu::Features::FLOAT32_FILTERABLE)
        {
            return Err(Error::message(
                "this GPU cannot sample the RGBA32Float target requested by float_framebuffer",
            ));
        }
        if pass_plans.iter().any(|pass| {
            pass.address_mode == wgpu::AddressMode::ClampToBorder
                || pass
                    .textures
                    .iter()
                    .any(|texture| texture.address_mode == wgpu::AddressMode::ClampToBorder)
        }) && !device
            .features()
            .contains(wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER)
        {
            return Err(Error::message(
                "this GPU does not support clamp_to_border requested by the shader preset",
            ));
        }
        let texture_plans = pass_plans
            .first()
            .map(|pass| pass.textures.as_slice())
            .unwrap_or_default();
        let history_slots = pass_plans
            .iter()
            .map(|pass| referenced_slots(&pass.source, "prev_texture_", 7))
            .max()
            .unwrap_or(0);
        let pass_slots = pass_plans
            .iter()
            .map(|pass| referenced_slots(&pass.source, "pass_texture_", 20))
            .max()
            .unwrap_or(0);
        if texture_plans.len() > 10 {
            return Err(Error::message(
                "WGSL presets support at most 10 auxiliary textures, matching the C backend",
            ));
        }
        if pass_plans
            .iter()
            .any(|pass| pass.parameters.len() > MAX_SHADER_PARAMETERS)
        {
            return Err(Error::message(format!(
                "WGSL shaders support at most {MAX_SHADER_PARAMETERS} parameters"
            )));
        }
        let sampled_textures = 2 + texture_plans.len() + history_slots + pass_slots;
        if sampled_textures > device.limits().max_sampled_textures_per_shader_stage as usize {
            return Err(Error::message(format!(
                "shader preset needs {sampled_textures} sampled textures, but this GPU supports {}",
                device.limits().max_sampled_textures_per_shader_stage
            )));
        }
        if sampled_textures > device.limits().max_samplers_per_shader_stage as usize {
            return Err(Error::message(format!(
                "shader preset needs {sampled_textures} samplers, but this GPU supports {}",
                device.limits().max_samplers_per_shader_stage
            )));
        }
        let mut layout_entries = vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ];
        for index in 0..texture_plans.len() {
            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: 6 + index as u32 * 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: 7 + index as u32 * 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("z3 wgpu blit bind group layout"),
            entries: &layout_entries,
        });
        let history_bind_group_layout = texture_pair_layout(
            &device,
            "z3 wgpu previous-frame bind group layout",
            history_slots,
        );
        let pass_bind_group_layout = texture_pair_layout(
            &device,
            "z3 wgpu previous-pass bind group layout",
            pass_slots,
        );
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("z3 wgpu blit pipeline layout"),
            bind_group_layouts: &[
                Some(&bind_group_layout),
                Some(&history_bind_group_layout),
                Some(&pass_bind_group_layout),
            ],
            immediate_size: 0,
        });
        let aux_textures = texture_plans
            .iter()
            .map(|plan| load_aux_texture(&device, &queue, plan))
            .collect::<Result<Vec<_>, _>>()?;
        let passes = pass_plans
            .into_iter()
            .enumerate()
            .map(|(index, plan)| {
                // Match the C preset path: ordinary intermediate passes use a
                // linear normalized RGBA target; only the final pass adopts
                // the presentation surface's format/color space.
                let target_format = if index + 1 == pass_count {
                    config.format
                } else if plan.srgb_framebuffer {
                    wgpu::TextureFormat::Rgba8UnormSrgb
                } else if plan.float_framebuffer {
                    wgpu::TextureFormat::Rgba32Float
                } else {
                    wgpu::TextureFormat::Rgba8Unorm
                };
                let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("z3 wgpu shader pass"),
                    source: wgpu::ShaderSource::Wgsl(plan.source.into()),
                });
                let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("z3 wgpu shader pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: target_format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: None,
                });
                let pass_linear = plan.linear.unwrap_or(linear && index + 1 == pass_count);
                let filter = if pass_linear {
                    wgpu::FilterMode::Linear
                } else {
                    wgpu::FilterMode::Nearest
                };
                let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("z3 wgpu shader sampler"),
                    address_mode_u: plan.address_mode,
                    address_mode_v: plan.address_mode,
                    mag_filter: filter,
                    min_filter: filter,
                    mipmap_filter: if pass_linear {
                        wgpu::MipmapFilterMode::Linear
                    } else {
                        wgpu::MipmapFilterMode::Nearest
                    },
                    border_color: (plan.address_mode == wgpu::AddressMode::ClampToBorder)
                        .then_some(wgpu::SamplerBorderColor::TransparentBlack),
                    ..Default::default()
                });
                let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("z3 wgpu shader uniforms"),
                    size: std::mem::size_of::<ShaderUniforms>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                let mut parameter_values = [0.0f32; MAX_SHADER_PARAMETERS];
                for (slot, parameter) in parameter_values.iter_mut().zip(&plan.parameters) {
                    *slot = parameter.value;
                }
                let parameters = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("z3 wgpu shader parameters"),
                    contents: bytemuck::cast_slice(&parameter_values),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
                ShaderPass {
                    pipeline,
                    sampler,
                    scale_x: plan.scale_x,
                    scale_y: plan.scale_y,
                    mipmap_input: plan.mipmap_input,
                    frame_count_mod: plan.frame_count_mod,
                    target_format,
                    uniforms,
                    parameters,
                }
            })
            .collect();
        let dim_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("z3 wgpu pause dim shader"),
            source: wgpu::ShaderSource::Wgsl(DIM_SHADER.into()),
        });
        let dim_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("z3 wgpu pause dim pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let dim_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("z3 wgpu pause dim pipeline"),
            layout: Some(&dim_layout),
            vertex: wgpu::VertexState {
                module: &dim_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &dim_shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let mipmap_generator = MipmapGenerator::new(&device);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            passes,
            dim_pipeline,
            mipmap_generator,
            bind_group_layout,
            history_bind_group_layout,
            pass_bind_group_layout,
            source: None,
            pass_targets: (0..pass_count.saturating_sub(1)).map(|_| None).collect(),
            aux_textures,
            history: (0..history_slots).map(|_| None).collect(),
            history_slots,
            pass_slots,
            pixels: Vec::new(),
            draw_width: 0,
            draw_height: 0,
            frame_count: 0,
            keep_aspect,
            #[cfg(target_os = "macos")]
            _metal_view: metal_view,
            _window: PhantomData,
        })
    }

    pub fn begin_draw(&mut self, width: u32, height: u32) -> usize {
        self.draw_width = width;
        self.draw_height = height;
        let pitch = width as usize * 4;
        self.pixels.resize(pitch * height as usize, 0);
        pitch
    }

    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    pub fn present(&mut self, window: &Window) -> Result<(), Error> {
        self.present_inner(window, false)
    }

    /// Re-presents the most recent game frame under the C host's translucent
    /// black `PauseDimmed` overlay.
    pub fn present_dimmed(&mut self, window: &Window) -> Result<(), Error> {
        // A pause key can arrive before the first emulated frame. There is no
        // previous image to dim in that case, and zero-sized GPU textures are
        // invalid.
        if self.draw_width == 0 || self.draw_height == 0 {
            return Ok(());
        }
        self.present_inner(window, true)
    }

    fn present_inner(&mut self, window: &Window, dimmed: bool) -> Result<(), Error> {
        self.reconfigure_surface(window);
        self.ensure_source_texture();
        {
            let source = self.source.as_ref().expect("source texture was created");
            self.queue.write_texture(
                source.texture.as_image_copy(),
                &self.pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.draw_width * 4),
                    rows_per_image: Some(self.draw_height),
                },
                wgpu::Extent3d {
                    width: self.draw_width,
                    height: self.draw_height,
                    depth_or_array_layers: 1,
                },
            );
            if source.mip_level_count > 1 {
                let mut pixels = self.pixels.clone();
                let mut width = self.draw_width;
                let mut height = self.draw_height;
                for level in 1..source.mip_level_count {
                    let (next, next_width, next_height) = downsample_rgba(&pixels, width, height);
                    self.queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &source.texture,
                            mip_level: level,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        &next,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(next_width * 4),
                            rows_per_image: Some(next_height),
                        },
                        wgpu::Extent3d {
                            width: next_width,
                            height: next_height,
                            depth_or_array_layers: 1,
                        },
                    );
                    pixels = next;
                    width = next_width;
                    height = next_height;
                }
            }
        }

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                match self.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(frame)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
                    wgpu::CurrentSurfaceTexture::Timeout
                    | wgpu::CurrentSurfaceTexture::Occluded => return Ok(()),
                    status => return Err(surface_error(&status)),
                }
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            status => return Err(surface_error(&status)),
        };
        let target = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("z3 wgpu frame encoder"),
            });
        let viewport = self.viewport();
        let viewport_size = (viewport.2.max(1.0) as u32, viewport.3.max(1.0) as u32);
        let mut input_size = (self.draw_width, self.draw_height);
        for index in 0..self.passes.len() {
            let last_pass = index + 1 == self.passes.len();
            let output_size = if last_pass {
                viewport_size
            } else {
                (
                    scaled_size(self.passes[index].scale_x, input_size.0, viewport_size.0),
                    scaled_size(self.passes[index].scale_y, input_size.1, viewport_size.1),
                )
            };
            if !last_pass {
                ensure_pass_target(
                    &self.device,
                    self.passes[index].target_format,
                    &mut self.pass_targets[index],
                    output_size.0,
                    output_size.1,
                    self.passes
                        .get(index + 1)
                        .is_some_and(|pass| pass.mipmap_input),
                );
            }
            let input_view = if index == 0 {
                &self
                    .source
                    .as_ref()
                    .expect("source texture was created")
                    .view
            } else {
                &self.pass_targets[index - 1]
                    .as_ref()
                    .expect("previous shader pass target was created")
                    .view
            };
            let output_view = if last_pass {
                &target
            } else {
                &self.pass_targets[index]
                    .as_ref()
                    .expect("shader pass target was created")
                    .view
            };
            let uniforms = ShaderUniforms {
                input_size: [input_size.0 as f32, input_size.1 as f32],
                texture_size: [input_size.0 as f32, input_size.1 as f32],
                output_size: [output_size.0 as f32, output_size.1 as f32],
                original_size: [self.draw_width as f32, self.draw_height as f32],
                frame_count: if self.passes[index].frame_count_mod == 0 {
                    self.frame_count as u32
                } else {
                    (self.frame_count % u64::from(self.passes[index].frame_count_mod)) as u32
                },
                frame_direction: 1,
                pass_index: index as u32,
                _padding: 0,
            };
            self.queue.write_buffer(
                &self.passes[index].uniforms,
                0,
                bytemuck::bytes_of(&uniforms),
            );
            let original = self.source.as_ref().expect("source texture was created");
            let mut bind_entries = vec![
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.passes[index].sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.passes[index].uniforms.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&original.view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.passes[index].sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: self.passes[index].parameters.as_entire_binding(),
                },
            ];
            for (texture_index, texture) in self.aux_textures.iter().enumerate() {
                bind_entries.push(wgpu::BindGroupEntry {
                    binding: 6 + texture_index as u32 * 2,
                    resource: wgpu::BindingResource::TextureView(&texture.image.view),
                });
                bind_entries.push(wgpu::BindGroupEntry {
                    binding: 7 + texture_index as u32 * 2,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                });
            }
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("z3 wgpu shader bind group"),
                layout: &self.bind_group_layout,
                entries: &bind_entries,
            });
            let mut history_entries = Vec::with_capacity(self.history_slots * 2);
            for history_index in 0..self.history_slots {
                let view = self
                    .frame_count
                    .checked_sub(1 + history_index as u64)
                    .and_then(|frame| self.history[frame as usize % self.history_slots].as_ref())
                    .map(|texture| &texture.view)
                    .unwrap_or(&original.view);
                history_entries.push(wgpu::BindGroupEntry {
                    binding: history_index as u32 * 2,
                    resource: wgpu::BindingResource::TextureView(view),
                });
                history_entries.push(wgpu::BindGroupEntry {
                    binding: history_index as u32 * 2 + 1,
                    resource: wgpu::BindingResource::Sampler(&self.passes[index].sampler),
                });
            }
            let history_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("z3 wgpu previous-frame bind group"),
                layout: &self.history_bind_group_layout,
                entries: &history_entries,
            });
            let mut pass_entries = Vec::with_capacity(self.pass_slots * 2);
            for pass_index in 0..self.pass_slots {
                let view = self
                    .pass_targets
                    .get(pass_index)
                    .and_then(Option::as_ref)
                    .map(|texture| &texture.view)
                    .unwrap_or(&original.view);
                pass_entries.push(wgpu::BindGroupEntry {
                    binding: pass_index as u32 * 2,
                    resource: wgpu::BindingResource::TextureView(view),
                });
                pass_entries.push(wgpu::BindGroupEntry {
                    binding: pass_index as u32 * 2 + 1,
                    resource: wgpu::BindingResource::Sampler(&self.passes[index].sampler),
                });
            }
            let pass_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("z3 wgpu previous-pass bind group"),
                layout: &self.pass_bind_group_layout,
                entries: &pass_entries,
            });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("z3 wgpu shader pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            let (x, y, width, height) = if last_pass {
                viewport
            } else {
                (0.0, 0.0, output_size.0 as f32, output_size.1 as f32)
            };
            pass.set_viewport(x, y, width, height, 0.0, 1.0);
            pass.set_pipeline(&self.passes[index].pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_bind_group(1, &history_bind_group, &[]);
            pass.set_bind_group(2, &pass_bind_group, &[]);
            pass.draw(0..3, 0..1);
            drop(pass);
            if !last_pass {
                let output = self.pass_targets[index]
                    .as_ref()
                    .expect("shader pass target was created");
                if output.mip_level_count > 1 {
                    self.mipmap_generator
                        .encode(&self.device, &mut encoder, output);
                }
            }
            input_size = output_size;
        }
        if dimmed {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("z3 wgpu pause dim pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            // Match the C host: the overlay covers the whole renderer output,
            // including letterbox/pillarbox bars, after the final shader pass.
            pass.set_pipeline(&self.dim_pipeline);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        frame.present();
        if self.history_slots != 0 {
            let slot = self.frame_count as usize % self.history_slots;
            ensure_history_texture(
                &self.device,
                &mut self.history[slot],
                self.draw_width,
                self.draw_height,
            );
            let history = self.history[slot].as_ref().expect("history was created");
            self.queue.write_texture(
                history.texture.as_image_copy(),
                &self.pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.draw_width * 4),
                    rows_per_image: Some(self.draw_height),
                },
                wgpu::Extent3d {
                    width: self.draw_width,
                    height: self.draw_height,
                    depth_or_array_layers: 1,
                },
            );
        }
        self.frame_count = self.frame_count.wrapping_add(1);
        Ok(())
    }

    fn reconfigure_surface(&mut self, window: &Window) {
        let (width, height) = drawable_size(window);
        if width != self.config.width || height != self.config.height {
            self.config.width = width.max(1);
            self.config.height = height.max(1);
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn ensure_source_texture(&mut self) {
        if self.source.as_ref().is_some_and(|source| {
            source.width == self.draw_width && source.height == self.draw_height
        }) {
            return;
        }
        let mip_level_count = if self.passes.first().is_some_and(|pass| pass.mipmap_input) {
            mip_level_count(self.draw_width, self.draw_height)
        } else {
            1
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("z3 wgpu SNES framebuffer"),
            size: wgpu::Extent3d {
                width: self.draw_width,
                height: self.draw_height,
                depth_or_array_layers: 1,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.source = Some(SourceTexture {
            texture,
            view,
            width: self.draw_width,
            height: self.draw_height,
            format: wgpu::TextureFormat::Bgra8Unorm,
            mip_level_count,
        });
    }

    fn viewport(&self) -> (f32, f32, f32, f32) {
        let surface_width = self.config.width as f32;
        let surface_height = self.config.height as f32;
        if !self.keep_aspect {
            return (0.0, 0.0, surface_width, surface_height);
        }
        let source_ratio = self.draw_width as f32 / self.draw_height as f32;
        let surface_ratio = surface_width / surface_height;
        let (width, height) = if surface_ratio > source_ratio {
            (surface_height * source_ratio, surface_height)
        } else {
            (surface_width, surface_width / source_ratio)
        };
        (
            (surface_width - width) * 0.5,
            (surface_height - height) * 0.5,
            width,
            height,
        )
    }
}

fn ensure_pass_target(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    target: &mut Option<SourceTexture>,
    width: u32,
    height: u32,
    mipmapped: bool,
) {
    let levels = if mipmapped {
        mip_level_count(width, height)
    } else {
        1
    };
    if target.as_ref().is_some_and(|target| {
        target.width == width
            && target.height == height
            && target.format == format
            && target.mip_level_count == levels
    }) {
        return;
    }
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("z3 wgpu shader pass target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: levels,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    *target = Some(SourceTexture {
        texture,
        view,
        width,
        height,
        format,
        mip_level_count: levels,
    });
}

impl MipmapGenerator {
    fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("z3 wgpu mipmap bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("z3 wgpu mipmap pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("z3 wgpu mipmap shader"),
            source: wgpu::ShaderSource::Wgsl(MIPMAP_SHADER.into()),
        });
        let pipelines = [
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Rgba32Float,
        ]
        .into_iter()
        .map(|format| {
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("z3 wgpu mipmap pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            });
            (format, pipeline)
        })
        .collect();
        Self { layout, pipelines }
    }

    fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture: &SourceTexture,
    ) {
        let pipeline = self
            .pipelines
            .iter()
            .find(|(format, _)| *format == texture.format)
            .map(|(_, pipeline)| pipeline)
            .expect("mipmap target format has a pipeline");
        for level in 1..texture.mip_level_count {
            let source = texture.texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("z3 wgpu mipmap source view"),
                base_mip_level: level - 1,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let target = texture.texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("z3 wgpu mipmap target view"),
                base_mip_level: level,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("z3 wgpu mipmap bind group"),
                layout: &self.layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&source),
                }],
            });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("z3 wgpu mipmap pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}

fn ensure_history_texture(
    device: &wgpu::Device,
    target: &mut Option<SourceTexture>,
    width: u32,
    height: u32,
) {
    if target
        .as_ref()
        .is_some_and(|target| target.width == width && target.height == height)
    {
        return;
    }
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("z3 wgpu previous frame"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    *target = Some(SourceTexture {
        texture,
        view,
        width,
        height,
        format: wgpu::TextureFormat::Bgra8Unorm,
        mip_level_count: 1,
    });
}

fn load_aux_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    plan: &AuxTexturePlan,
) -> Result<AuxTexture, Error> {
    let path = plan.filename.as_ref().ok_or_else(|| {
        Error::message(format!("WGSL preset texture {} has no filename", plan.id))
    })?;
    let file = std::fs::File::open(path).map_err(|error| {
        Error::message(format!(
            "unable to read texture {}: {error}",
            path.display()
        ))
    })?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|error| {
        Error::message(format!(
            "unable to decode texture {}: {error}",
            path.display()
        ))
    })?;
    let mut decoded = vec![0; reader.output_buffer_size().unwrap_or_default()];
    let info = reader.next_frame(&mut decoded).map_err(|error| {
        Error::message(format!(
            "unable to decode texture {}: {error}",
            path.display()
        ))
    })?;
    let decoded = &decoded[..info.buffer_size()];
    let mut rgba = Vec::with_capacity(info.width as usize * info.height as usize * 4);
    match info.color_type {
        png::ColorType::Rgba => rgba.extend_from_slice(decoded),
        png::ColorType::Rgb => decoded.chunks_exact(3).for_each(|pixel| {
            rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
        }),
        png::ColorType::Grayscale => decoded.iter().for_each(|&value| {
            rgba.extend_from_slice(&[value, value, value, 255]);
        }),
        png::ColorType::GrayscaleAlpha => decoded.chunks_exact(2).for_each(|pixel| {
            rgba.extend_from_slice(&[pixel[0], pixel[0], pixel[0], pixel[1]]);
        }),
        png::ColorType::Indexed => {
            return Err(Error::message(format!(
                "texture {} remained indexed after PNG expansion",
                path.display()
            )));
        }
    }
    let levels = if plan.mipmap {
        mip_level_count(info.width, info.height)
    } else {
        1
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("z3 wgpu preset texture"),
        size: wgpu::Extent3d {
            width: info.width,
            height: info.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: levels,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let mut width = info.width;
    let mut height = info.height;
    let mut pixels = rgba;
    for level in 0..levels {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        if level + 1 < levels {
            let (next, next_width, next_height) = downsample_rgba(&pixels, width, height);
            pixels = next;
            width = next_width;
            height = next_height;
        }
    }
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let filter = if plan.linear {
        wgpu::FilterMode::Linear
    } else {
        wgpu::FilterMode::Nearest
    };
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("z3 wgpu preset texture sampler"),
        address_mode_u: plan.address_mode,
        address_mode_v: plan.address_mode,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: if plan.linear {
            wgpu::MipmapFilterMode::Linear
        } else {
            wgpu::MipmapFilterMode::Nearest
        },
        border_color: (plan.address_mode == wgpu::AddressMode::ClampToBorder)
            .then_some(wgpu::SamplerBorderColor::TransparentBlack),
        ..Default::default()
    });
    Ok(AuxTexture {
        image: SourceTexture {
            texture,
            view,
            width: info.width,
            height: info.height,
            format: wgpu::TextureFormat::Rgba8Unorm,
            mip_level_count: levels,
        },
        sampler,
    })
}

fn downsample_rgba(pixels: &[u8], width: u32, height: u32) -> (Vec<u8>, u32, u32) {
    let next_width = (width / 2).max(1);
    let next_height = (height / 2).max(1);
    let mut result = vec![0; next_width as usize * next_height as usize * 4];
    for y in 0..next_height {
        for x in 0..next_width {
            let mut sum = [0u32; 4];
            let mut count = 0;
            for source_y in (y * 2)..((y * 2 + 2).min(height)) {
                for source_x in (x * 2)..((x * 2 + 2).min(width)) {
                    let offset = (source_y * width + source_x) as usize * 4;
                    for channel in 0..4 {
                        sum[channel] += u32::from(pixels[offset + channel]);
                    }
                    count += 1;
                }
            }
            let offset = (y * next_width + x) as usize * 4;
            for channel in 0..4 {
                result[offset + channel] = (sum[channel] / count) as u8;
            }
        }
    }
    (result, next_width, next_height)
}

fn mip_level_count(width: u32, height: u32) -> u32 {
    u32::BITS - width.max(height).max(1).leading_zeros()
}

fn referenced_slots(source: &str, prefix: &str, maximum: usize) -> usize {
    (0..maximum)
        .filter(|index| source.contains(&format!("{prefix}{index}")))
        .max()
        .map_or(0, |index| index + 1)
}

fn texture_pair_layout(device: &wgpu::Device, label: &str, count: usize) -> wgpu::BindGroupLayout {
    let mut entries = Vec::with_capacity(count * 2);
    for index in 0..count {
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: index as u32 * 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        });
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: index as u32 * 2 + 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
    }
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &entries,
    })
}

fn scaled_size(scale: PassScale, source: u32, viewport: u32) -> u32 {
    let value = match scale {
        PassScale::Default => source as f32,
        PassScale::Source(scale) => source as f32 * scale,
        PassScale::Viewport(scale) => viewport as f32 * scale,
        PassScale::Absolute(size) => size,
    };
    value.max(1.0) as u32
}

fn load_shader_plan(path: Option<&Path>) -> Result<Vec<ShaderPassPlan>, Error> {
    let Some(path) = path else {
        return Ok(vec![default_pass(BLIT_SHADER.into())]);
    };
    let extension = path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();
    if extension.eq_ignore_ascii_case("wgsl") {
        let (source, parameters) = read_shader_source(path)?;
        let mut pass = default_pass(source);
        pass.parameters = parameters;
        return Ok(vec![pass]);
    }
    if extension.eq_ignore_ascii_case("wgslp") {
        return parse_wgsl_preset(path);
    }
    Err(Error::message(format!(
        "the wgpu renderer expects a .wgsl shader or .wgslp preset, not {}",
        path.display()
    )))
}

fn default_pass(source: String) -> ShaderPassPlan {
    ShaderPassPlan {
        source,
        linear: None,
        scale_x: PassScale::Default,
        scale_y: PassScale::Default,
        address_mode: wgpu::AddressMode::ClampToBorder,
        mipmap_input: false,
        frame_count_mod: 0,
        float_framebuffer: false,
        srgb_framebuffer: false,
        parameters: Vec::new(),
        textures: Vec::new(),
    }
}

fn read_shader(path: &Path) -> Result<String, Error> {
    std::fs::read_to_string(path).map_err(|error| {
        Error::message(format!("unable to read shader {}: {error}", path.display()))
    })
}

fn read_shader_source(path: &Path) -> Result<(String, Vec<ShaderParameter>), Error> {
    fn append(
        path: &Path,
        source: &mut String,
        parameters: &mut Vec<ShaderParameter>,
        stack: &mut Vec<PathBuf>,
    ) -> Result<(), Error> {
        let canonical = path.canonicalize().map_err(|error| {
            Error::message(format!("unable to read shader {}: {error}", path.display()))
        })?;
        if stack.contains(&canonical) {
            return Err(Error::message(format!(
                "recursive WGSL include involving {}",
                path.display()
            )));
        }
        stack.push(canonical);
        let text = read_shader(path)?;
        for (line_index, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("#include") {
                let name = rest.trim().trim_matches(['\"', '<', '>']);
                if name.is_empty() {
                    return Err(preset_error(path, line_index + 1, "empty #include"));
                }
                append(
                    &path.parent().unwrap_or_else(|| Path::new("")).join(name),
                    source,
                    parameters,
                    stack,
                )?;
            } else if let Some(rest) = trimmed.strip_prefix("#pragma parameter") {
                let fields = split_quoted_fields(rest);
                if fields.len() < 6 {
                    return Err(preset_error(
                        path,
                        line_index + 1,
                        "#pragma parameter expects ID, label, default, min, max, and step",
                    ));
                }
                let parse = |index: usize| {
                    fields[index].parse::<f32>().map_err(|_| {
                        preset_error(path, line_index + 1, "invalid #pragma parameter number")
                    })
                };
                let parameter = ShaderParameter {
                    id: fields[0].clone(),
                    value: parse(2)?,
                    min: parse(3)?,
                    max: parse(4)?,
                    step: parse(5)?,
                };
                if let Some(existing) = parameters.iter_mut().find(|p| p.id == parameter.id) {
                    *existing = parameter;
                } else {
                    parameters.push(parameter);
                }
            } else {
                source.push_str(line);
                source.push('\n');
            }
        }
        stack.pop();
        Ok(())
    }

    let mut source = String::new();
    let mut parameters = Vec::new();
    append(path, &mut source, &mut parameters, &mut Vec::new())?;
    Ok((source, parameters))
}

fn split_quoted_fields(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for character in text.trim().chars() {
        match character {
            '\"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    result.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

#[derive(Default)]
struct PresetPass {
    filename: Option<PathBuf>,
    linear: Option<bool>,
    scale_type_x: Option<String>,
    scale_type_y: Option<String>,
    scale_x: Option<f32>,
    scale_y: Option<f32>,
    wrap_mode: Option<String>,
    mipmap_input: bool,
    frame_count_mod: u32,
    float_framebuffer: bool,
    srgb_framebuffer: bool,
}

fn parse_wgsl_preset(path: &Path) -> Result<Vec<ShaderPassPlan>, Error> {
    let text = read_shader(path)?;
    let mut pass_count = None;
    let mut values = Vec::<(String, String, usize)>::new();
    for (line_index, raw_line) in text.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(preset_error(path, line_index + 1, "expected key = value"));
        };
        let key = key.trim();
        let value = value.trim().trim_matches('"');
        if key == "shaders" {
            let count = value
                .parse::<usize>()
                .map_err(|_| preset_error(path, line_index + 1, "shaders must be an integer"))?;
            if !(1..=20).contains(&count) {
                return Err(preset_error(
                    path,
                    line_index + 1,
                    "shaders must be between 1 and 20",
                ));
            }
            pass_count = Some(count);
        } else {
            values.push((key.into(), value.into(), line_index + 1));
        }
    }
    let pass_count = pass_count.ok_or_else(|| {
        Error::message(format!(
            "invalid WGSL preset {}: missing shaders",
            path.display()
        ))
    })?;
    let mut passes: Vec<PresetPass> = (0..pass_count).map(|_| PresetPass::default()).collect();
    let mut parameter_ids = Vec::<String>::new();
    let mut parameter_values = std::collections::HashMap::<String, f32>::new();
    let texture_ids = values
        .iter()
        .find(|(key, _, _)| key == "textures")
        .map(|(_, value, _)| {
            value
                .split(';')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut textures = texture_ids
        .iter()
        .map(|id| AuxTexturePlan {
            id: id.clone(),
            filename: None,
            linear: false,
            mipmap: false,
            address_mode: wgpu::AddressMode::ClampToBorder,
        })
        .collect::<Vec<_>>();
    for (key, value, line) in values {
        if key == "parameters" {
            parameter_ids.extend(
                value
                    .split(';')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned),
            );
            continue;
        }
        if key == "textures" {
            continue;
        }
        if let Some((texture, suffix)) = textures.iter_mut().find_map(|texture| {
            key.strip_prefix(&texture.id)
                .map(|suffix| (texture, suffix))
        }) {
            match suffix {
                "" => texture.filename = Some(PathBuf::from(value)),
                "_linear" => texture.linear = parse_preset_bool(path, line, &value)?,
                "_mipmap" => texture.mipmap = parse_preset_bool(path, line, &value)?,
                "_wrap_mode" => texture.address_mode = preset_address_mode(Some(&value), path)?,
                _ => {
                    return Err(preset_error(
                        path,
                        line,
                        &format!("unsupported texture key {key}"),
                    ));
                }
            }
            continue;
        }
        if parameter_ids.iter().any(|id| id == &key) {
            parameter_values.insert(key, parse_any_finite_float(path, line, &value)?);
            continue;
        }
        let Some((field, index)) = split_pass_key(&key) else {
            return Err(preset_error(path, line, &format!("unsupported key {key}")));
        };
        let Some(pass) = passes.get_mut(index) else {
            return Err(preset_error(path, line, "pass index is out of range"));
        };
        match field {
            "shader" => pass.filename = Some(PathBuf::from(value)),
            "filter_linear" => pass.linear = Some(parse_preset_bool(path, line, &value)?),
            "scale_type" => {
                pass.scale_type_x = Some(value.clone());
                pass.scale_type_y = Some(value);
            }
            "scale_type_x" => pass.scale_type_x = Some(value),
            "scale_type_y" => pass.scale_type_y = Some(value),
            "scale" => {
                let scale = parse_preset_float(path, line, &value)?;
                pass.scale_x = Some(scale);
                pass.scale_y = Some(scale);
            }
            "scale_x" => pass.scale_x = Some(parse_preset_float(path, line, &value)?),
            "scale_y" => pass.scale_y = Some(parse_preset_float(path, line, &value)?),
            "wrap_mode" => pass.wrap_mode = Some(value),
            "mipmap_input" => pass.mipmap_input = parse_preset_bool(path, line, &value)?,
            "frame_count_mod" => {
                pass.frame_count_mod = value.parse().map_err(|_| {
                    preset_error(path, line, "frame_count_mod must be a nonnegative integer")
                })?
            }
            "float_framebuffer" => pass.float_framebuffer = parse_preset_bool(path, line, &value)?,
            "srgb_framebuffer" => pass.srgb_framebuffer = parse_preset_bool(path, line, &value)?,
            "alias" => {}
            _ => return Err(preset_error(path, line, &format!("unsupported key {key}"))),
        }
    }
    let base = path.parent().unwrap_or_else(|| Path::new(""));
    passes
        .into_iter()
        .enumerate()
        .map(|(index, pass)| {
            let filename = pass.filename.ok_or_else(|| {
                Error::message(format!(
                    "invalid WGSL preset {}: shader{index} is missing",
                    path.display()
                ))
            })?;
            let shader_path = base.join(filename);
            if !shader_path
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .is_some_and(|extension| extension.eq_ignore_ascii_case("wgsl"))
            {
                return Err(Error::message(format!(
                    "WGSL preset pass {index} expects a .wgsl shader, not {}",
                    shader_path.display()
                )));
            }
            let (source, mut parameters) = read_shader_source(&shader_path)?;
            for parameter in &mut parameters {
                if let Some(value) = parameter_values.get(&parameter.id) {
                    parameter.value = value.clamp(parameter.min, parameter.max);
                } else {
                    parameter.value = parameter.value.clamp(parameter.min, parameter.max);
                }
            }
            Ok(ShaderPassPlan {
                source,
                linear: pass.linear,
                scale_x: preset_scale(pass.scale_type_x.as_deref(), pass.scale_x, path)?,
                scale_y: preset_scale(pass.scale_type_y.as_deref(), pass.scale_y, path)?,
                address_mode: preset_address_mode(pass.wrap_mode.as_deref(), path)?,
                mipmap_input: pass.mipmap_input,
                frame_count_mod: pass.frame_count_mod,
                float_framebuffer: pass.float_framebuffer,
                srgb_framebuffer: pass.srgb_framebuffer,
                parameters,
                textures: textures
                    .iter()
                    .cloned()
                    .map(|mut texture| {
                        texture.filename = texture.filename.map(|filename| base.join(filename));
                        texture
                    })
                    .collect(),
            })
        })
        .collect()
}

fn split_pass_key(key: &str) -> Option<(&str, usize)> {
    let split = key.rfind(|character: char| !character.is_ascii_digit())? + 1;
    if split == key.len() {
        return None;
    }
    Some((&key[..split], key[split..].parse().ok()?))
}

fn parse_preset_bool(path: &Path, line: usize, value: &str) -> Result<bool, Error> {
    if value.eq_ignore_ascii_case("true") || value == "1" {
        Ok(true)
    } else if value.eq_ignore_ascii_case("false") || value == "0" {
        Ok(false)
    } else {
        Err(preset_error(path, line, "expected a boolean"))
    }
}

fn parse_preset_float(path: &Path, line: usize, value: &str) -> Result<f32, Error> {
    value
        .parse::<f32>()
        .ok()
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or_else(|| preset_error(path, line, "expected a positive finite number"))
}

fn parse_any_finite_float(path: &Path, line: usize, value: &str) -> Result<f32, Error> {
    value
        .parse::<f32>()
        .ok()
        .filter(|value| value.is_finite())
        .ok_or_else(|| preset_error(path, line, "expected a finite number"))
}

fn preset_scale(kind: Option<&str>, value: Option<f32>, path: &Path) -> Result<PassScale, Error> {
    let value = value.unwrap_or(1.0);
    match kind.unwrap_or("source").to_ascii_lowercase().as_str() {
        "source" => Ok(PassScale::Source(value)),
        "viewport" => Ok(PassScale::Viewport(value)),
        "absolute" => Ok(PassScale::Absolute(value)),
        "none" => Ok(PassScale::Default),
        kind => Err(Error::message(format!(
            "invalid WGSL preset {}: unsupported scale type {kind}",
            path.display()
        ))),
    }
}

fn preset_address_mode(value: Option<&str>, path: &Path) -> Result<wgpu::AddressMode, Error> {
    match value
        .unwrap_or("clamp_to_border")
        .to_ascii_lowercase()
        .as_str()
    {
        "repeat" => Ok(wgpu::AddressMode::Repeat),
        "mirrored_repeat" => Ok(wgpu::AddressMode::MirrorRepeat),
        "clamp" | "clamp_to_border" => Ok(wgpu::AddressMode::ClampToBorder),
        "clamp_to_edge" => Ok(wgpu::AddressMode::ClampToEdge),
        value => Err(Error::message(format!(
            "invalid WGSL preset {}: unsupported wrap mode {value}",
            path.display()
        ))),
    }
}

fn preset_error(path: &Path, line: usize, message: &str) -> Error {
    Error::message(format!(
        "invalid WGSL preset {}:{line}: {message}",
        path.display()
    ))
}

fn surface_error(status: &wgpu::CurrentSurfaceTexture) -> Error {
    Error::message(format!("wgpu surface error: {status:?}"))
}

fn drawable_size(window: &Window) -> (u32, u32) {
    #[cfg(target_os = "macos")]
    let size = window.metal_drawable_size();
    #[cfg(not(target_os = "macos"))]
    let size = window.size();
    (size.0.max(1) as u32, size.1.max(1) as u32)
}

#[cfg(target_os = "macos")]
struct MetalView(*mut std::ffi::c_void);

#[cfg(target_os = "macos")]
impl Drop for MetalView {
    fn drop(&mut self) {
        // SAFETY: this wrapper uniquely owns the SDL metal view.
        unsafe { super::SDL_Metal_DestroyView(self.0) };
    }
}

#[cfg(target_os = "macos")]
fn create_surface<'window>(
    instance: &wgpu::Instance,
    window: &'window Window,
) -> Result<(wgpu::Surface<'window>, MetalView), Error> {
    // SAFETY: the SDL window is live and remains borrowed for the surface's
    // lifetime. The owned MetalView below retains the CAMetalLayer.
    let view = unsafe { super::SDL_Metal_CreateView(window.0.as_ptr()) };
    if view.is_null() {
        return Err(Error::last());
    }
    let metal_view = MetalView(view);
    // SAFETY: SDL documents this as the live CAMetalLayer backing metal_view.
    let layer = unsafe { super::SDL_Metal_GetLayer(view) };
    if layer.is_null() {
        return Err(Error::message("SDL returned a null CAMetalLayer"));
    }
    // SAFETY: layer remains valid because metal_view is owned by the renderer,
    // and window is borrowed for the returned surface lifetime.
    let surface = unsafe {
        instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer))
    }
    .map_err(|error| Error::message(format!("failed to create wgpu surface: {error}")))?;
    Ok((surface, metal_view))
}

#[cfg(target_os = "windows")]
fn create_surface<'window>(
    instance: &wgpu::Instance,
    window: &'window Window,
) -> Result<wgpu::Surface<'window>, Error> {
    use std::num::NonZeroIsize;

    let info = syswm_info(window)?;
    if info.subsystem != SYSWM_WINDOWS {
        return Err(Error::message("SDL window is not using the Win32 backend"));
    }
    // SAFETY: the active union member is selected by subsystem above.
    let win = unsafe { info.info.windows };
    let hwnd = NonZeroIsize::new(win.window as isize)
        .ok_or_else(|| Error::message("SDL returned a null HWND"))?;
    let mut handle = wgpu::rwh::Win32WindowHandle::new(hwnd);
    handle.hinstance = NonZeroIsize::new(win.hinstance as isize);
    let target = wgpu::SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: Some(wgpu::rwh::WindowsDisplayHandle::new().into()),
        raw_window_handle: handle.into(),
    };
    // SAFETY: SDL owns the HWND for the lifetime represented by window.
    unsafe { instance.create_surface_unsafe(target) }
        .map_err(|error| Error::message(format!("failed to create wgpu surface: {error}")))
}

#[cfg(target_os = "linux")]
fn create_surface<'window>(
    instance: &wgpu::Instance,
    window: &'window Window,
) -> Result<wgpu::Surface<'window>, Error> {
    use std::ptr::NonNull;

    let info = syswm_info(window)?;
    let target = match info.subsystem {
        SYSWM_X11 => {
            // SAFETY: the active union member is selected by subsystem.
            let x11 = unsafe { info.info.x11 };
            let display = NonNull::new(x11.display)
                .ok_or_else(|| Error::message("SDL returned a null X11 display"))?;
            wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(
                    wgpu::rwh::XlibDisplayHandle::new(Some(display), 0).into(),
                ),
                raw_window_handle: wgpu::rwh::XlibWindowHandle::new(x11.window).into(),
            }
        }
        SYSWM_WAYLAND => {
            // SAFETY: the active union member is selected by subsystem.
            let wayland = unsafe { info.info.wayland };
            let display = NonNull::new(wayland.display)
                .ok_or_else(|| Error::message("SDL returned a null Wayland display"))?;
            let surface = NonNull::new(wayland.surface)
                .ok_or_else(|| Error::message("SDL returned a null Wayland surface"))?;
            wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(wgpu::rwh::WaylandDisplayHandle::new(display).into()),
                raw_window_handle: wgpu::rwh::WaylandWindowHandle::new(surface).into(),
            }
        }
        subsystem => {
            return Err(Error::message(format!(
                "unsupported SDL window subsystem {subsystem} for wgpu"
            )));
        }
    };
    // SAFETY: SDL owns the native display/window handles for window's lifetime.
    unsafe { instance.create_surface_unsafe(target) }
        .map_err(|error| Error::message(format!("failed to create wgpu surface: {error}")))
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn create_surface<'window>(
    _instance: &wgpu::Instance,
    _window: &'window Window,
) -> Result<wgpu::Surface<'window>, Error> {
    Err(Error::message("SDL/wgpu is unsupported on this platform"))
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct SdlVersion {
    major: u8,
    minor: u8,
    patch: u8,
}

#[cfg(target_os = "windows")]
const SYSWM_WINDOWS: i32 = 1;
#[cfg(target_os = "linux")]
const SYSWM_X11: i32 = 2;
#[cfg(target_os = "linux")]
const SYSWM_WAYLAND: i32 = 6;

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
struct WindowsInfo {
    window: *mut std::ffi::c_void,
    hdc: *mut std::ffi::c_void,
    hinstance: *mut std::ffi::c_void,
}

#[cfg(target_os = "linux")]
#[repr(C)]
#[derive(Clone, Copy)]
struct X11Info {
    display: *mut std::ffi::c_void,
    window: std::ffi::c_ulong,
}

#[cfg(target_os = "linux")]
#[repr(C)]
#[derive(Clone, Copy)]
struct WaylandInfo {
    display: *mut std::ffi::c_void,
    surface: *mut std::ffi::c_void,
    shell_surface: *mut std::ffi::c_void,
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
#[repr(C)]
#[derive(Clone, Copy)]
union SysWmUnion {
    #[cfg(target_os = "windows")]
    windows: WindowsInfo,
    #[cfg(target_os = "linux")]
    x11: X11Info,
    #[cfg(target_os = "linux")]
    wayland: WaylandInfo,
    padding: [usize; 8],
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
#[repr(C)]
struct SysWmInfo {
    version: SdlVersion,
    subsystem: i32,
    info: SysWmUnion,
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn syswm_info(window: &Window) -> Result<SysWmInfo, Error> {
    let mut info = SysWmInfo {
        version: SdlVersion::default(),
        subsystem: 0,
        info: SysWmUnion { padding: [0; 8] },
    };
    // SAFETY: SDL fills its version structure and then the ABI-compatible
    // SDL_SysWMinfo storage while window remains live.
    unsafe {
        SDL_GetVersion(&mut info.version);
        if SDL_GetWindowWMInfo(window.0.as_ptr(), &mut info) == 0 {
            return Err(Error::last());
        }
    }
    Ok(info)
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
#[link(name = "SDL2")]
unsafe extern "C" {
    fn SDL_GetVersion(version: *mut SdlVersion);
    fn SDL_GetWindowWMInfo(window: *mut std::ffi::c_void, info: *mut SysWmInfo) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_shader_directory(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("z3-wgpu-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn default_shader_exposes_expected_entry_points() {
        assert!(BLIT_SHADER.contains("fn vs_main"));
        assert!(BLIT_SHADER.contains("fn fs_main"));
    }

    #[test]
    fn pause_dim_shader_matches_the_c_overlay_alpha() {
        assert!(DIM_SHADER.contains("159.0 / 255.0"));
    }

    #[test]
    fn legacy_glsl_shader_gets_a_clear_migration_error() {
        let error = load_shader_plan(Some(Path::new("legacy.glsl"))).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("expects a .wgsl shader or .wgslp preset")
        );
    }

    #[test]
    fn wgsl_preset_loads_relative_multipass_shaders_and_c_style_options() {
        let directory = temporary_shader_directory("preset");
        let shader = directory.join("pass.wgsl");
        let preset = directory.join("test.wgslp");
        std::fs::write(&shader, BLIT_SHADER).unwrap();
        std::fs::write(
            &preset,
            r#"
                # Paths are relative to this preset.
                shaders = 2
                shader0 = "pass.wgsl"
                filter_linear0 = false
                scale_type0 = source
                scale0 = 2
                wrap_mode0 = repeat
                shader1 = pass.wgsl
                filter_linear1 = true
                scale_type_x1 = viewport
                scale_x1 = 1
                scale_type_y1 = absolute
                scale_y1 = 224
            "#,
        )
        .unwrap();

        let plan = parse_wgsl_preset(&preset).unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].linear, Some(false));
        assert_eq!(plan[0].scale_x, PassScale::Source(2.0));
        assert_eq!(plan[0].scale_y, PassScale::Source(2.0));
        assert_eq!(plan[0].address_mode, wgpu::AddressMode::Repeat);
        assert_eq!(plan[1].linear, Some(true));
        assert_eq!(plan[1].scale_x, PassScale::Viewport(1.0));
        assert_eq!(plan[1].scale_y, PassScale::Absolute(224.0));
        assert!(plan.iter().all(|pass| pass.source == BLIT_SHADER));

        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn pass_scaling_matches_c_preset_rules() {
        assert_eq!(scaled_size(PassScale::Default, 256, 800), 256);
        assert_eq!(scaled_size(PassScale::Source(2.0), 256, 800), 512);
        assert_eq!(scaled_size(PassScale::Viewport(0.5), 256, 800), 400);
        assert_eq!(scaled_size(PassScale::Absolute(320.0), 256, 800), 320);
    }

    #[test]
    fn wgsl_includes_parameters_textures_and_extended_pass_controls() {
        let directory = temporary_shader_directory("extended-preset");
        let include = directory.join("common.wgsl");
        let shader = directory.join("pass.wgsl");
        let preset = directory.join("test.wgslp");
        std::fs::write(
            &include,
            "#pragma parameter GAIN \"Gain\" 1.0 0.0 2.0 0.1\n",
        )
        .unwrap();
        std::fs::write(&shader, format!("#include \"common.wgsl\"\n{BLIT_SHADER}")).unwrap();
        std::fs::write(
            &preset,
            r#"
                shaders = 1
                shader0 = pass.wgsl
                mipmap_input0 = true
                frame_count_mod0 = 120
                float_framebuffer0 = true
                srgb_framebuffer0 = false
                alias0 = ignored_like_the_c_backend
                parameters = GAIN
                GAIN = 9
                textures = LUT
                LUT = lookup.png
                LUT_linear = true
                LUT_mipmap = true
                LUT_wrap_mode = repeat
            "#,
        )
        .unwrap();

        let plan = parse_wgsl_preset(&preset).unwrap();
        assert!(plan[0].mipmap_input);
        assert_eq!(plan[0].frame_count_mod, 120);
        assert!(plan[0].float_framebuffer);
        assert!(!plan[0].srgb_framebuffer);
        assert_eq!(plan[0].parameters[0].id, "GAIN");
        assert_eq!(plan[0].parameters[0].value, 2.0);
        assert_eq!(plan[0].textures[0].id, "LUT");
        assert_eq!(
            plan[0].textures[0].filename.as_deref(),
            Some(directory.join("lookup.png").as_path())
        );
        assert!(plan[0].textures[0].linear);
        assert!(plan[0].textures[0].mipmap);
        assert_eq!(plan[0].textures[0].address_mode, wgpu::AddressMode::Repeat);

        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn rgba_mipmap_generation_averages_edge_pixels() {
        let pixels = [0, 0, 0, 0, 4, 4, 4, 4, 8, 8, 8, 8, 12, 12, 12, 12];
        let (result, width, height) = downsample_rgba(&pixels, 2, 2);
        assert_eq!((width, height), (1, 1));
        assert_eq!(result, [6, 6, 6, 6]);
    }

    #[test]
    fn resource_slot_detection_only_allocates_referenced_history() {
        let source = "var prev_texture_0: texture_2d<f32>; var prev_texture_3: texture_2d<f32>;";
        assert_eq!(referenced_slots(source, "prev_texture_", 7), 4);
        assert_eq!(referenced_slots(source, "pass_texture_", 20), 0);
    }
}
