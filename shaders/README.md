# wgpu shaders

The native wgpu renderer accepts either one `.wgsl` file or a `.wgslp`
multi-pass preset. Every WGSL pass must provide `vs_main` and `fs_main` entry
points. Rewritten shaders use this binding contract; unused bindings may be
omitted from the WGSL source:

```wgsl
@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

struct ZeldaUniforms {
    input_size: vec2<f32>,
    texture_size: vec2<f32>,
    output_size: vec2<f32>,
    original_size: vec2<f32>,
    frame_count: u32,
    frame_direction: i32,
    pass_index: u32,
    padding: u32,
}
@group(0) @binding(2) var<uniform> zelda: ZeldaUniforms;
@group(0) @binding(3) var original_texture: texture_2d<f32>;
@group(0) @binding(4) var original_sampler: sampler;

struct ZeldaParameters { values: array<vec4<f32>, 64>, }
@group(0) @binding(5) var<uniform> parameters: ZeldaParameters;
```

`frame_direction` is always 1, matching the C renderer. `frame_count_modN`
applies independently to each pass. Parameters occupy declaration order; use
`parameters.values[index / 4][index % 4]`. A shader or included file declares
them with the C syntax:

```wgsl
#pragma parameter BLOOM "Bloom strength" 0.5 0.0 1.0 0.05
```

The loader removes pragmas before WGSL compilation. `#include "file.wgsl"` is
recursive and relative to the including file. Preset overrides are clamped to
the declared range.

WGSLP uses the C port's pass keys. Paths are relative to the preset file.

```ini
shaders = 2
shader0 = "first.wgsl"
filter_linear0 = false
scale_type0 = source
scale0 = 2
wrap_mode0 = clamp_to_edge
float_framebuffer0 = true
shader1 = "second.wgsl"
filter_linear1 = true
mipmap_input1 = true
frame_count_mod1 = 120
```

Supported scale types are `source`, `viewport`, `absolute`, and `none`.
Supported wrap modes are `repeat`, `mirrored_repeat`, `clamp_to_edge`, `clamp`,
and `clamp_to_border`; the last two use a transparent-black border like C.
Hardware without wgpu's optional border-clamp feature reports a clear error. Intermediate
passes support RGBA32F `float_framebufferN`, `srgb_framebufferN`, and
`mipmap_inputN`. Hardware without filterable RGBA32F reports a clear error.

Auxiliary PNG textures use the C preset syntax. They are assigned bindings in
`textures` order starting at group 0 binding 6, as texture/sampler pairs:

```ini
textures = "MASK;NOISE"
MASK = "mask.png"
MASK_linear = true
MASK_mipmap = true
MASK_wrap_mode = repeat
```

Here `MASK` is bindings 6/7 and `NOISE` is 8/9. PNG inputs are normalized to
RGBA8 and mip levels are generated when requested.

Frame and pass history are texture/sampler pairs. `prev_texture_N` at group 1
bindings `N*2`/`N*2+1` exposes prior original frame N for N=0..6.
`pass_texture_N` at group 2 exposes zero-based pass N's output. Only slots named
in shader source are allocated. Use `textureDimensions()` where a GLSL shader
used that texture's `InputSize` or `TextureSize` uniform.

Legacy `.glsl` and `.glslp` need rewriting because wgpu consumes WGSL and
cannot reflect arbitrary GLSL uniform names. `.wgslp` otherwise retains the C
preset's rendering facilities. `aliasN` is accepted but has no effect because
WGSL resources use explicit bindings.
