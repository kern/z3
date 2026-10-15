//! Built-in shader text and recursive WGSL includes/parameter declarations.

use std::path::{Path, PathBuf};

use super::{Error, plan::ShaderParameter};

pub(super) const BLIT_SHADER: &str = r#"
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

pub(super) const DIM_SHADER: &str = r#"
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

pub(super) const MIPMAP_SHADER: &str = r#"
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

pub(super) fn read_shader(path: &Path) -> Result<String, Error> {
    std::fs::read_to_string(path).map_err(|error| {
        Error::message(format!("unable to read shader {}: {error}", path.display()))
    })
}

pub(super) fn read_shader_source(path: &Path) -> Result<(String, Vec<ShaderParameter>), Error> {
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

pub(super) fn preset_error(path: &Path, line: usize, message: &str) -> Error {
    Error::message(format!(
        "invalid WGSL preset {}:{line}: {message}",
        path.display()
    ))
}
