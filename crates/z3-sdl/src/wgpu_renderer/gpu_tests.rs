//! Opt-in presentation readback using synthetic pixels and a headless adapter.
//! This covers the extracted resource path, not SDL surfaces or every preset.

use wgpu::util::DeviceExt;

use super::{
    plan::{MAX_SHADER_PARAMETERS, ShaderUniforms},
    resources::{MipmapGenerator, ensure_history_texture, ensure_pass_target, texture_pair_layout},
    source::BLIT_SHADER,
};

const WIDTH: u32 = 8;
const HEIGHT: u32 = 4;

fn generated_rgba(frame: u8) -> Vec<u8> {
    (0..HEIGHT as u8)
        .flat_map(|y| {
            (0..WIDTH as u8).flat_map(move |x| {
                [
                    x * 31 + frame * 7,
                    y * 53 + frame * 11,
                    (x + y) * 17 + frame * 13,
                    255 - (x * 7 + y * 3) % 64,
                ]
            })
        })
        .collect()
}

fn pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    source: &str,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("synthetic readback shader"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("synthetic readback pipeline"),
        layout: Some(layout),
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
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn draw(
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    pipeline: &wgpu::RenderPipeline,
    groups: [&wgpu::BindGroup; 3],
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("synthetic readback pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
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
    for (index, group) in groups.into_iter().enumerate() {
        pass.set_bind_group(index as u32, group, &[]);
    }
    pass.draw(0..3, 0..1);
}

fn copy_readback(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("synthetic readback pixels"),
        size: u64::from(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT * HEIGHT),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
                rows_per_image: Some(HEIGHT),
            },
        },
        wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
    );
    buffer
}

fn read_rgba(device: &wgpu::Device, buffer: &wgpu::Buffer) -> Vec<u8> {
    let slice = buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap()
    });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    receiver.recv().unwrap().unwrap();
    let mapped = slice.get_mapped_range();
    let pixels = mapped
        .chunks_exact(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize)
        .flat_map(|row| row[..WIDTH as usize * 4].iter().copied())
        .collect();
    drop(mapped);
    buffer.unmap();
    pixels
}

#[test]
#[ignore = "requires a working GPU adapter; run explicitly for presentation readback"]
fn gpu_blit_and_multipass_history_readback_matches_generated_pixels() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .expect("GPU readback requires an available adapter");
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();

    // Match the renderer's group-0 binding contract without changing its
    // interactive construction code. Groups 1 and 2 use its actual helper.
    let input_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("synthetic shader input layout"),
        entries: &(0..6)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: if binding == 2 || binding == 5 {
                    wgpu::ShaderStages::VERTEX_FRAGMENT
                } else {
                    wgpu::ShaderStages::FRAGMENT
                },
                ty: match binding {
                    0 | 3 => wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    1 | 4 => wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    _ => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                count: None,
            })
            .collect::<Vec<_>>(),
    });
    let history_layout = texture_pair_layout(&device, "synthetic history layout", 1);
    let pass_layout = texture_pair_layout(&device, "synthetic previous-pass layout", 1);
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("synthetic shader pipeline layout"),
        bind_group_layouts: &[
            Some(&input_layout),
            Some(&history_layout),
            Some(&pass_layout),
        ],
        immediate_size: 0,
    });
    let blit = pipeline(&device, &layout, BLIT_SHADER);
    let history_source = BLIT_SHADER.replace(
        "return textureSample(source_texture, source_sampler, input.tex_coords);",
        concat!(
            "let current = textureSample(pass_texture_0, pass_sampler_0, input.tex_coords);\n",
            "let previous = textureSample(prev_texture_0, prev_sampler_0, input.tex_coords);\n",
            "return select(current, previous, input.tex_coords.x < parameters.values[0].x ",
            "&& uniforms.frame_count == 7u);"
        ),
    ) + r#"
