//! Decode WGSL presets without creating or consulting GPU resources.

use std::path::{Path, PathBuf};

use super::{
    Error,
    plan::{AuxTexturePlan, PassScale, ShaderPassPlan},
    source::{BLIT_SHADER, preset_error, read_shader, read_shader_source},
};

pub(super) fn load_shader_plan(path: Option<&Path>) -> Result<Vec<ShaderPassPlan>, Error> {
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

pub(super) fn parse_wgsl_preset(path: &Path) -> Result<Vec<ShaderPassPlan>, Error> {
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
