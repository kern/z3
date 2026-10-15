mod history;
mod plan;
mod preset;
mod resources;
mod source;
mod surface;

use history::FrameHistory;
#[cfg(target_os = "macos")]
use surface::MetalView;
use surface::{SurfaceOwner, create_surface};

use plan::{MAX_SHADER_PARAMETERS, PassScale, ShaderUniforms, referenced_slots, scaled_size};
use preset::load_shader_plan;
use resources::{
    AuxTexture, MipmapGenerator, PassTarget, SourceTexture, downsample_rgba, ensure_pass_target,
    load_aux_texture, mip_level_count, texture_pair_layout,
};
use source::DIM_SHADER;

use std::{marker::PhantomData, path::Path};

use super::{Error, Window};
use wgpu::util::DeviceExt;

/// Cross-platform GPU presentation backend. SDL continues to own the native
/// window while wgpu selects Metal, Vulkan, Direct3D, or GLES as appropriate.
pub struct WgpuRenderer<'window> {
    // Declared first so its surface drops before the retained Metal view.
    surface: SurfaceOwner<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    passes: Vec<ShaderPass>,
    dim_pipeline: wgpu::RenderPipeline,
    mipmap_generator: MipmapGenerator,
    bind_group_layout: wgpu::BindGroupLayout,
    history_bind_group_layout: wgpu::BindGroupLayout,
    pass_bind_group_layout: wgpu::BindGroupLayout,
    source: Option<SourceTexture>,
    pass_targets: Vec<Option<PassTarget>>,
    aux_textures: Vec<AuxTexture>,
    history: FrameHistory,
    pass_slots: usize,
    pixels: Vec<u8>,
    draw_width: u32,
    draw_height: u32,
    keep_aspect: bool,
    // Keep the SDL layer alive until after the surface and GPU resources drop.
    #[cfg(target_os = "macos")]
    _metal_view: MetalView,
    _window: PhantomData<&'window Window>,
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

impl<'window> WgpuRenderer<'window> {
    pub fn new(
        window: &'window Window,
        keep_aspect: bool,
        linear: bool,
        shader_path: Option<&Path>,
    ) -> Result<Self, Error> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        #[cfg(target_os = "macos")]
        let metal_view;
        #[cfg(target_os = "macos")]
        let surface = {
            let (surface, view) = create_surface(&instance, window)?;
            // Locals drop in reverse declaration order on constructor errors too:
            // the surface must release its layer before SDL destroys the view.
            metal_view = view;
            surface
        };
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
        let surface = SurfaceOwner::configure(surface, &adapter, &device, window)?;

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
                    surface.format()
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
                    format: surface.format(),
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
            passes,
            dim_pipeline,
            mipmap_generator,
            bind_group_layout,
            history_bind_group_layout,
            pass_bind_group_layout,
            source: None,
            pass_targets: (0..pass_count.saturating_sub(1)).map(|_| None).collect(),
            aux_textures,
            history: FrameHistory::new(history_slots),
            pass_slots,
            pixels: Vec::new(),
            draw_width: 0,
            draw_height: 0,
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
        self.surface.reconfigure(&self.device, window);
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

        let Some(frame) = self.surface.acquire(&self.device)? else {
            return Ok(());
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
                    .sampled
                    .view
            };
            let output_view = if last_pass {
                &target
            } else {
                &self.pass_targets[index]
                    .as_ref()
                    .expect("shader pass target was created")
                    .attachment
            };
            let uniforms = ShaderUniforms {
                input_size: [input_size.0 as f32, input_size.1 as f32],
                texture_size: [input_size.0 as f32, input_size.1 as f32],
                output_size: [output_size.0 as f32, output_size.1 as f32],
                original_size: [self.draw_width as f32, self.draw_height as f32],
                frame_count: if self.passes[index].frame_count_mod == 0 {
                    self.history.frame_count() as u32
                } else {
                    (self.history.frame_count() % u64::from(self.passes[index].frame_count_mod))
                        as u32
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
            let history_bind_group = self.history.bind_group(
                &self.device,
                &self.history_bind_group_layout,
                &self.passes[index].sampler,
                &original.view,
            );
            let mut pass_entries = Vec::with_capacity(self.pass_slots * 2);
            for pass_index in 0..self.pass_slots {
                let view = self
                    .pass_targets
                    .get(pass_index)
                    .and_then(Option::as_ref)
                    .map(|target| &target.sampled.view)
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
                if output.sampled.mip_level_count > 1 {
                    self.mipmap_generator
                        .encode(&self.device, &mut encoder, &output.sampled);
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
        self.history.record_presented(
            &self.device,
            &self.queue,
            (self.draw_width, self.draw_height),
            &self.pixels,
        );
        Ok(())
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
        self.surface
            .viewport((self.draw_width, self.draw_height), self.keep_aspect)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod gpu_tests;
