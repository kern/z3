//! Differential test of the sprite-view migration against the frozen
//! pre-migration implementation.
//!
//! Both implementations receive identical whole-WRAM images and an identical
//! logging runtime, and must leave identical WRAM, runtime logs and panic
//! outcomes. Key state bytes (modules, frame counter) are swept so every
//! state-machine branch is reachable.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Once, OnceLock};

use super::{DrawMultipleData, Ending, EndingCall, EndingRuntime, IntroSpriteEnt, frozen};
use crate::ram::Ram;
use z3_core::WRAM_SIZE;

struct Tables {
    map: Vec<u16>,
    credits_text: Vec<u8>,
    credits_offsets: Vec<u16>,
    scene_text: Vec<u8>,
    scene_offsets: Vec<u16>,
}

fn tables() -> &'static Tables {
    static T: OnceLock<Tables> = OnceLock::new();
    T.get_or_init(|| {
        let mut rng = Rng(0x1234_5678_9abc_def1);
        Tables {
            map: (0..0x4000).map(|_| rng.next() as u16).collect(),
            credits_text: (0..0x4000).map(|_| rng.next() as u8).collect(),
            credits_offsets: (0..0x400).map(|_| (rng.next() % 0x3f00) as u16).collect(),
            scene_text: (0..0x4000).map(|_| rng.next() as u8).collect(),
            scene_offsets: (0..0x400).map(|_| (rng.next() % 0x3f00) as u16).collect(),
        }
    })
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

macro_rules! spy {
    ($name:ident, $trait:path, $call:ty, $dm:ty) => {
        #[derive(Default)]
        struct $name {
            log: Vec<String>,
            random: u8,
        }
        impl $trait for $name {
            fn call(&mut self, call: $call, ram: &mut Ram<'_>) {
                self.log
                    .push(format!("call {call:?} {:02x}", ram.read_u8(0x99)));
            }
            fn ppu_write(&mut self, register: u16, value: u8) {
                self.log.push(format!("ppu {register:04x} {value:02x}"));
            }
            fn sprite_reset(&mut self, _ram: &mut Ram<'_>, k: usize) {
                self.log.push(format!("reset {k}"));
            }
            fn sprite_move_xy(&mut self, _ram: &mut Ram<'_>, k: usize) {
                self.log.push(format!("move {k}"));
            }
            fn sprite_active_main(&mut self, _ram: &mut Ram<'_>, k: usize) {
                self.log.push(format!("active {k}"));
            }
            fn sprite_draw_shadow(&mut self, _ram: &mut Ram<'_>, k: usize) {
                self.log.push(format!("shadow {k}"));
            }
            fn oam_allocate_region_a(&mut self, _ram: &mut Ram<'_>, bytes: u8) {
                self.log.push(format!("alloc {bytes}"));
            }
            fn sprite_get_16_bit_coords(&mut self, _ram: &mut Ram<'_>, k: usize) {
                self.log.push(format!("coords {k}"));
            }
            fn sprite_draw_multiple(&mut self, _ram: &mut Ram<'_>, k: usize, entries: &[$dm]) {
                self.log.push(format!("multi {k} {}", entries.len()));
                for e in entries {
                    self.log
                        .push(format!("  {} {} {} {}", e.x, e.y, e.tile, e.ext));
                }
            }
            fn ending_map_data(&self) -> &[u16] {
                &tables().map
            }
            fn ending_credits_text(&self) -> &[u8] {
                &tables().credits_text
            }
            fn ending_credits_offsets(&self) -> &[u16] {
                &tables().credits_offsets
            }
            fn ending_scene_text(&self) -> &[u8] {
                &tables().scene_text
            }
            fn ending_scene_offsets(&self) -> &[u16] {
                &tables().scene_offsets
            }
            fn random_number(&mut self, _ram: &mut Ram<'_>) -> u8 {
                self.random = self.random.wrapping_mul(75).wrapping_add(74);
                self.random
            }
            fn generated_end_sequence_15(&self, frame: u8) -> u8 {
                frame.wrapping_mul(7) ^ 0x5a
            }
            fn decompress_animated_overworld_tiles(&mut self, _ram: &mut Ram<'_>, set: u8) {
                self.log.push(format!("ow tiles {set}"));
            }
            fn get_overworld_bg_palette(&mut self, screen: u8) -> u8 {
                screen.wrapping_mul(3).wrapping_add(1)
            }
            fn overworld_load_palettes(&mut self, _ram: &mut Ram<'_>, bg: u8, sprites: u8) {
                self.log.push(format!("ow pal {bg} {sprites}"));
            }
            fn decompress_animated_dungeon_tiles(&mut self, _ram: &mut Ram<'_>, set: u8) {
                self.log.push(format!("dg tiles {set}"));
            }
            fn dungeon_palette_info(&mut self, index: u8) -> (u8, u8) {
                (index.wrapping_mul(5), index ^ 0x33)
            }
            fn health_after_death(&self, capacity_index: usize) -> u8 {
                capacity_index as u8 ^ 0x17
            }
            fn save_game_file(&mut self, _ram: &mut Ram<'_>) {
                self.log.push("save".into());
            }
            fn credits_hdma_setup(&mut self, _ram: &mut Ram<'_>) {
                self.log.push("hdma".into());
            }
            fn vram_write_word(&mut self, word_address: usize, value: u16) {
                self.log.push(format!("vram {word_address:x} {value:04x}"));
            }
            fn dungeon_animated_tile_for_theme(&self, theme: u8) -> u8 {
                theme ^ 0x6c
            }
            fn overworld_load_area_palettes_ex(&mut self, _ram: &mut Ram<'_>, index: u8) {
                self.log.push(format!("area pal {index}"));
            }
        }
    };
}
spy!(NewSpy, EndingRuntime, EndingCall, DrawMultipleData);
spy!(
    OldSpy,
    frozen::EndingRuntime,
    frozen::EndingCall,
    frozen::DrawMultipleData
);

