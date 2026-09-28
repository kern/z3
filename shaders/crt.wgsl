// A lightweight CRT-style pixel shader for zrust's wgpu renderer.
// Binding 0 is the emulated SNES framebuffer and binding 1 is its sampler.
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
    let dimensions = vec2<f32>(textureDimensions(source_texture));
    let pixel = input.tex_coords * dimensions;
    var color = textureSample(source_texture, source_sampler, input.tex_coords).rgb;

    // Alternate scanline intensity and apply a subtle repeating RGB phosphor mask.
    let scanline = 0.88 + 0.12 * cos(3.14159265 * pixel.y);
    let phosphor = u32(pixel.x) % 3u;
    var mask = vec3<f32>(0.92);
    if phosphor == 0u {
        mask.r = 1.0;
    } else if phosphor == 1u {
        mask.g = 1.0;
    } else {
        mask.b = 1.0;
    }
    color *= scanline * mask;
    return vec4<f32>(color, 1.0);
}
