use std::path::{Path, PathBuf};

use super::{
    plan::{PassScale, ShaderUniforms, referenced_slots, scaled_size},
    preset::{load_shader_plan, parse_wgsl_preset},
    resources::{downsample_rgba, mip_level_count},
    source::{BLIT_SHADER, DIM_SHADER, read_shader_source},
};

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

#[test]
fn recursive_shader_includes_report_the_original_path() {
    let directory = temporary_shader_directory("recursive-include");
    let shader = directory.join("pass.wgsl");
    let include = directory.join("common.wgsl");
    std::fs::write(&shader, "#include \"common.wgsl\"\n").unwrap();
    std::fs::write(&include, "#include \"pass.wgsl\"\n").unwrap();
    let error = read_shader_source(&shader).unwrap_err();
    assert_eq!(
        error.to_string(),
        format!("recursive WGSL include involving {}", shader.display())
    );
    std::fs::write(&shader, "// header\n#include \"\"\n").unwrap();
    let error = read_shader_source(&shader).unwrap_err();
    assert_eq!(
        error.to_string(),
        format!("invalid WGSL preset {}:2: empty #include", shader.display())
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn includes_keep_source_order_and_replace_parameters_in_their_original_slots() {
    let directory = temporary_shader_directory("parameter-override");
    let shader = directory.join("pass.WGSL");
    std::fs::write(
        directory.join("common.wgsl"),
        "#pragma parameter GAIN \"Initial gain\" 1 0 2 0.1\nconst INCLUDED = 1;\n",
    )
    .unwrap();
    std::fs::write(
        &shader,
        concat!(
            "const BEFORE = 0;\n#include <common.wgsl>\n",
            "#pragma parameter OFFSET \"Color offset\" 0 -1 1 0.2\n",
            "#pragma parameter GAIN \"Updated gain\" 3 0 4 0.5\n",
            "const AFTER = 2;\n"
        ),
    )
    .unwrap();
    let plan = load_shader_plan(Some(&shader)).unwrap();
    assert_eq!(plan.len(), 1);
    assert_eq!(
        plan[0].source,
        "const BEFORE = 0;\nconst INCLUDED = 1;\nconst AFTER = 2;\n"
    );
    assert_eq!(plan[0].parameters.len(), 2);
    let gain = &plan[0].parameters[0];
    assert_eq!(gain.id, "GAIN");
    assert_eq!(
        (gain.value, gain.min, gain.max, gain.step),
        (3.0, 0.0, 4.0, 0.5)
    );
    assert_eq!(plan[0].parameters[1].id, "OFFSET");
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn preset_errors_preserve_line_numbers_and_validation_boundaries() {
    let directory = temporary_shader_directory("invalid-preset");
    let preset = directory.join("test.wgslp");
    for (text, line, message) in [
        ("shaders = 0\n", 1, "shaders must be between 1 and 20"),
        ("shaders = 21\n", 1, "shaders must be between 1 and 20"),
        (
            "shaders = 1\nshader1 = missing.wgsl\n",
            2,
            "pass index is out of range",
        ),
        (
            "shaders = 1\nscale0 = NaN\n",
            2,
            "expected a positive finite number",
        ),
        (
            "shaders = 1\nscale0 = 0\n",
            2,
            "expected a positive finite number",
        ),
        (
            "shaders = 1\nfilter_linear0 = yes\n",
            2,
            "expected a boolean",
        ),
    ] {
        std::fs::write(&preset, text).unwrap();
        assert_eq!(
            parse_wgsl_preset(&preset).unwrap_err().to_string(),
            format!("invalid WGSL preset {}:{line}: {message}", preset.display())
        );
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn auxiliary_texture_binding_order_is_shared_by_every_pass() {
    let directory = temporary_shader_directory("texture-order");
    let preset = directory.join("test.wgslp");
    std::fs::write(directory.join("pass.wgsl"), BLIT_SHADER).unwrap();
    std::fs::write(
        &preset,
        concat!(
            "shaders = 2\nshader0 = pass.wgsl\nshader1 = pass.wgsl\n",
            "textures = SECOND;FIRST\nFIRST = first.png\nSECOND = second.png\n",
            "SECOND_linear = true\nFIRST_wrap_mode = mirrored_repeat\n"
        ),
    )
    .unwrap();
    let plan = parse_wgsl_preset(&preset).unwrap();
    assert_eq!(plan[0].textures, plan[1].textures);
    assert_eq!(plan[0].textures[0].id, "SECOND");
    assert_eq!(plan[0].textures[1].id, "FIRST");
    assert!(plan[0].textures[0].linear);
    assert_eq!(
        plan[0].textures[1].address_mode,
        wgpu::AddressMode::MirrorRepeat
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn uniform_layout_and_odd_mipmap_edges_retain_the_shader_contract() {
    assert_eq!(std::mem::size_of::<ShaderUniforms>(), 48);
    assert_eq!(std::mem::offset_of!(ShaderUniforms, original_size), 24);
    assert_eq!(std::mem::offset_of!(ShaderUniforms, frame_count), 32);
    assert_eq!(std::mem::offset_of!(ShaderUniforms, _padding), 44);
    for (width, height) in [(3, 1), (1, 3)] {
        let pixels = [0, 0, 0, 0, 8, 8, 8, 8, 255, 255, 255, 255];
        let (result, next_width, next_height) = downsample_rgba(&pixels, width, height);
        assert_eq!((next_width, next_height), (1, 1));
        assert_eq!(result, [4, 4, 4, 4]);
        assert_eq!(mip_level_count(width, height), 2);
    }
    assert_eq!(scaled_size(PassScale::Source(0.5), 3, 800), 1);
    assert_eq!(scaled_size(PassScale::Absolute(0.25), 3, 800), 1);
}