type NewFn<'f> = &'f mut dyn FnMut(&mut Ram<'_>, &mut NewSpy);
type OldFn<'f> = &'f mut dyn FnMut(&mut Ram<'_>, &mut OldSpy);

fn silence_expected_panics() {
    static ONCE: Once = Once::new();
    thread_local!(static QUIET: std::cell::Cell<bool> = const { std::cell::Cell::new(false) });
    ONCE.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if !QUIET.with(|q| q.get()) {
                previous(info);
            }
        }));
    });
    QUIET.with(|q| q.set(true));
}

fn outcome(result: std::thread::Result<()>) -> Option<String> {
    result.err().map(|payload| {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned()))
            .unwrap_or_default()
    })
}

fn compare(name: &str, label: &str, base: &[u8; WRAM_SIZE], new: NewFn<'_>, old: OldFn<'_>) {
    let mut a = Box::new(*base);
    let mut b = Box::new(*base);
    let mut spy_a = NewSpy::default();
    let mut spy_b = OldSpy::default();
    let result_a = {
        let mut ram = Ram::new(&mut a);
        outcome(catch_unwind(AssertUnwindSafe(|| new(&mut ram, &mut spy_a))))
    };
    let result_b = {
        let mut ram = Ram::new(&mut b);
        outcome(catch_unwind(AssertUnwindSafe(|| old(&mut ram, &mut spy_b))))
    };
    assert_eq!(result_a, result_b, "{name} {label}: panic outcome");
    assert_eq!(spy_a.log, spy_b.log, "{name} {label}: runtime log");
    if let Some(i) = (0..WRAM_SIZE).find(|&i| a[i] != b[i]) {
        panic!(
            "{name} {label}: WRAM differs at {i:#x}: {:#04x} vs {:#04x}",
            a[i], b[i]
        );
    }
}

fn bases(count: usize, seed: u64) -> Vec<Box<[u8; WRAM_SIZE]>> {
    let mut rng = Rng(seed);
    (0..count)
        .map(|_| {
            let mut image = Box::new([0u8; WRAM_SIZE]);
            for chunk in image.chunks_mut(8) {
                let word = rng.next().to_le_bytes();
                chunk.copy_from_slice(&word[..chunk.len()]);
            }
            image
        })
        .collect()
}