struct Uniforms {
    input_size: vec2<f32>, texture_size: vec2<f32>,
    output_size: vec2<f32>, original_size: vec2<f32>,
    frame_count: u32, frame_direction: i32, pass_index: u32, padding: u32,
}
struct Parameters { values: array<vec4<f32>, 64>, }
@group(0) @binding(2) var<uniform> uniforms: Uniforms;
@group(0) @binding(5) var<uniform> parameters: Parameters;
@group(1) @binding(0) var prev_texture_0: texture_2d<f32>;
@group(1) @binding(1) var prev_sampler_0: sampler;
@group(2) @binding(0) var pass_texture_0: texture_2d<f32>;
@group(2) @binding(1) var pass_sampler_0: sampler;
"#;
    let history_pass = pipeline(&device, &layout, &history_source);
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
    let uniforms = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("synthetic pass uniforms"),
        contents: bytemuck::bytes_of(&ShaderUniforms {
            input_size: [WIDTH as f32, HEIGHT as f32],
            texture_size: [WIDTH as f32, HEIGHT as f32],
            output_size: [WIDTH as f32, HEIGHT as f32],
            original_size: [WIDTH as f32, HEIGHT as f32],
            frame_count: 7,
            frame_direction: 1,
            pass_index: 1,
            _padding: 0,
        }),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let mut parameter_values = [0.0f32; MAX_SHADER_PARAMETERS];
    parameter_values[0] = 0.5;
    let parameters = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("synthetic shader parameters"),
        contents: bytemuck::cast_slice(&parameter_values),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let current_rgba = generated_rgba(1);
    let previous_rgba = generated_rgba(0);
    let mut original = None;
    let mut history = None;
    for (target, pixels) in [
        (&mut original, &current_rgba),
        (&mut history, &previous_rgba),
    ] {
        ensure_history_texture(&device, target, WIDTH, HEIGHT);
        let bgra = pixels
            .chunks_exact(4)
            .flat_map(|pixel| [pixel[2], pixel[1], pixel[0], pixel[3]])
            .collect::<Vec<_>>();
        queue.write_texture(
            target.as_ref().unwrap().texture.as_image_copy(),
            &bgra,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(WIDTH * 4),
                rows_per_image: Some(HEIGHT),
            },
            wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
        );
    }
    let original = original.unwrap();
    let history = history.unwrap();
    let input_group = |input: &wgpu::TextureView| {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("synthetic input bindings"),
            layout: &input_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniforms.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&original.view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: parameters.as_entire_binding(),
                },
            ],
        })
    };
    let pair_group = |layout: &wgpu::BindGroupLayout, view: &wgpu::TextureView| {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("synthetic texture pair"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        })
    };
    let original_group = input_group(&original.view);
    let history_group = pair_group(&history_layout, &history.view);
    let fallback_pass_group = pair_group(&pass_layout, &original.view);
    let mut intermediate = None;
    ensure_pass_target(
        &device,
        wgpu::TextureFormat::Rgba8Unorm,
        &mut intermediate,
        WIDTH,
        HEIGHT,
        true,
    );
    let intermediate = intermediate.unwrap();
    let output = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("synthetic readable output"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let output_view = output.create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&Default::default());
    let groups = [&original_group, &history_group, &fallback_pass_group];
    draw(&mut encoder, &output_view, &blit, groups);
    let blit_readback = copy_readback(&device, &mut encoder, &output);
    draw(&mut encoder, &intermediate.attachment, &blit, groups);
    MipmapGenerator::new(&device).encode(&device, &mut encoder, &intermediate.sampled);
    let intermediate_group = input_group(&intermediate.sampled.view);
    let previous_pass_group = pair_group(&pass_layout, &intermediate.sampled.view);
    draw(
        &mut encoder,
        &output_view,
        &history_pass,
        [&intermediate_group, &history_group, &previous_pass_group],
    );
    let history_readback = copy_readback(&device, &mut encoder, &output);
    queue.submit([encoder.finish()]);
    assert_eq!(read_rgba(&device, &blit_readback), current_rgba);
    let expected_history = previous_rgba
        .chunks_exact(4)
        .zip(current_rgba.chunks_exact(4))
        .enumerate()
        .flat_map(|(index, (previous, current))| {
            if index as u32 % WIDTH < WIDTH / 2 {
                previous
            } else {
                current
            }
            .iter()
            .copied()
        })
        .collect::<Vec<_>>();
    assert_eq!(read_rgba(&device, &history_readback), expected_history);
}
