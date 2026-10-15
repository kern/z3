//! GPU textures, auxiliary PNG uploads, and mipmap resource generation.

use super::{Error, plan::AuxTexturePlan, source::MIPMAP_SHADER};

pub(super) struct SourceTexture {
    pub(super) texture: wgpu::Texture,
    pub(super) view: wgpu::TextureView,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) format: wgpu::TextureFormat,
    pub(super) mip_level_count: u32,
}

pub(super) struct AuxTexture {
    pub(super) image: SourceTexture,
    pub(super) sampler: wgpu::Sampler,
}

/// A pass samples the whole mip chain but renders only into its base level.
/// Keeping these views distinct prevents using a multi-level sampling view
/// as a render attachment, which wgpu rejects.
pub(super) struct PassTarget {
    pub(super) sampled: SourceTexture,
    pub(super) attachment: wgpu::TextureView,
}

pub(super) struct MipmapGenerator {
    layout: wgpu::BindGroupLayout,
    pipelines: Vec<(wgpu::TextureFormat, wgpu::RenderPipeline)>,
}

pub(super) fn ensure_pass_target(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    target: &mut Option<PassTarget>,
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
        target.sampled.width == width
            && target.sampled.height == height
            && target.sampled.format == format
            && target.sampled.mip_level_count == levels
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
    let attachment = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("z3 wgpu pass base-level attachment"),
        base_mip_level: 0,
        mip_level_count: Some(1),
        ..Default::default()
    });
    *target = Some(PassTarget {
        sampled: SourceTexture {
            texture,
            view,
            width,
            height,
            format,
            mip_level_count: levels,
        },
        attachment,
    });
}

impl MipmapGenerator {
    pub(super) fn new(device: &wgpu::Device) -> Self {
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

    pub(super) fn encode(
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

pub(super) fn ensure_history_texture(
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

pub(super) fn load_aux_texture(
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

pub(super) fn downsample_rgba(pixels: &[u8], width: u32, height: u32) -> (Vec<u8>, u32, u32) {
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

pub(super) fn mip_level_count(width: u32, height: u32) -> u32 {
    u32::BITS - width.max(height).max(1).leading_zeros()
}

pub(super) fn texture_pair_layout(
    device: &wgpu::Device,
    label: &str,
    count: usize,
) -> wgpu::BindGroupLayout {
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