/// Entry points without arguments: sweep the module, submodule and
/// sub-submodule bytes that select state-machine branches.
fn sweep_modules(name: &str, new: NewFn<'_>, old: OldFn<'_>) {
    silence_expected_panics();
    for (n, base) in bases(3, 0xe11d_0001).iter().enumerate() {
        for sub in 0..40u8 {
            for subsub in 0..24u8 {
                let mut image = base.clone();
                image[0x10] = [0x00, 0x18, 0x19, 0x1a][n % 4];
                image[0x11] = sub;
                image[0xb0] = subsub;
                compare(
                    name,
                    &format!("base {n} sub {sub} subsub {subsub}"),
                    &image,
                    &mut *new,
                    &mut *old,
                );
            }
        }
    }
}

/// Entry points with arguments: sweep the arguments over random images.
fn sweep_args(name: &str, args: &str, new: NewFn<'_>, old: OldFn<'_>) {
    silence_expected_panics();
    for (n, base) in bases(12, 0xe11d_0002).iter().enumerate() {
        let mut image = base.clone();
        image[0x11] = (n * 3) as u8;
        image[0xb0] = n as u8;
        compare(
            name,
            &format!("{args} base {n}"),
            &image,
            &mut *new,
            &mut *old,
        );
    }
}

#[test]
fn sprite_view_migration_matches_frozen_pre_migration_ending() {
    sweep_modules(
        "credits_load_scene_overworld_prep_gfx",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_load_scene_overworld_prep_gfx(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_load_scene_overworld_prep_gfx(),
    );
    sweep_modules(
        "credits_load_scene_overworld_overlay",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_load_scene_overworld_overlay(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_load_scene_overworld_overlay(),
    );
    sweep_modules(
        "credits_load_scene_overworld_load_map",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_load_scene_overworld_load_map(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_load_scene_overworld_load_map(),
    );
    sweep_modules(
        "credits_load_cool_background",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_load_cool_background(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_load_cool_background(),
    );
    sweep_modules(
        "credits_load_scene_dungeon",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_load_scene_dungeon(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_load_scene_dungeon(),
    );
    sweep_modules(
        "credits_load_next_scene_overworld",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_load_next_scene_overworld(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_load_next_scene_overworld(),
    );
    sweep_modules(
        "credits_load_next_scene_dungeon",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_load_next_scene_dungeon(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_load_next_scene_dungeon(),
    );
    sweep_modules(
        "module1a_credits",
        &mut |ram, rt| Ending { ram, runtime: rt }.module1a_credits(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.module1a_credits(),
    );
    sweep_modules(
        "module18_ganon_emerges",
        &mut |ram, rt| Ending { ram, runtime: rt }.module18_ganon_emerges(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.module18_ganon_emerges(),
    );
    sweep_modules(
        "module19_triforce_room",
        &mut |ram, rt| Ending { ram, runtime: rt }.module19_triforce_room(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.module19_triforce_room(),
    );
    sweep_modules(
        "intro_initialize_background_settings",
        &mut |ram, rt| Ending { ram, runtime: rt }.intro_initialize_background_settings(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_initialize_background_settings(),
    );
    sweep_modules(
        "polyhedral_initialize_thread",
        &mut |ram, rt| Ending { ram, runtime: rt }.polyhedral_initialize_thread(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.polyhedral_initialize_thread(),
    );
    sweep_modules(
        "load_triforce_sprite_palette",
        &mut |ram, rt| Ending { ram, runtime: rt }.load_triforce_sprite_palette(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.load_triforce_sprite_palette(),
    );
    sweep_modules(
        "intro_init_gfx_helper",
        &mut |ram, rt| Ending { ram, runtime: rt }.intro_init_gfx_helper(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_init_gfx_helper(),
    );
    sweep_modules(
        "intro_setup_screen",
        &mut |ram, rt| Ending { ram, runtime: rt }.intro_setup_screen(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_setup_screen(),
    );
    sweep_modules(
        "intro_load_text_pointers_and_palettes",
        &mut |ram, rt| Ending { ram, runtime: rt }.intro_load_text_pointers_and_palettes(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_load_text_pointers_and_palettes(),
    );
    for enhanced_features0 in [0u32, 1, 0xffff_ffff, 0xa5a5_a5a5] {
        sweep_args(
            "module00_intro",
            &format!("enhanced_features0={enhanced_features0}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.module00_intro(enhanced_features0),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.module00_intro(enhanced_features0),
        );
    }

    sweep_modules(
        "intro_init",
        &mut |ram, rt| Ending { ram, runtime: rt }.intro_init(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_init(),
    );
    sweep_modules(
        "intro_init_continue",
        &mut |ram, rt| Ending { ram, runtime: rt }.intro_init_continue(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_init_continue(),
    );
    sweep_modules(
        "intro_clear_1kb_blocks_of_wram",
        &mut |ram, rt| Ending { ram, runtime: rt }.intro_clear_1kb_blocks_of_wram(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_clear_1kb_blocks_of_wram(),
    );
    sweep_modules(
        "intro_initialize_memory_darken",
        &mut |ram, rt| Ending { ram, runtime: rt }.intro_initialize_memory_darken(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_initialize_memory_darken(),
    );
    sweep_modules(
        "intro_zelda_fadein",
        &mut |ram, rt| Ending { ram, runtime: rt }.intro_zelda_fadein(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_zelda_fadein(),
    );
    for features in [0u32, 1, 0xffff_ffff, 0xa5a5_a5a5] {
        sweep_args(
            "intro_fade_in_bg",
            &format!("features={features}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.intro_fade_in_bg(features),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_fade_in_bg(features),
        );
    }

    for features in [0u32, 1, 0xffff_ffff, 0xa5a5_a5a5] {
        sweep_args(
            "intro_sword_coming_down",
            &format!("features={features}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.intro_sword_coming_down(features),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_sword_coming_down(features),
        );
    }

    for features in [0u32, 1, 0xffff_ffff, 0xa5a5_a5a5] {
        sweep_args(
            "intro_wait_player",
            &format!("features={features}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.intro_wait_player(features),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_wait_player(features),
        );
    }

    sweep_modules(
        "fade_music_and_reset_sram_mirror",
        &mut |ram, rt| Ending { ram, runtime: rt }.fade_music_and_reset_sram_mirror(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.fade_music_and_reset_sram_mirror(),
    );
    sweep_modules(
        "intro_initialize_triforce_poly_thread",
        &mut |ram, rt| Ending { ram, runtime: rt }.intro_initialize_triforce_poly_thread(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_initialize_triforce_poly_thread(),
    );
    sweep_modules(
        "intro_handle_all_triforce_animations",
        &mut |ram, rt| Ending { ram, runtime: rt }.intro_handle_all_triforce_animations(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_handle_all_triforce_animations(),
    );
    sweep_modules(
        "scene_animate_every_sprite",
        &mut |ram, rt| Ending { ram, runtime: rt }.scene_animate_every_sprite(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.scene_animate_every_sprite(),
    );
    sweep_modules(
        "intro_animate_triforce",
        &mut |ram, rt| Ending { ram, runtime: rt }.intro_animate_triforce(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_animate_triforce(),
    );
    sweep_modules(
        "intro_run_step",
        &mut |ram, rt| Ending { ram, runtime: rt }.intro_run_step(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_run_step(),
    );
    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "intro_anim_one_obj",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.intro_anim_one_obj(k),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_anim_one_obj(k),
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "intro_sprite_type_a_0",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.intro_sprite_type_a_0(k),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_sprite_type_a_0(k),
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "intro_sprite_type_b_0",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.intro_sprite_type_b_0(k),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_sprite_type_b_0(k),
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "animate_scene_sprite_draw_triangle",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.animate_scene_sprite_draw_triangle(k),
            &mut |ram, rt| {
                frozen::Ending { ram, runtime: rt }.animate_scene_sprite_draw_triangle(k)
            },
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        let entries = [
            IntroSpriteEnt {
                x: 3,
                y: -4,
                charnum: 0x21,
                flags: 0x35,
                ext: 2,
            },
            IntroSpriteEnt {
                x: -8,
                y: 9,
                charnum: 0xf0,
                flags: 0x41,
                ext: 0,
            },
        ];
        let old_entries = [
            frozen::IntroSpriteEnt {
                x: 3,
                y: -4,
                charnum: 0x21,
                flags: 0x35,
                ext: 2,
            },
            frozen::IntroSpriteEnt {
                x: -8,
                y: 9,
                charnum: 0xf0,
                flags: 0x41,
                ext: 0,
            },
        ];
        sweep_args(
            "animate_scene_sprite_add_objects_to_oam_buffer",
            &format!("k={k}"),
            &mut |ram, rt| {
                Ending { ram, runtime: rt }
                    .animate_scene_sprite_add_objects_to_oam_buffer(k, &entries)
            },
            &mut |ram, rt| {
                frozen::Ending { ram, runtime: rt }
                    .animate_scene_sprite_add_objects_to_oam_buffer(k, &old_entries)
            },
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "exit_0cca90",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.exit_0cca90(k),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.exit_0cca90(k),
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "intro_copy_sprite_type4_to_oam",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.intro_copy_sprite_type4_to_oam(k),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_copy_sprite_type4_to_oam(k),
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "initialize_scene_sprite_copyright",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.initialize_scene_sprite_copyright(k),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.initialize_scene_sprite_copyright(k),
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "animate_scene_sprite_copyright",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.animate_scene_sprite_copyright(k),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.animate_scene_sprite_copyright(k),
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "initialize_scene_sprite_sparkle",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.initialize_scene_sprite_sparkle(k),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.initialize_scene_sprite_sparkle(k),
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "animate_scene_sprite_sparkle",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.animate_scene_sprite_sparkle(k),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.animate_scene_sprite_sparkle(k),
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "animate_scene_sprite_move_triangle",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.animate_scene_sprite_move_triangle(k),
            &mut |ram, rt| {
                frozen::Ending { ram, runtime: rt }.animate_scene_sprite_move_triangle(k)
            },
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "initialize_scene_sprite_triforce_room_triangle",
            &format!("k={k}"),
            &mut |ram, rt| {
                Ending { ram, runtime: rt }.initialize_scene_sprite_triforce_room_triangle(k)
            },
            &mut |ram, rt| {
                frozen::Ending { ram, runtime: rt }
                    .initialize_scene_sprite_triforce_room_triangle(k)
            },
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "animate_triforce_room_triangle_handle_contracting",
            &format!("k={k}"),
            &mut |ram, rt| {
                Ending { ram, runtime: rt }.animate_triforce_room_triangle_handle_contracting(k)
            },
            &mut |ram, rt| {
                frozen::Ending { ram, runtime: rt }
                    .animate_triforce_room_triangle_handle_contracting(k)
            },
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "intro_sprite_type_b_456",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.intro_sprite_type_b_456(k),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_sprite_type_b_456(k),
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "initialize_scene_sprite_credits_triangle",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.initialize_scene_sprite_credits_triangle(k),
            &mut |ram, rt| {
                frozen::Ending { ram, runtime: rt }.initialize_scene_sprite_credits_triangle(k)
            },
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "animate_scene_sprite_credits_triangle",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.animate_scene_sprite_credits_triangle(k),
            &mut |ram, rt| {
                frozen::Ending { ram, runtime: rt }.animate_scene_sprite_credits_triangle(k)
            },
        );
    }

    sweep_modules(
        "triforce_room_prep_gfx_slot_for_poly",
        &mut |ram, rt| Ending { ram, runtime: rt }.triforce_room_prep_gfx_slot_for_poly(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.triforce_room_prep_gfx_slot_for_poly(),
    );
    sweep_modules(
        "credits_initialize_polyhedral",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_initialize_polyhedral(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_initialize_polyhedral(),
    );
    sweep_modules(
        "advance_polyhedral",
        &mut |ram, rt| Ending { ram, runtime: rt }.advance_polyhedral(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.advance_polyhedral(),
    );
    sweep_modules(
        "triforce_room_handle_poly",
        &mut |ram, rt| Ending { ram, runtime: rt }.triforce_room_handle_poly(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.triforce_room_handle_poly(),
    );
    sweep_modules(
        "credits_animate_the_triangles",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_animate_the_triangles(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_animate_the_triangles(),
    );
    sweep_modules(
        "intro_display_logo",
        &mut |ram, rt| Ending { ram, runtime: rt }.intro_display_logo(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_display_logo(),
    );
    sweep_modules(
        "intro_setup_sword_and_intro_flash",
        &mut |ram, rt| Ending { ram, runtime: rt }.intro_setup_sword_and_intro_flash(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.intro_setup_sword_and_intro_flash(),
    );
    for features in [0u32, 1, 0xffff_ffff, 0xa5a5_a5a5] {
        sweep_args(
            "intro_periodic_sword_and_intro_flash",
            &format!("features={features}"),
            &mut |ram, rt| {
                Ending { ram, runtime: rt }.intro_periodic_sword_and_intro_flash(features)
            },
            &mut |ram, rt| {
                frozen::Ending { ram, runtime: rt }.intro_periodic_sword_and_intro_flash(features)
            },
        );
    }

    sweep_modules(
        "credits_prep_and_load_sprites",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_prep_and_load_sprites(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_prep_and_load_sprites(),
    );
    sweep_modules(
        "credits_scroll_scene_overworld",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_scroll_scene_overworld(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_scroll_scene_overworld(),
    );
    sweep_modules(
        "credits_scroll_scene_dungeon",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_scroll_scene_dungeon(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_scroll_scene_dungeon(),
    );
    sweep_modules(
        "credits_handle_scene_fade",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_handle_scene_fade(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_handle_scene_fade(),
    );
    sweep_modules(
        "credits_operate_scrolling_and_tile_map",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_operate_scrolling_and_tile_map(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_operate_scrolling_and_tile_map(),
    );
    sweep_modules(
        "credits_handle_camera_scroll_control",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_handle_camera_scroll_control(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_handle_camera_scroll_control(),
    );
    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        for ain in [0u8, 1, 0x30, 0xff] {
            sweep_args(
                "ending_func2",
                &format!("k={k} ain={ain}"),
                &mut |ram, rt| Ending { ram, runtime: rt }.ending_func2(k, ain),
                &mut |ram, rt| frozen::Ending { ram, runtime: rt }.ending_func2(k, ain),
            );
        }
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        for a in [0u8, 1, 0x30, 0xff] {
            sweep_args(
                "credits_sprite_draw_set_shadow_prop",
                &format!("k={k} a={a}"),
                &mut |ram, rt| {
                    Ending { ram, runtime: rt }.credits_sprite_draw_set_shadow_prop(k, a)
                },
                &mut |ram, rt| {
                    frozen::Ending { ram, runtime: rt }.credits_sprite_draw_set_shadow_prop(k, a)
                },
            );
        }
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "credits_sprite_draw_draw_shadow",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.credits_sprite_draw_draw_shadow(k),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_sprite_draw_draw_shadow(k),
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "end_sequence_draw_shadow2",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.end_sequence_draw_shadow2(k),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.end_sequence_draw_shadow2(k),
        );
    }

    for count in [0usize, 1, 3, 8] {
        for xb in [0u8, 1, 0x30, 0xff] {
            for yb in [0u8, 1, 0x30, 0xff] {
                sweep_args(
                    "credits_sprite_draw_add_sparkle",
                    &format!("count={count} xb={xb} yb={yb}"),
                    &mut |ram, rt| {
                        Ending { ram, runtime: rt }.credits_sprite_draw_add_sparkle(count, xb, yb)
                    },
                    &mut |ram, rt| {
                        frozen::Ending { ram, runtime: rt }
                            .credits_sprite_draw_add_sparkle(count, xb, yb)
                    },
                );
            }
        }
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        for a in [0u8, 1, 0x30, 0xff] {
            for j in [0u8, 1, 0x30, 0xff] {
                sweep_args(
                    "credits_sprite_draw_single",
                    &format!("k={k} a={a} j={j}"),
                    &mut |ram, rt| Ending { ram, runtime: rt }.credits_sprite_draw_single(k, a, j),
                    &mut |ram, rt| {
                        frozen::Ending { ram, runtime: rt }.credits_sprite_draw_single(k, a, j)
                    },
                );
            }
        }
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "credits_sprite_draw_move_squirrel",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.credits_sprite_draw_move_squirrel(k),
            &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_sprite_draw_move_squirrel(k),
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "credits_sprite_draw_circling_birds",
            &format!("k={k}"),
            &mut |ram, rt| Ending { ram, runtime: rt }.credits_sprite_draw_circling_birds(k),
            &mut |ram, rt| {
                frozen::Ending { ram, runtime: rt }.credits_sprite_draw_circling_birds(k)
            },
        );
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        for bytes in [0u8, 1, 0x30, 0xff] {
            sweep_args(
                "credits_sprite_draw_activate_and_run_sprite",
                &format!("k={k} bytes={bytes}"),
                &mut |ram, rt| {
                    Ending { ram, runtime: rt }
                        .credits_sprite_draw_activate_and_run_sprite(k, bytes)
                },
                &mut |ram, rt| {
                    frozen::Ending { ram, runtime: rt }
                        .credits_sprite_draw_activate_and_run_sprite(k, bytes)
                },
            );
        }
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        for bytes in [0u8, 1, 0x30, 0xff] {
            sweep_args(
                "credits_sprite_draw_preexisting_sprite_draw",
                &format!("k={k} bytes={bytes}"),
                &mut |ram, rt| {
                    Ending { ram, runtime: rt }
                        .credits_sprite_draw_preexisting_sprite_draw(k, bytes)
                },
                &mut |ram, rt| {
                    frozen::Ending { ram, runtime: rt }
                        .credits_sprite_draw_preexisting_sprite_draw(k, bytes)
                },
            );
        }
    }

    for k in [0usize, 1, 2, 5, 7, 12, 15] {
        sweep_args(
            "credits_sprite_draw_walk_link_away_from_pedestal",
            &format!("k={k}"),
            &mut |ram, rt| {
                Ending { ram, runtime: rt }.credits_sprite_draw_walk_link_away_from_pedestal(k)
            },
            &mut |ram, rt| {
                frozen::Ending { ram, runtime: rt }
                    .credits_sprite_draw_walk_link_away_from_pedestal(k)
            },
        );
    }

    sweep_modules(
        "end_sequence_32",
        &mut |ram, rt| Ending { ram, runtime: rt }.end_sequence_32(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.end_sequence_32(),
    );
    sweep_modules(
        "credits_fade_color_and_begin_animating",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_fade_color_and_begin_animating(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_fade_color_and_begin_animating(),
    );
    sweep_modules(
        "credits_add_next_attribution",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_add_next_attribution(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_add_next_attribution(),
    );
    sweep_modules(
        "credits_add_ending_sequence_text",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_add_ending_sequence_text(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_add_ending_sequence_text(),
    );
    sweep_modules(
        "credits_fade_out_fixed_col",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_fade_out_fixed_col(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_fade_out_fixed_col(),
    );
    sweep_modules(
        "credits_brighten_triangles",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_brighten_triangles(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_brighten_triangles(),
    );
    sweep_modules(
        "credits_stop_credits_scroll",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_stop_credits_scroll(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_stop_credits_scroll(),
    );
    sweep_modules(
        "credits_fade_and_disperse_triangles",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_fade_and_disperse_triangles(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_fade_and_disperse_triangles(),
    );
    sweep_modules(
        "credits_fade_in_the_end",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_fade_in_the_end(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_fade_in_the_end(),
    );
    sweep_modules(
        "credits_hang_forever",
        &mut |ram, rt| Ending { ram, runtime: rt }.credits_hang_forever(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.credits_hang_forever(),
    );
    sweep_modules(
        "crystal_cutscene_initialize_polyhedral",
        &mut |ram, rt| Ending { ram, runtime: rt }.crystal_cutscene_initialize_polyhedral(),
        &mut |ram, rt| frozen::Ending { ram, runtime: rt }.crystal_cutscene_initialize_polyhedral(),
    );
}
