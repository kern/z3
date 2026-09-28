//! Cross-unit portions of `messaging.c` used by the production service.
//!
//! These live outside `native_services.rs` so the large DungeonHooks routing
//! pass can proceed independently without losing the exact messaging ports.

use crate::{
    ancilla::Ancilla,
    dungeon::{Dungeon, DungeonHooks},
    hud::Hud,
    integration::NativeGameServices,
    load_gfx::LoadGfx,
    messaging::MessagingRuntime,
    overworld,
    ram::Ram,
};

fn with_gfx(
    services: &mut NativeGameServices<'_>,
    ram: &mut Ram<'_>,
    action: impl FnOnce(&mut LoadGfx<'_, crate::assets::RuntimeAssets<'_>, NativeGameServices<'_>>),
) {
    let mut vram = std::mem::replace(
        &mut services.scratch_vram,
        Box::new([0; z3_core::VRAM_WORDS]),
    );
    let assets = services.assets;
    action(&mut LoadGfx {
        wram: ram.bytes_mut(),
        vram: vram.as_mut(),
        assets,
        hooks: services,
    });
    services.scratch_vram = vram;
}

/// Exact `DungeonMap_RecoverGFX` body.
pub fn dungeon_map_recover_graphics(services: &mut NativeGameServices<'_>, ram: &mut Ram<'_>) {
    let hdma = ram.read_u8(0x9b);
    ram.write_u8(0x9b, 0);
    with_gfx(services, ram, |gfx| gfx.erase_tile_maps_normal());
    ram.write_u8(0x1c, ram.read_u8(0xc211));
    ram.write_u8(0x1d, ram.read_u8(0xc212));
    ram.write_u8(0xaa1, ram.read_u8(0xc20e));
    ram.write_u8(0xaa2, ram.read_u8(0xc20f));
    ram.write_u8(0xaa3, ram.read_u8(0xc210));
    with_gfx(services, ram, |gfx| gfx.initialize_tilesets());
    ram.write_u16(0xaa8, 0);
    ram.write_u8(0xab9, 0);
    Hud {
        ram,
        runtime: services,
    }
    .rebuild();
    ram.write_u8(0x418, 0);
    ram.write_u8(0x45c, 0);
    loop {
        {
            let mut dungeon = Dungeon::new(ram.bytes_mut(), services);
            dungeon.water_flood_build_one_quadrant_for_vram();
        }
        DungeonHooks::dungeon_nmi_upload_tilemap(services, ram);
        {
            let mut dungeon = Dungeon::new(ram.bytes_mut(), services);
            dungeon.dungeon_prepare_next_room_quadrant_upload();
        }
        DungeonHooks::dungeon_nmi_upload_tilemap(services, ram);
        if ram.read_u8(0x45c) == 0x10 {
            break;
        }
    }
    ram.write_u8(0x14, 0);
    ram.write_u8(0xb0, 0);
    ram.write_u8(0x9b, hdma);
    let palette = ram.slice(0x1dd80, 512).to_vec();
    ram.slice_mut(0xc500, 512).copy_from_slice(&palette);
    let plusminus = ram.read_u8(0xc017);
    for address in [0x9c, 0x9d, 0x9e] {
        ram.write_u8(address, ram.read_u8(address) | plusminus);
    }
    ram.write_u8(0x12f, 16);
    ram.write_u8(0x12c, 0xf3);
    with_gfx(services, ram, |gfx| gfx.recover_peg_gfx_from_mapping());
    ram.write_u8(0x15, ram.read_u8(0x15).wrapping_add(1));
    ram.write_u8(0x200, ram.read_u8(0x200).wrapping_add(1));
    ram.write_u8(0x13, 0);
    ram.write_u8(0x710, 0);
}

/// Exact two terminal dungeon-map dispatcher entries. The remaining drawing
/// entries are translated alongside their data tables in `messaging.rs`.
pub fn dungeon_map_terminal_step(
    services: &mut NativeGameServices<'_>,
    ram: &mut Ram<'_>,
    state: u8,
) -> bool {
    match state {
        6 => dungeon_map_recover_graphics(services, ram),
        7 | 0x1b => {
            with_gfx(services, ram, |gfx| gfx.dungeon_restore_star_tile_chr());
            ram.write_u8(0x200, ram.read_u8(0x200).wrapping_add(1));
        }
        _ => return false,
    }
    true
}

fn death_prep_faint(services: &mut NativeGameServices<'_>, ram: &mut Ram<'_>) {
    ram.write_u8(0x2f, 2);
    ram.write_u8(0x36b, 1);
    ram.write_u8(0x30d, 0);
    ram.write_u8(0x30a, 0);
    ram.write_u8(0x30b, 5);
    ram.write_u8(0xf372, 0);
    ram.write_u8(0xf36d, 0);
    MessagingRuntime::link_reset_properties_c(services, ram);
    for address in [
        0x2f5, 0x351, 0x2e0, 0x48, 0x2ec, 0x4d, 0x46, 0x373, 0x2e1, 0x5e, 0x3f7,
    ] {
        ram.write_u8(address, 0);
    }
    if ram.read_u8(0xf357) != 0 {
        ram.write_u8(0x56, 0);
    }
    ram.write_u16(0x3f5, 0);
    let pan = MessagingRuntime::calculate_link_sfx_pan(services, ram);
    ram.write_u8(0x12e, 0x27 | pan);
    if !(0..4).any(|i| ram.read_u8(0xf35c + i) == 6) {
        ram.write_u16(0x430, 0);
    }
}

fn revival_fairy_main(services: &mut NativeGameServices<'_>, ram: &mut Ram<'_>) {
    Ancilla {
        ram,
        runtime: services,
    }
    .revival_fairy_main();
}

/// Cross-unit bodies used by the game-over dispatcher in `messaging.c`.
///
/// The local timing/letter state machine remains in [`crate::messaging`]; this
/// function owns the exact LoadGfx, Ancilla, Dungeon, Hud, and Overworld calls
/// that the C translation dispatches across compilation units.
pub fn death_step(services: &mut NativeGameServices<'_>, ram: &mut Ram<'_>, state: u8) {
    match state {
        // `GameOver_DelayBeforeIris` calls this after its local timer and letter
        // initialization have completed.
        2 => with_gfx(services, ram, |gfx| gfx.iris_spotlight_close()),
        3 => {
            with_gfx(services, ram, |gfx| {
                gfx.palette_filter_restore_bg_subtractive_strict()
            });
            ram.write_u16(0xc500, ram.read_u16(0xc540));
            let main_module = ram.read_u8(0x10);
            with_gfx(services, ram, |gfx| gfx.iris_spotlight_configure_table());
            ram.write_u8(0x10, main_module);
            if ram.read_u8(0x11) != 0 {
                return;
            }
            for palette in 2..=7 {
                for color in 0..16 {
                    ram.write_u16(0xc500 + (palette * 16 + color) * 2, 0x18);
                }
            }
            ram.write_u16(0xc500, 0x18);
            ram.write_u16(0xc540, 0x18);
            with_gfx(services, ram, |gfx| gfx.iris_spotlight_reset_table());
            ram.write_u8(0x9c, 0x20);
            ram.write_u8(0x9d, 0x40);
            ram.write_u8(0x9e, 0x80);
            ram.write_u8(0x96, 0);
            ram.write_u8(0x97, 0);
            ram.write_u8(0x98, 0);
            ram.write_u8(0x11, 4);
            ram.write_u8(0x15, ram.read_u8(0x15).wrapping_add(1));
            ram.write_u8(0x13, 15);
            ram.write_u8(0x1c, 20);
            ram.write_u8(0x1d, 0);
            ram.write_u8(0x99, 32);
            ram.write_u8(0xc8, 64);
            ram.write_u8(0xc007, 0);
            ram.write_u8(0xc009, 0);
            death_prep_faint(services, ram);
        }
        5 => {
            with_gfx(services, ram, |gfx| {
                gfx.palette_filter_restore_bg_subtractive_strict()
            });
            ram.write_u16(0xc500, ram.read_u16(0xc540));
            if ram.read_u8(0xc009) != 0xff {
                return;
            }
            ram.write_u8(0xc011, 0);
            ram.write_u8(0x647, 0);
            ram.write_u8(0x95, 3);
            for i in 0..4 {
                if ram.read_u8(0xf35c + i) == 6 {
                    ram.write_u8(0xf35c + i, 2);
                    ram.write_u8(0xc8, 12);
                    ram.write_u8(0xaa, 15);
                    with_gfx(services, ram, |gfx| gfx.graphics_load_chr_half_slot());
                    ram.write_u8(0xaa, 0);
                    ram.write_u8(0x11, 10);
                    return;
                }
            }
            ram.write_u8(0x5fc, 0);
            ram.write_u8(0x5fd, 0);
            ram.write_u8(0x17, 22);
            ram.write_u8(0x710, 22);
            ram.write_u8(0x11, ram.read_u8(0x11).wrapping_add(1));
        }
        6 => {
            ram.write_u8(0xc8, 12);
            ram.write_u8(0xaa, 15);
            with_gfx(services, ram, |gfx| gfx.graphics_load_chr_half_slot());
            ram.write_u8(0xaa, 0);
            ram.write_u8(0xab1, 5);
            ram.write_u16(0xaa8, 0x200);
            with_gfx(services, ram, |gfx| {
                gfx.palette_load_sprite_environment_dungeon();
                gfx.palette_load_sprite_main();
            });
            ram.write_u8(0x15, ram.read_u8(0x15).wrapping_add(1));
        }
        10 => {
            Ancilla {
                ram,
                runtime: services,
            }
            .configure_revival_ancillae();
            ram.write_u8(0xf372, 56);
            ram.write_u8(0x11, ram.read_u8(0x11).wrapping_add(1));
            ram.write_u8(0x200, 0);
        }
        11 => revival_fairy_main(services, ram),
        12 => {
            if ram.read_u8(0xf372) == 0 {
                let palette = ram.slice(0x1dd80, 512).to_vec();
                ram.slice_mut(0xc300, 512).copy_from_slice(&palette);
                ram.slice_mut(0xc540, 192).fill(0);
                ram.write_u16(0xc500, 0);
                ram.write_u16(0xc007, 0);
                ram.write_u16(0xc009, 2);
                ram.write_u16(0x99, ram.read_u16(0xc225));
                ram.write_u8(0x11, ram.read_u8(0x11).wrapping_add(1));
            }
            revival_fairy_main(services, ram);
            Hud {
                ram,
                runtime: services,
            }
            .refill_logic();
        }
        13 => {
            ram.write_u8(0xaa, 1);
            with_gfx(services, ram, |gfx| gfx.graphics_load_chr_half_slot());
            let fixed_color = ram.read_u8(0xc017);
            Dungeon::new(ram.bytes_mut(), services)
                .dungeon_approach_fixed_color_variable(fixed_color);
            ram.write_u8(0x11, ram.read_u8(0x11).wrapping_add(1));
            revival_fairy_main(services, ram);
            Hud {
                ram,
                runtime: services,
            }
            .refill_logic();
        }
        14 => {
            with_gfx(services, ram, |gfx| gfx.graphics_load_chr_half_slot());
            ram.write_u8(0x1d, ram.read_u8(0xc212));
            ram.write_u8(0x11, ram.read_u8(0x11).wrapping_add(1));
        }
        15 => {
            with_gfx(services, ram, |gfx| {
                gfx.palette_filter_restore_bg_additive_strict()
            });
            ram.write_u16(0xc500, ram.read_u16(0xc540));
            if ram.read_u8(0xc007) != 32 {
                return;
            }
            if ram.read_u8(0x1b) == 0 {
                overworld::overworld_set_fixed_col_and_scroll(ram);
            }
            ram.write_u8(0x1d, ram.read_u8(0xc212));
            ram.write_u8(0x10, ram.read_u8(0x10c));
            ram.write_u8(0x11, 0);
            ram.write_u8(0x31f, 144);
            ram.write_u8(0x12c, ram.read_u8(0xc227));
            ram.write_u8(0x12d, ram.read_u8(0xc228));
            ram.write_u16(0xc007, ram.read_u16(0xc221));
            ram.write_u16(0xc009, ram.read_u16(0xc223));
        }
        0x30 => {
            Ancilla {
                ram,
                runtime: services,
            }
            .game_over_text_draw();
        }
        _ => unreachable!("unsupported native game-over dispatcher state {state:#x}"),
    }
}
