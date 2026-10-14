//! Production cross-module service composition for the native runtime.

use crate::attract::AttractRuntime;
use crate::dungeon::DungeonHooks;
use crate::ending::{Ending, EndingRuntime};
use crate::hud::{Hud, HudRenderOp, HudRuntime};
use crate::integration::{
    NativeGameServices, NativeHostAction, OverworldCall, OverworldServices, SpotlightServices,
    UnifiedGameplayServices,
};
use crate::load_gfx::{LoadGfx, LoadGfxHooks};
use crate::messaging::{Messaging, MessagingAssets, MessagingRuntime};
use crate::misc::{MiscCall, MiscHooks};
use crate::overworld::{self, PlayerControlHooks, PreOverworldHooks, TransitionHooks};
use crate::player_oam::PlayerOamHooks;
use crate::ram::Ram;
use crate::select_file::FileSelectRuntime;

/// Marker used only for LoadGfx entrypoints whose C bodies make no
/// cross-translation-unit calls. Callback-bearing entrypoints are routed
/// through `NativeGameServices::with_load_gfx`, which owns the real hooks.
struct NoGfxHooks;
impl LoadGfxHooks for NoGfxHooks {
    fn ppu_write(&mut self, _register: u8, _value: u8) {
        unreachable!("hook-free LoadGfx entrypoint wrote the PPU")
    }
    fn setup_hdma(&mut self, _table: u16, _table_bank: u8, _bbad: u8) {
        unreachable!("hook-free LoadGfx entrypoint configured HDMA")
    }
    fn set_target_overworld_warp_to_pyramid(&mut self, _wram: &mut [u8; z3_core::WRAM_SIZE]) {
        unreachable!("hook-free LoadGfx entrypoint changed warp destination")
    }
    fn pre_overworld_load_overlays(&mut self, _wram: &mut [u8; z3_core::WRAM_SIZE]) {
        unreachable!("hook-free LoadGfx entrypoint loaded overlays")
    }
    fn overworld_draw_mirror_position(&mut self, _wram: &mut [u8; z3_core::WRAM_SIZE]) {
        unreachable!("hook-free LoadGfx entrypoint drew mirror position")
    }
    fn mirror_warp_load_sprites_and_colors(&mut self, _wram: &mut [u8; z3_core::WRAM_SIZE]) {
        unreachable!("hook-free LoadGfx entrypoint loaded mirror sprites")
    }
    fn handle_followers_after_mirroring(&mut self, _wram: &mut [u8; z3_core::WRAM_SIZE]) {
        unreachable!("hook-free LoadGfx entrypoint handled followers")
    }
    fn sprite_reset_all(&mut self, _wram: &mut [u8; z3_core::WRAM_SIZE]) {
        unreachable!("hook-free LoadGfx entrypoint reset sprites")
    }
    fn file_select_gear(&mut self, _slot: usize) -> (u8, u8, u8, u8) {
        unreachable!("hook-free LoadGfx entrypoint requested SRAM gear")
    }
}

impl NativeGameServices<'_> {
    fn with_load_gfx(
        &mut self,
        ram: &mut Ram<'_>,
        operation: impl FnOnce(
            &mut LoadGfx<'_, crate::assets::RuntimeAssets<'_>, NativeGameServices<'_>>,
        ),
    ) {
        let mut vram =
            std::mem::replace(&mut self.scratch_vram, Box::new([0; z3_core::VRAM_WORDS]));
        let assets = self.assets;
        operation(&mut LoadGfx {
            wram: ram.bytes_mut(),
            vram: vram.as_mut(),
            assets,
            hooks: self,
        });
        self.scratch_vram = vram;
    }
}

struct FileSelectGfxHooks<'a> {
    sram: &'a [u8],
}
impl LoadGfxHooks for FileSelectGfxHooks<'_> {
    fn file_select_gear(&mut self, slot: usize) -> (u8, u8, u8, u8) {
        let save = slot * 0x500;
        (
            self.sram[save + 0x35b],
            self.sram[save + 0x354],
            self.sram[save + 0x359],
            self.sram[save + 0x35a],
        )
    }
}

impl LoadGfxHooks for NativeGameServices<'_> {
    fn ppu_write(&mut self, register: u8, value: u8) {
        self.pending_ppu_writes.push((register, value));
    }

    fn setup_hdma(&mut self, table: u16, table_bank: u8, bbad: u8) {
        self.pending_load_gfx_hdma.push((table, table_bank, bbad));
    }

    fn set_target_overworld_warp_to_pyramid(&mut self, wram: &mut [u8; z3_core::WRAM_SIZE]) {
        let mut ram = Ram::new(wram);
        if ram.read_u8(0x10) != 21 {
            return;
        }
        PreOverworldHooks::load_from_dungeon(self, &mut ram);
        self.with_load_gfx(&mut ram, |gfx| {
            gfx.decompress_animated_overworld_tiles(0x5a);
        });
        overworld::reset_ancilla_and_cutscene(&mut ram, |ram, selected| {
            crate::ancilla::Ancilla { ram, runtime: self }.terminate_select_interactives(selected);
        });
    }

    fn pre_overworld_load_overlays(&mut self, wram: &mut [u8; z3_core::WRAM_SIZE]) {
        let mut ram = Ram::new(wram);
        overworld::pre_overworld_load_overlays(&mut ram, |ram| {
            overworld::load_overlays_inner(ram, |ram| {
                overworld::load_overworld_overlay(ram, self.assets);
            });
        });
    }

    fn overworld_draw_mirror_position(&mut self, wram: &mut [u8; z3_core::WRAM_SIZE]) {
        let mut ram = Ram::new(wram);
        overworld::draw_screen_at_current_mirror_position(&mut ram, self.assets, |ram| {
            overworld::draw_quadrants_and_overlays(
                ram,
                self.assets,
                overworld::overworld_load_event_overlay,
            );
        });
    }

    fn mirror_warp_load_sprites_and_colors(&mut self, wram: &mut [u8; z3_core::WRAM_SIZE]) {
        let services = std::cell::RefCell::new(self);
        let assets = services.borrow().assets;
        let mut ram = Ram::new(wram);
        overworld::mirror_warp_load_sprites_and_colors(
            &mut ram,
            assets,
            |ram| {
                services
                    .borrow_mut()
                    .with_load_gfx(ram, |gfx| gfx.overworld_load_screens_palette_set())
            },
            |ram, background, sprites| {
                services
                    .borrow_mut()
                    .with_load_gfx(ram, |gfx| gfx.overworld_load_palettes(background, sprites))
            },
            |ram| {
                services
                    .borrow_mut()
                    .with_load_gfx(ram, |gfx| gfx.palette_special_ow())
            },
            |ram| {
                crate::sprite::Sprite::new(ram.bytes_mut(), &mut **services.borrow_mut())
                    .sprite_reset_all()
            },
            |ram| PreOverworldHooks::reload_overworld_sprites(&mut **services.borrow_mut(), ram),
            |ram| {
                crate::player::Player {
                    ram,
                    runtime: &mut **services.borrow_mut(),
                    applied_movement_to_camera: false,
                }
                .link_item_reset_from_overworld_things();
            },
            |ram| {
                TransitionHooks::reset_torch_background_and_player(
                    &mut **services.borrow_mut(),
                    ram,
                )
            },
            |ram| PreOverworldHooks::initialize_mirror_portal(&mut **services.borrow_mut(), ram),
        );
    }

    fn handle_followers_after_mirroring(&mut self, wram: &mut [u8; z3_core::WRAM_SIZE]) {
        let mut ram = Ram::new(wram);
        crate::player::Player {
            ram: &mut ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .handle_followers_after_mirroring();
    }

    fn sprite_reset_all(&mut self, wram: &mut [u8; z3_core::WRAM_SIZE]) {
        crate::sprite::Sprite::new(wram, self).sprite_reset_all();
    }
}
impl MessagingRuntime for NativeGameServices<'_> {
    fn misc_bugfixes_enabled(&self) -> bool {
        self.enhanced_features0 & 0x1000 != 0
    }

    fn run_hud_module(&mut self, ram: &mut Ram<'_>) {
        Hud { ram, runtime: self }.run_module();
    }

    fn hud_floor_indicator(&mut self, ram: &mut Ram<'_>) {
        Hud { ram, runtime: self }.floor_indicator();
    }

    fn link_oam_main(&mut self, ram: &mut Ram<'_>) {
        UnifiedGameplayServices::run_link_oam_main(self, ram);
    }

    fn enable_force_blank(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.enable_force_blank());
    }

    fn reset_hud_palettes_4_and_5(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .reset_hud_palettes_4_and_5();
    }

    fn decompress_story_gfx(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .attract_decompress_story_gfx();
    }

    fn palette_filter_history(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .palette_filter_history();
    }

    fn apply_palette_filter_bounce(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.apply_palette_filter_bounce());
    }

    fn reset_transition_props_and_advance_reset_interface(&mut self, ram: &mut Ram<'_>) {
        crate::dungeon::Dungeon::new(ram.bytes_mut(), self)
            .reset_transition_props_and_advance_reset_interface();
    }

    fn hud_refill_health(&mut self, ram: &mut Ram<'_>) -> bool {
        Hud { ram, runtime: self }.refill_health()
    }

    fn hud_refill_magic(&mut self, ram: &mut Ram<'_>) -> bool {
        Hud { ram, runtime: self }.refill_magic_power()
    }

    fn draw_selected_item(&mut self, ram: &mut Ram<'_>, item: u8) {
        let value = if matches!(item, 3 | 32) {
            1
        } else {
            ram.read_u8(0xf340 + usize::from(item))
        };
        let icon = Hud::<NativeGameServices<'_>>::item_box_icon(item, value);
        ram.write_u16(0x13c2, icon[0]);
        ram.write_u16(0x13c4, icon[1]);
        ram.write_u16(0x13ec, icon[2]);
        ram.write_u16(0x13ee, icon[3]);
    }

    fn world_map_load_light(&mut self, ram: &mut Ram<'_>) {
        crate::integration::setup_world_map_registers(ram);
        self.pending_host_actions
            .push(NativeHostAction::WorldMapLoadLight);
    }

    fn world_map_load_dark(&mut self, ram: &mut Ram<'_>) {
        if ram.read_u16(0x8a) & 0x40 != 0 {
            ram.slice_mut(0x1000, 1024)
                .copy_from_slice(&self.assets.bytes(68)[..1024]);
            ram.write_u8(0x17, 21);
        }
        ram.write_u8(0x200, ram.read_u8(0x200).wrapping_add(1));
    }

    fn world_map_load_sprite_gfx(&mut self, ram: &mut Ram<'_>) {
        ram.write_u8(0xaaa, 0x10);
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .graphics_load_chr_half_slot();
        ram.write_u8(0xaaa, 0);
        ram.write_u8(0x0200, ram.read_u8(0x0200).wrapping_add(1));
    }

    fn world_map_restore_gfx(&mut self, ram: &mut Ram<'_>) {
        let brightness = ram.read_u8(0x13).wrapping_sub(1);
        ram.write_u8(0x13, brightness);
        if brightness != 0 {
            return;
        }
        self.with_load_gfx(ram, |gfx| gfx.enable_force_blank());
        ram.write_u8(0x200, ram.read_u8(0x200).wrapping_add(1));
        let palette = ram.slice(0xc300, 512).to_vec();
        ram.slice_mut(0xc500, 512).copy_from_slice(&palette);
        ram.write_u16(0x99, ram.read_u16(0xc225));
        ram.write_u16(0xe4, 0);
        ram.write_u16(0xea, 0);
        for (dst, src) in [
            (0xe0, 0xc200),
            (0xe2, 0xc202),
            (0xe6, 0xc204),
            (0xe8, 0xc206),
        ] {
            ram.write_u16(dst, ram.read_u16(src));
        }
        ram.write_u16(0x1c, ram.read_u16(0xc211));
        self.pending_host_actions.push(NativeHostAction::HdmaSetup {
            addr6: 0x0a_bddd,
            addr7: 0x0a_bddd,
            transfer_unit: 0x42,
            reg6: 0x1b,
            reg7: 0x1e,
            indirect_bank: 0,
        });
        ram.write_u8(0x9b, 0x80);
        ram.write_u8(0x94, 9);
        ram.write_u8(0x710, 0);
    }

    fn world_map_exit(&mut self, ram: &mut Ram<'_>) {
        ram.write_u16(0xaa8, 0);
        ram.write_u8(0xab9, 0);
        self.with_load_gfx(ram, |gfx| gfx.initialize_tilesets());
        ram.write_u8(0x15, ram.read_u8(0x15).wrapping_add(1));
        ram.write_u8(0xb2, 0);
        ram.write_u8(0x200, 0);
        ram.write_u8(0xb0, 0);
        ram.write_u8(0x10, ram.read_u8(0x10c));
        ram.write_u8(0x11, 32);
        ram.write_u16(0x1000, 0);
        ram.write_u8(0x9b, ram.read_u8(0xc229));
        let music = ram.read_u8(0x15b00 + usize::from(ram.read_u8(0x8a)));
        ram.write_u8(0x12d, music >> 4);
        ram.write_u8(0x12f, 0x10);
        ram.write_u8(0x12c, 0xf3);
    }

    fn dungeon_map_step(&mut self, ram: &mut Ram<'_>, state: u8) {
        assert!(
            crate::native_messaging_services::dungeon_map_terminal_step(self, ram, state),
            "all non-terminal dungeon/world map steps are owned directly by Messaging"
        );
    }

    fn death_step(&mut self, ram: &mut Ram<'_>, state: u8) {
        crate::native_messaging_services::death_step(self, ram, state);
    }

    fn iris_spotlight_reset_table(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .iris_spotlight_reset_table();
    }

    fn flute_load_overlay(&mut self, ram: &mut Ram<'_>) {
        let module = ram.read_u16(0x10);
        let map_state = ram.read_u16(0x200);
        overworld::load_and_build_screen(ram, self.assets, |ram| {
            overworld::draw_quadrants_and_overlays(
                ram,
                self.assets,
                overworld::overworld_load_event_overlay,
            );
        });
        ram.write_u16(0x200, map_state.wrapping_add(1));
        ram.write_u16(0x10, module);
    }

    fn flute_load_selected_screen(&mut self, ram: &mut Ram<'_>) {
        ram.write_u8(0xf280 + 0x3b, ram.read_u8(0xf280 + 0x3b) & !0x20);
        ram.write_u8(0xf280 + 0x7b, ram.read_u8(0xf280 + 0x7b) & !0x20);
        ram.write_u16(0xf000 + 267 * 2, ram.read_u16(0xf000 + 267 * 2) & !0x80);
        ram.write_u16(0xf000 + 40 * 2, ram.read_u16(0xf000 + 40 * 2) & !0x100);
        let services = std::cell::RefCell::new(self);
        let assets = services.borrow().assets;
        if ram.read_u8(0xc8) & 0x40 == 0 {
            overworld::flute_menu_load_transport(ram, |ram, position| {
                overworld::load_bird_travel_position(
                    ram,
                    assets,
                    usize::from(position),
                    |ram| overworld::load_gfx_and_screen_size(ram, assets),
                    |ram| {
                        crate::sprite::Sprite::new(ram.bytes_mut(), &mut **services.borrow_mut())
                            .sprite_reset_all();
                    },
                    |ram| {
                        PreOverworldHooks::reload_overworld_sprites(
                            &mut **services.borrow_mut(),
                            ram,
                        );
                    },
                    |ram| {
                        TransitionHooks::reset_torch_background_and_player(
                            &mut **services.borrow_mut(),
                            ram,
                        );
                    },
                );
            });
        }
        overworld::flute_menu_load_selected_screen_palettes(
            ram,
            assets,
            |ram| {
                services
                    .borrow_mut()
                    .with_load_gfx(ram, |gfx| gfx.overworld_load_screens_palette_set());
            },
            |ram, bg, sprites| {
                services
                    .borrow_mut()
                    .with_load_gfx(ram, |gfx| gfx.overworld_load_palettes(bg, sprites));
            },
            |ram| {
                services
                    .borrow_mut()
                    .with_load_gfx(ram, |gfx| gfx.palette_set_ow_bg_color());
            },
            |ram| {
                services
                    .borrow_mut()
                    .with_load_gfx(ram, |gfx| gfx.overworld_load_palettes_inner());
            },
        );
        let screen = ram.read_u8(0x8a) & 0xbf;
        services.borrow_mut().with_load_gfx(ram, |gfx| {
            gfx.decompress_animated_overworld_tiles(if matches!(screen, 3 | 5 | 7) {
                0x58
            } else {
                0x5a
            });
            gfx.initialize_tilesets();
        });
        overworld::overworld_set_fixed_col_and_scroll(ram);
        ram.write_u16(0xaa8, 0);
        ram.write_u8(0xab9, 0);
        ram.write_u8(0x200, ram.read_u8(0x200).wrapping_add(1));
        ram.write_u8(0xb2, 0);
        overworld::overworld_load_overlays2(ram, assets);
        ram.write_u8(0x11, ram.read_u8(0x11).wrapping_sub(1));
        ram.write_u8(0x12f, 16);
        let music = ram.read_u8(0x15b00 + usize::from(ram.read_u8(0x8a)));
        ram.write_u8(0x12d, music >> 4);
        let playing =
            OverworldServices::is_playing_music_track(&mut **services.borrow_mut(), music & 0x0f);
        ram.write_u8(0x12c, if playing { 0xf3 } else { music & 0x0f });
    }

    fn flute_finish(&mut self, ram: &mut Ram<'_>) {
        crate::ancilla::Ancilla { ram, runtime: self }.add_bird_travel_something(0x27, 4);
        UnifiedGameplayServices::run_sprite_main(self, ram);
    }

    fn flute_sprite_main(&mut self, ram: &mut Ram<'_>) {
        UnifiedGameplayServices::run_sprite_main(self, ram);
    }

    fn transfer_mode7_characters(&mut self, _ram: &mut Ram<'_>) {
        self.pending_host_actions
            .push(NativeHostAction::TransferMode7Characters);
    }

    fn overworld_map_setup_hdma(&mut self, _ram: &mut Ram<'_>, _zoomed: bool) {
        self.pending_host_actions
            .push(NativeHostAction::WorldMapSetupHdma);
    }

    fn desert_prayer_setup_hdma(&mut self, ram: &mut Ram<'_>) {
        self.pending_host_actions.push(NativeHostAction::HdmaSetup {
            addr6: 0,
            addr7: 0x02_c80c,
            transfer_unit: 0x41,
            reg6: 0,
            reg7: 0x26,
            indirect_bank: 0,
        });
        ram.write_u8(0x96, 0x33);
        ram.write_u8(0x97, 3);
        ram.write_u8(0x98, 0x33);
        ram.write_u8(0x1e, ram.read_u8(0x1c));
        ram.write_u8(0x1f, ram.read_u8(0x1d));
        ram.write_u8(0x9b, 0x80);
        ram.slice_mut(0x1dba0, 240 * 2).fill(0);
    }

    fn dungeon_map_prepare_graphics(&mut self, ram: &mut Ram<'_>) {
        let hdma = ram.read_u8(0x9b);
        ram.write_u8(0x9b, 0);
        ram.write_u8(0xc20e, ram.read_u8(0xaa1));
        ram.write_u8(0xc20f, ram.read_u8(0xaa3));
        ram.write_u8(0xc210, ram.read_u8(0xaa2));
        ram.write_u8(0xc211, ram.read_u8(0x1c));
        ram.write_u8(0xc212, ram.read_u8(0x1d));
        ram.write_u8(0xaa1, 32);
        ram.write_u8(0xaa3, 0x80 | (ram.read_u16(0x40c) >> 1) as u8);
        ram.write_u8(0xaa2, 64);
        ram.write_u8(0x1c, 0x16);
        ram.write_u8(0x1d, 1);
        self.with_load_gfx(ram, |gfx| {
            gfx.erase_tile_maps_dungeonmap();
            gfx.initialize_tilesets();
        });
        ram.write_u16(0xaa8, 0x200);
        self.with_load_gfx(ram, |gfx| {
            gfx.palette_load_dungeon_map_bg();
            gfx.palette_load_dungeon_map_sprite();
        });
        ram.write_u8(0xab2, 1);
        self.with_load_gfx(ram, |gfx| {
            gfx.palette_load_hud();
            gfx.load_actual_gear_palettes();
        });
        ram.write_u8(0x15, ram.read_u8(0x15).wrapping_add(1));
        ram.write_u8(0x20d, ram.read_u8(0x20d).wrapping_add(1));
        ram.write_u8(0x9b, hdma);
        ram.write_u8(0x14, 9);
        ram.write_u8(0x710, 9);
    }

    fn dungeon_map_recover_graphics(&mut self, ram: &mut Ram<'_>) {
        crate::native_messaging_services::dungeon_map_recover_graphics(self, ram);
    }

    fn interface_gameplay_step(&mut self, ram: &mut Ram<'_>, run_world: bool) {
        if ram.read_u8(0x1b) != 0 && ram.read_u8(0x11) != 3 {
            crate::dungeon::Dungeon::new(ram.bytes_mut(), self).dungeon_push_block_handler();
        }
        if !run_world {
            return;
        }
        UnifiedGameplayServices::run_sprite_main(self, ram);
        UnifiedGameplayServices::run_link_oam_main(self, ram);
        if ram.read_u8(0x1b) == 0 {
            overworld::overworld_overlay_handle_rain(ram);
        }
        Hud { ram, runtime: self }.refill_logic();
        if ram.read_u8(0x11) != 2 {
            DungeonHooks::orient_lamp_light_cone(self, ram);
        }
    }

    fn display_select_menu(&mut self, ram: &mut Ram<'_>) {
        HudRuntime::display_select_menu(self, ram);
    }

    fn link_reset_properties_c(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_reset_properties_c();
    }

    fn death_finalize_choice(&mut self, ram: &mut Ram<'_>, sram: &mut [u8], count_as_death: bool) {
        // Death_Func15 is also reached by Module17_SaveAndQuit. Calling back
        // into Messaging::death_func15 here would recurse indefinitely.
        ram.write_u8(0x12c, 0xf1);
        if ram.read_u8(0x1b) != 0 {
            crate::dungeon::Dungeon::new(ram.bytes_mut(), self).dungeon_flag_room_data_quadrants();
        }
        overworld::adjust_link_bunny_status(ram);
        if ram.read_u8(0xf3c5) < 3 {
            ram.write_u8(0xf3ca, 0);
            if ram.read_u8(0xf357) == 0 {
                overworld::force_nonbunny_status(ram);
            }
        }
        if ram.read_u16(0xa0) == 0 {
            ram.write_u8(0x1b, 0);
        }
        let room = ram.read_u8(0xa0);
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .reset_some_things_after_death(room);
        if matches!(ram.read_u8(0xf3cc), 6 | 9 | 10 | 13) {
            ram.write_u8(0xf3cc, 0);
        }
        let health = crate::messaging::HEALTH_AFTER_DEATH[usize::from(ram.read_u8(0xf36c) >> 3)];
        ram.write_u8(0x4aa, health);
        ram.write_u8(0xf36d, health);
        let palace = ram.read_u8(0x40c);
        if palace != 0xff {
            ram.write_u8(
                0xf37c + usize::from(if palace == 2 { 0 } else { palace } >> 1),
                ram.read_u8(0xf36f),
            );
        }
        crate::sprite::Sprite::new(ram.bytes_mut(), self).sprite_reset_all();
        if ram.read_u16(0xf405) == 0xffff
            && (self.enhanced_features0 & 0x1000 == 0 || count_as_death)
        {
            ram.write_u16(0xf403, ram.read_u16(0xf403).wrapping_add(1));
        }
        ram.write_u8(0x10a, ram.read_u8(0x10a).wrapping_add(1));
        let messages: [&[u8]; 0] = [];
        let dictionary: [&[u8]; 0] = [];
        let assets = MessagingAssets {
            dialogue_flags: 0,
            dictionary: &dictionary,
            messages: &messages,
            font_tiles: &[],
            font_widths: &[],
            light_overworld_tilemap: &[],
            dungeon_map_floor_layout: &[],
            dungeon_map_info: &[],
        };
        if ram.read_u8(0xb0) != 1 {
            if ram.read_u8(0x1b) != 0 {
                if ram.read_u8(0xf3cc) != 1 && palace != 0xff {
                    ram.write_u8(0x4aa, 0);
                } else {
                    ram.write_u8(0x132, 0);
                    ram.write_u8(0x1b, 0);
                }
            }
            if ram.read_u8(0x1b) == 0 && ram.read_u8(0xf3ca) != 0 {
                ram.write_u16(0xa0, 32);
            }
            if ram.read_u8(0xf3c5) != 0 {
                if ram.read_u8(0xb0) == 0 {
                    Messaging {
                        ram,
                        sram,
                        assets: &assets,
                        runtime: self,
                    }
                    .save_game_file();
                }
                ram.write_u8(0x10, 5);
                ram.write_u8(0x11, 0);
                ram.write_u8(0x14, 0);
            } else {
                let slot = u16::from_le_bytes([sram[0x1ffe], sram[0x1fff]]);
                let offset = usize::from((slot >> 1).wrapping_sub(1)) * 0x500;
                ram.write_u16(0, offset as u16);
                ram.write_u8(0x10a, 0);
                Messaging {
                    ram,
                    sram,
                    assets: &assets,
                    runtime: self,
                }
                .copy_save_to_wram(offset);
            }
        } else {
            if ram.read_u8(0xf3c5) != 0 {
                Messaging {
                    ram,
                    sram,
                    assets: &assets,
                    runtime: self,
                }
                .save_game_file();
            }
            ram.write_u8(0x1c, 16);
            ram.write_u8(0x1b, 0);
            ram.write_u8(0x710, ram.read_u8(0x710).wrapping_add(1));
            AttractRuntime::intro_initialize_memory_darken(self, ram);
            AttractRuntime::overworld_load_all_palettes(self, ram);
            ram.write_u8(0xea, 0);
            for address in [0x63a, 0x638, 0x120, 0x124, 0x11e, 0x122] {
                ram.write_u16(address, 0);
            }
            ram.write_u8(0x12c, 0xf1);
            ram.write_u8(0x23, 0);
            ram.write_u8(0x10, 0);
            ram.write_u8(0x11, 10);
            ram.write_u8(0xb0, 10);
            ram.write_u8(0x4aa, 0);
            ram.write_u8(0x10a, 0);
            ram.write_u8(0x132, 0);
            for address in [0xe0, 0xe2, 0xe4, 0xe6, 0xe8, 0xea] {
                ram.write_u16(address, 0);
            }
            for address in [0x120, 0x11e, 0x124, 0x122] {
                ram.write_u16(address, 0);
            }
            ram.slice_mut(0xf000, 0x500).fill(0);
            ram.write_u8(0x136, 0);
            crate::misc::load_overworld_songs(ram, self);
        }
    }

    fn load_actual_gear_palettes(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .load_actual_gear_palettes();
    }

    fn calculate_link_sfx_pan(&mut self, ram: &mut Ram<'_>) -> u8 {
        crate::misc::link_calculate_sfx_pan(ram)
    }
}
impl AttractRuntime for NativeGameServices<'_> {
    fn intro_handle_all_triforce_animations(&mut self, ram: &mut Ram<'_>) {
        Ending { ram, runtime: self }.intro_handle_all_triforce_animations();
    }

    fn intro_periodic_sword_and_intro_flash(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.palette_bg_and_fixed_color_black());
        let features = self.enhanced_features0;
        Ending { ram, runtime: self }.intro_periodic_sword_and_intro_flash(features);
    }

    fn force_blank(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .enable_force_blank();
    }

    fn erase_tile_maps(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .erase_tile_maps_normal();
    }

    fn load_bg3_gfx(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .attract_load_bg3_gfx();
    }

    fn palette_load_hud(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.palette_load_hud());
    }

    fn palette_load_ow_bg_main(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.palette_load_owbg_main());
    }

    fn palette_load_link_armor_and_gloves(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.palette_load_link_armor_and_gloves());
    }

    fn hdma_setup(
        &mut self,
        dma: &mut z3_snes::dma::Dma,
        _ram: &mut Ram<'_>,
        args: crate::attract::HdmaSetupArgs,
    ) {
        crate::runtime::hdma_setup(
            dma,
            args.table,
            args.indirect,
            args.mode,
            args.first_register,
            args.second_register,
            args.flags,
        );
    }

    fn world_map_load_light_world_map(
        &mut self,
        dma: &mut z3_snes::dma::Dma,
        ppu: &mut z3_snes::ppu::Ppu,
        ram: &mut Ram<'_>,
    ) {
        let map_graphics = self.assets.bytes(66);
        for (word, &high) in ppu.vram[..0x4000].iter_mut().zip(map_graphics) {
            *word = u16::from_le_bytes([0xef, high]);
        }
        ram.write_u8(0x1c, 0x11);
        ram.write_u8(0x1d, 0);
        ram.write_u16(0xe0, 0x80);
        ram.write_u16(0xe6, 0xc8);
        ram.write_u16(0x63a, 0x1c9);
        ram.write_u16(0x638, 0x100);
        ram.write_u8(0x23, 0);
        ram.write_u8(0x24, 0);
        ram.write_u8(0x25, 0);
        crate::integration::setup_world_map_hdma_transfer(ram, dma);
        ram.write_u8(0x9b, 0xc0);
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: &mut ppu.vram,
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .load_overworld_map_palette();
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: &mut ppu.vram,
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .load_actual_gear_palettes();
        ram.write_u8(0x15, ram.read_u8(0x15).wrapping_add(1));
        ram.write_u8(0x17, 7);
        ram.write_u8(0x13, 0);
        ram.write_u8(0x0710, ram.read_u8(0x0710).wrapping_add(1));
        ram.write_u8(0x0200, ram.read_u8(0x0200).wrapping_add(1));
    }

    fn load_common_sprites(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.load_common_sprites());
    }

    fn dungeon_load_entrance(&mut self, ram: &mut Ram<'_>) {
        crate::dungeon::Dungeon::new(ram.bytes_mut(), self).dungeon_load_entrance();
    }

    fn dungeon_load_and_draw_room(&mut self, ram: &mut Ram<'_>) {
        crate::dungeon::Dungeon::new(ram.bytes_mut(), self).dungeon_load_and_draw_room();
    }

    fn dungeon_reset_torch_background_and_player(&mut self, ram: &mut Ram<'_>) {
        crate::dungeon::Dungeon::new(ram.bytes_mut(), self)
            .dungeon_reset_torch_background_and_player_inner();
    }

    fn initialize_tilesets(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.initialize_tilesets());
    }

    fn palette_bg_and_fixed_color_black(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.palette_bg_and_fixed_color_black());
    }

    fn palette_load_sp0l(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.palette_load_sp0l());
    }

    fn palette_load_sprite_main(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.palette_load_sprite_main());
    }

    fn palette_load_sp5l(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.palette_load_sp5l());
    }

    fn palette_load_sp6l(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.palette_load_sp6l());
    }

    fn palette_load_sprite_environment_dungeon(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| {
            gfx.palette_load_sprite_environment_dungeon();
        });
    }

    fn palette_load_dungeon_set(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.palette_load_dungeon_set());
    }

    fn setup_conclusion_hdma(&mut self, dma: &mut z3_snes::dma::Dma, ram: &mut Ram<'_>) {
        crate::runtime::hdma_setup(dma, 0x0a_bddd, 0x0a_bddd, 0x42, 0x1b, 0x1e, 0);
        ram.write_u8(0x9b, 0x80);
        ram.write_u8(0x94, 9);
        ram.write_u8(0x0710, 0);
    }

    fn overworld_load_all_palettes(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.overworld_load_all_palettes());
    }

    fn render_text(&mut self, ram: &mut Ram<'_>, sram: &mut [u8], assets: &MessagingAssets<'_>) {
        Messaging {
            ram,
            sram,
            assets,
            runtime: self,
        }
        .render_text();
    }

    fn intro_initialize_memory_darken(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| {
            gfx.enable_force_blank();
            gfx.erase_tile_maps_normal();
        });
        ram.write_u8(0xaa1, 35);
        ram.write_u8(0xaa3, 125);
        ram.write_u8(0xaa2, 81);
        ram.write_u8(0xaa4, 8);
        self.with_load_gfx(ram, |gfx| {
            gfx.load_default_graphics();
            gfx.initialize_tilesets();
            gfx.decompress_animated_dungeon_tiles(0x5d);
        });
        ram.write_u16(0xc00d, 2);
        ram.write_u8(0x8a, 0);
        ram.write_u8(0xab6, 0);
        ram.write_u8(0xab8, 0);
        ram.write_u16(0xc8, 0);
        ram.write_u16(0xca, 0);
        ram.write_u16(0xc009, 2);
        ram.write_u16(0xc007, 31);
        ram.write_u8(0xc00b, 0);
        ram.write_u8(0x11, ram.read_u8(0x11).wrapping_add(1));
    }

    fn handle_screen_flash(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.handle_screen_flash());
    }

    fn fade_music_and_reset_sram_mirror(&mut self, ram: &mut Ram<'_>) {
        ram.write_u8(0x12, 0xff);
        ram.write_u8(0x1c, 0x15);
        ram.write_u8(0x1d, 0);
        ram.write_u8(0x1b, 0);
        ram.write_u8(0x12c, 0xf1);
        self.with_load_gfx(ram, |gfx| gfx.palette_bg_and_fixed_color_black());
        ram.slice_mut(0x20, 0x70).fill(0);
        ram.slice_mut(0xf000, 0x500).fill(0);
        ram.write_u8(0x10, 1);
        ram.write_u8(0x4aa, 1);
        ram.write_u8(0x11, 0);
    }

    fn sprite_prep_reset_properties(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite::Sprite::new(ram.bytes_mut(), self).sprite_prep_reset_properties(k);
    }

    fn guard_handle_all_animation(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank0::SpriteMainBank0 { ram, runtime: self }
            .guard_handle_all_animation(k);
    }
}
impl EndingRuntime for NativeGameServices<'_> {
    fn call(&mut self, name: &'static str, ram: &mut Ram<'_>) {
        match name {
            "ApplyPaletteFilter_bounce" => {
                self.with_load_gfx(ram, |gfx| gfx.apply_palette_filter_bounce());
            }
            "CallForDuckIndoors" => {
                crate::ancilla::Ancilla { ram, runtime: self }.call_for_duck_indoors();
            }
            "ConditionalMosaicControl" => overworld::conditional_mosaic_control(ram),
            "Dungeon_HandleLayerEffect" => {
                crate::dungeon::Dungeon::new(ram.bytes_mut(), self).dungeon_handle_layer_effect();
            }
            "Dungeon_LoadAndDrawRoom" => {
                crate::dungeon::Dungeon::new(ram.bytes_mut(), self).dungeon_load_and_draw_room();
            }
            "Dungeon_LoadEntrance" => {
                crate::dungeon::Dungeon::new(ram.bytes_mut(), self).dungeon_load_entrance();
            }
            "Dungeon_LoadPalettes" => {
                self.with_load_gfx(ram, |gfx| gfx.dungeon_load_palettes());
            }
            "Hud_RebuildIndoor" => Hud { ram, runtime: self }.rebuild_indoor(),
            "Link_HandleMovingAnimation_FullLongEntry" => {
                crate::player::Player {
                    ram,
                    runtime: self,
                    applied_movement_to_camera: false,
                }
                .link_handle_moving_animation_full_long_entry();
            }
            "Link_HandleVelocity" => {
                crate::player::Player {
                    ram,
                    runtime: self,
                    applied_movement_to_camera: false,
                }
                .link_handle_velocity();
            }
            "Link_ResetProperties_A" => {
                crate::player::Player {
                    ram,
                    runtime: self,
                    applied_movement_to_camera: false,
                }
                .link_reset_properties_a();
            }
            "Main_ShowTextMessage" => crate::misc::main_show_text_message(ram),
            "Module08_02_LoadAndAdvance" => {
                overworld::module08_load_and_advance(ram, self.assets, |ram| {
                    overworld::draw_quadrants_and_overlays(
                        ram,
                        self.assets,
                        overworld::overworld_load_event_overlay,
                    );
                });
            }
            "OverworldHandleMapScroll" => overworld::handle_map_scroll(ram, self.assets),
            "Overworld_LoadAndBuildScreen" => {
                overworld::load_and_build_screen(ram, self.assets, |ram| {
                    overworld::draw_quadrants_and_overlays(
                        ram,
                        self.assets,
                        overworld::overworld_load_event_overlay,
                    );
                });
            }
            "Overworld_LoadOverlayAndMap" => {
                let module = ram.read_u16(0x10);
                let map_state = ram.read_u16(0x200);
                overworld::load_and_build_screen(ram, self.assets, |ram| {
                    overworld::draw_quadrants_and_overlays(
                        ram,
                        self.assets,
                        overworld::overworld_load_event_overlay,
                    );
                });
                ram.write_u16(0x200, map_state.wrapping_add(1));
                ram.write_u16(0x10, module);
            }
            "Overworld_LoadOverlays2" => overworld::overworld_load_overlays2(ram, self.assets),
            "Overworld_SetFixedColAndScroll" => overworld::overworld_set_fixed_col_and_scroll(ram),
            "Palette_AnimGetMasterSword2" => overworld::palette_anim_get_master_sword2(ram),
            "ResetTransitionPropsAndAdvance_ResetInterface" => {
                crate::dungeon::Dungeon::new(ram.bytes_mut(), self)
                    .reset_transition_props_and_advance_reset_interface();
            }
            "SaveDungeonKeys" => {
                crate::dungeon::Dungeon::new(ram.bytes_mut(), self).save_dungeon_keys();
            }
            "Sprite_SpawnBatCrashCutscene" => {
                crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }
                    .sprite_spawn_bat_crash_cutscene();
            }
            "TriforceRoom_LinkApproachTriforce" => {
                crate::misc::triforce_room_link_approach_triforce(ram);
            }
            _ => self
                .pending_host_actions
                .push(NativeHostAction::EndingExternal(name)),
        }
    }

    fn sprite_reset(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite::Sprite::new(ram.bytes_mut(), self).sprite_prep_reset_properties(k);
    }

    fn sprite_move_xy(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite::Sprite::new(ram.bytes_mut(), self).sprite_move_xy(k);
    }

    fn sprite_active_main(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite_main_bank1::SpriteMainBank1 { ram, runtime: self }.sprite_active_main(k);
    }

    fn sprite_draw_shadow(&mut self, ram: &mut Ram<'_>, k: usize) {
        let mut info = self.ending_sprite_oam;
        let mut sprite = crate::sprite::Sprite::new(ram.bytes_mut(), self);
        sprite.sprite_draw_shadow(k, &mut info);
        self.ending_sprite_oam = info;
    }

    fn oam_allocate_region_a(&mut self, ram: &mut Ram<'_>, bytes: u8) {
        crate::sprite_main_bank2a::SpriteMainBank2aRuntime::oam_allocate_region_a(self, ram, bytes);
    }

    fn sprite_get_16_bit_coords(&mut self, ram: &mut Ram<'_>, k: usize) {
        crate::sprite::Sprite::new(ram.bytes_mut(), self).sprite_get_16_bit_coords(k);
    }

    fn sprite_draw_multiple(
        &mut self,
        ram: &mut Ram<'_>,
        k: usize,
        entries: &[crate::ending::DrawMultipleData],
    ) {
        let entries = entries
            .iter()
            .map(|entry| crate::sprite::DrawMultipleData {
                x: entry.x,
                y: entry.y,
                char_flags: entry.tile,
                ext: entry.ext,
            })
            .collect::<Vec<_>>();
        let mut info = crate::sprite::PrepOamCoordsRet::default();
        let mut sprite = crate::sprite::Sprite::new(ram.bytes_mut(), self);
        sprite.sprite_get_16_bit_coords(k);
        sprite.sprite_draw_multiple(k, &entries, Some(&mut info));
        self.ending_sprite_oam = info;
    }

    fn ending_map_data(&self) -> &[u16] {
        self.assets.words(76)
    }

    fn ending_credits_text(&self) -> &[u8] {
        self.assets.bytes(74)
    }

    fn ending_credits_offsets(&self) -> &[u16] {
        self.assets.words(75)
    }

    fn ending_scene_text(&self) -> &[u8] {
        self.assets.bytes(78)
    }

    fn ending_scene_offsets(&self) -> &[u16] {
        self.assets.words(77)
    }

    fn random_number(&mut self, ram: &mut Ram<'_>) -> u8 {
        crate::misc::get_random_number(ram)
    }

    fn generated_end_sequence_15(&self, frame: u8) -> u8 {
        self.assets.bytes(73)[usize::from(frame)]
    }

    fn decompress_animated_overworld_tiles(&mut self, ram: &mut Ram<'_>, set: u8) {
        self.with_load_gfx(ram, |gfx| gfx.decompress_animated_overworld_tiles(set));
    }

    fn get_overworld_bg_palette(&mut self, screen: u8) -> u8 {
        overworld::get_overworld_bg_palette(self.assets, usize::from(screen))
    }

    fn overworld_load_palettes(&mut self, ram: &mut Ram<'_>, bg: u8, sprites: u8) {
        self.with_load_gfx(ram, |gfx| gfx.overworld_load_palettes(bg, sprites));
    }

    fn decompress_animated_dungeon_tiles(&mut self, ram: &mut Ram<'_>, set: u8) {
        self.with_load_gfx(ram, |gfx| gfx.decompress_animated_dungeon_tiles(set));
    }

    fn dungeon_palette_info(&mut self, index: u8) -> (u8, u8) {
        let info = crate::dungeon::Dungeon::<NativeGameServices<'_>>::get_dung_pal_info(
            usize::from(index),
        );
        (info.pal2, info.pal3)
    }

    fn health_after_death(&self, capacity_index: usize) -> u8 {
        crate::messaging::HEALTH_AFTER_DEATH[capacity_index]
    }

    fn vram_write_word(&mut self, word_address: usize, value: u16) {
        self.scratch_vram[word_address] = value;
    }

    fn dungeon_animated_tile_for_theme(&self, theme: u8) -> u8 {
        crate::dungeon::DUNG_ANIMATED_TILES[usize::from(theme)]
    }

    fn overworld_load_area_palettes_ex(&mut self, ram: &mut Ram<'_>, index: u8) {
        self.with_load_gfx(ram, |gfx| gfx.overworld_load_area_palettes_ex(index));
    }
}

impl DungeonHooks for NativeGameServices<'_> {
    fn dungeon_draw_pushable_block(&mut self, ram: &mut Ram<'_>, tilemap: u16, object: u16) {
        crate::dungeon::Dungeon::new(ram.bytes_mut(), self)
            .draw_objects_pushable_block(tilemap, object);
    }

    fn dungeon_draw_lightable_torch(&mut self, ram: &mut Ram<'_>, descriptor: u16, object: u16) {
        crate::dungeon::Dungeon::new(ram.bytes_mut(), self)
            .draw_objects_lightable_torch(descriptor, object);
    }

    fn dungeon_movable_block(&self, index: usize) -> (u16, u16) {
        self.assets.dungeon_movable_block(index)
    }

    fn dungeon_movable_init_data(&self) -> &[u8] {
        self.assets.bytes(53)
    }

    fn dungeon_torch_init_data(&self) -> &[u8] {
        self.assets.bytes(54)
    }

    fn dungeon_torch_junk_data(&self) -> &[u8] {
        self.assets.bytes(55)
    }

    fn dungeon_torch_data(&self, index: usize) -> u16 {
        self.assets.dungeon_torch_data(index)
    }

    fn dungeon_entrance(
        &self,
        starting_point: bool,
        index: usize,
    ) -> crate::dungeon::DungeonEntrance {
        self.assets.dungeon_entrance(starting_point, index)
    }

    fn dungeon_transition_aux_gfx_sprite(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.load_trans_aux_gfx_sprite());
    }
    fn dungeon_transition_aux_gfx(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.load_trans_aux_gfx());
    }
    fn dungeon_nmi_upload_tilemap(&mut self, ram: &mut Ram<'_>) {
        const ADDRS: [usize; 35] = [
            0, 0, 4, 8, 12, 8, 12, 0, 4, 0, 8, 4, 12, 4, 12, 0, 8, 16, 20, 24, 28, 24, 28, 16, 20,
            16, 24, 20, 28, 20, 28, 16, 24, 96, 104,
        ];
        let dst = ADDRS[usize::from(ram.read_u8(0x116))] << 8;
        for (i, bytes) in ram.slice(0x1000, 0x800).chunks_exact(2).enumerate() {
            self.scratch_vram[dst + i] = u16::from_le_bytes([bytes[0], bytes[1]]);
        }
        ram.write_u16(0x1000, 0);
        ram.write_u8(0x710, 0);
    }
    fn dungeon_entrance_music(&self, entrance: usize) -> u8 {
        self.assets.bytes(27)[entrance]
    }
    fn dungeon_starting_music(&self, starting_point: usize) -> u8 {
        self.assets.bytes(45)[starting_point]
    }
    fn dungeon_entrance_music_track(&self, entrance: usize) -> u8 {
        self.assets.bytes(27)[entrance]
    }
    fn hud_rebuild(&mut self, ram: &mut Ram<'_>) {
        Hud { ram, runtime: self }.rebuild();
    }
    fn hud_refill_logic(&mut self, ram: &mut Ram<'_>) {
        Hud { ram, runtime: self }.refill_logic();
    }
    fn hud_floor_indicator(&mut self, ram: &mut Ram<'_>) {
        Hud { ram, runtime: self }.floor_indicator();
    }
    fn initialize_tilesets(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.initialize_tilesets());
    }
    fn dungeon_load_palettes(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.dungeon_load_palettes());
    }
    fn decompress_animated_dungeon_tiles(&mut self, ram: &mut Ram<'_>, set: u8) {
        self.with_load_gfx(ram, |g| g.decompress_animated_dungeon_tiles(set));
    }
    fn load_gear_palettes_bunny(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.load_gear_palettes_bunny());
    }
    fn hud_remove_super_bomb_indicator(&mut self, ram: &mut Ram<'_>) {
        Hud { ram, runtime: self }.remove_super_bomb_indicator();
    }
    fn link_tuck_into_bed(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_tuck_into_bed();
    }
    fn enable_force_blank(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.enable_force_blank());
    }
    fn sprite_repel_dash(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .sprite_repel_dash();
    }
    fn ancilla_add_door_debris(&mut self, ram: &mut Ram<'_>) -> i32 {
        crate::ancilla::Ancilla { ram, runtime: self }
            .add_door_debris()
            .map_or(-1, |k| k as i32)
    }
    fn overworld_bomb_tiles_32x32(&mut self, ram: &mut Ram<'_>, x: u16, y: u16) {
        let assets = self.assets;
        let services = std::cell::RefCell::new(self);
        overworld::bomb_tiles_32x32(ram, assets, x, y, |ram, kind, x, y| {
            crate::sprite::Sprite::new(ram.bytes_mut(), &mut **services.borrow_mut())
                .sprite_spawn_immediately_smashed_terrain(kind, x, y);
        });
    }
    fn dungeon_calculate_sfx_pan(&self, ram: &Ram<'_>, x: u16) -> u8 {
        crate::misc::calculate_sfx_pan_arbitrary(ram, x as u8)
    }

    fn dungeon_object_data(&self, word_offset: usize) -> &[u16] {
        self.assets.dungeon_object_data(word_offset)
    }

    fn dungeon_tele_message(&self, room: usize) -> u16 {
        self.assets.words(9)[room]
    }
    fn dungeon_pits_hurt_player(&self) -> &[u16] {
        self.assets.words(10)
    }
    fn dungeon_tile_attribute(&self, ram: &Ram<'_>, tile: usize) -> u8 {
        ram.read_u8(0xfe00 + tile)
    }

    fn dungeon_room_door_info(&self, room: usize) -> &[u16] {
        self.assets.dungeon_room_door_info(room)
    }

    fn dungeon_room_header(&self, room: usize) -> &[u8] {
        self.assets.dungeon_room_header(room)
    }

    fn dungeon_room_chests(&self) -> &[u8] {
        self.assets.dungeon_room_chests()
    }

    fn dungeon_secrets(&self) -> &[u8] {
        self.assets.dungeon_secrets()
    }

    fn default_room_layout(&self, layout: usize) -> &[u8] {
        self.assets.default_room_layout(layout)
    }

    fn dungeon_room_layout(&self, room: usize) -> &[u8] {
        self.assets.dungeon_room_layout(room)
    }

    fn dungeon_restore_star_tile_chr(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.dungeon_restore_star_tile_chr());
    }
    fn cache_camera_properties(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .cache_camera_properties();
    }
    fn prep_trans_aux_gfx(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.prep_trans_aux_gfx());
    }
    fn iris_spotlight_reset_table(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.iris_spotlight_reset_table());
    }
    fn dungeon_prep_overlay_dma_next(
        &mut self,
        ram: &mut Ram<'_>,
        dst: usize,
        source: u16,
    ) -> usize {
        crate::misc::dungeon_prep_overlay_dma_next_prep(ram, dst, source)
    }
    fn dungeon_prep_overlay_dma_watergate(
        &mut self,
        ram: &mut Ram<'_>,
        dst: usize,
        source: u16,
        mode: u16,
        loops: usize,
    ) -> usize {
        crate::misc::dungeon_prep_overlay_dma_watergate(ram, dst, source, mode, loops)
    }
    fn sprite_check_if_screen_is_clear(&self, ram: &Ram<'_>) -> bool {
        (0..16).all(|i| {
            let x = u16::from(ram.read_u8(0xd10 + i)) | u16::from(ram.read_u8(0xd30 + i)) << 8;
            let y = u16::from(ram.read_u8(0xd00 + i)) | u16::from(ram.read_u8(0xd20 + i)) << 8;
            ram.read_u8(0xdd0 + i) == 0
                || ram.read_u8(0xf60 + i) & 0x40 != 0
                || x.wrapping_sub(ram.read_u16(0xe2)) >= 256
                || y.wrapping_sub(ram.read_u16(0xe8)) >= 256
        }) && (0..8).all(|i| !matches!(ram.read_u8(0xb00 + i), 0x14 | 0x18))
    }
    fn sprite_check_if_room_is_clear(&self, ram: &Ram<'_>) -> bool {
        (0..16).all(|i| ram.read_u8(0xdd0 + i) == 0 || ram.read_u8(0xf60 + i) & 0x40 != 0)
            && (0..8).all(|i| !matches!(ram.read_u8(0xb00 + i), 0x14 | 0x18))
    }
    fn load_overworld_songs(&mut self, ram: &mut Ram<'_>) {
        crate::misc::load_overworld_songs(ram, self);
    }
    fn load_dungeon_songs(&mut self, ram: &mut Ram<'_>) {
        crate::misc::load_dungeon_songs(ram, self);
    }
    fn apply_palette_filter_bounce(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.apply_palette_filter_bounce());
    }
    fn dungeon_handle_translucency_and_palette(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.dungeon_handle_translucency_and_palette());
    }
    fn hud_restore_torch_background(&mut self, ram: &mut Ram<'_>) {
        Hud { ram, runtime: self }.restore_torch_background();
    }
    fn ancilla_terminate_select_interactives(&mut self, ram: &mut Ram<'_>, kind: u8) -> u8 {
        crate::ancilla::Ancilla { ram, runtime: self }.terminate_select_interactives(kind)
    }
    fn turn_while_dashing_enabled(&self) -> bool {
        self.enhanced_features0 & 4 != 0
    }
    fn overworld_copy_palettes_to_cache(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.overworld_copy_palettes_to_cache());
    }
    fn orient_lamp_light_cone(&mut self, ram: &mut Ram<'_>) {
        if ram.read_u8(0x458) == 0 || ram.read_u8(0x11) == 20 {
            return;
        }
        const T0: [u16; 4] = [0, 256, 0, 256];
        const T1: [u16; 4] = [0, 0, 256, 256];
        const T2: [i16; 4] = [52, -2, 56, 6];
        const T3: [i16; 4] = [64, 64, 82, -176];
        const T4: [i16; 4] = [128, 384, 160, 160];
        let a = usize::from(ram.read_u8(0x2f) >> 1);
        let doorway = ram.read_u8(0x6c);
        let mut idx = a;
        if doorway != 0 {
            idx = usize::from(doorway & 0xfe);
            if idx != 0 {
                if a < 2 {
                    idx += usize::from(ram.read_u8(0x22).wrapping_add(8) >= 0x80);
                } else {
                    idx = a;
                }
            } else if a >= 2 {
                idx += usize::from(ram.read_u8(0x20) >= 0x80);
            } else {
                idx = a;
            }
        }
        if idx < 2 {
            ram.write_u16(
                0xe0,
                ram.read_u16(0xe2)
                    .wrapping_sub(ram.read_u16(0x22).wrapping_sub(0x77))
                    .wrapping_add(T0[idx]),
            );
            let v = ram
                .read_u16(0xe8)
                .wrapping_sub(ram.read_u16(0x20).wrapping_sub(0x58))
                .wrapping_add(T1[idx])
                .wrapping_add_signed(T2[idx])
                .wrapping_add_signed(T3[idx]);
            let v = if (v as i16) < 0 {
                0
            } else {
                v.min(T4[idx] as u16)
            };
            ram.write_u16(0xe6, v.wrapping_add_signed(-T3[idx]));
        } else {
            ram.write_u16(
                0xe6,
                ram.read_u16(0xe8)
                    .wrapping_sub(ram.read_u16(0x20).wrapping_sub(0x72))
                    .wrapping_add(T1[idx]),
            );
            let v = ram
                .read_u16(0xe2)
                .wrapping_sub(ram.read_u16(0x22).wrapping_sub(0x58))
                .wrapping_add(T0[idx])
                .wrapping_add_signed(T2[idx])
                .wrapping_add_signed(T3[idx]);
            let v = if (v as i16) < 0 {
                0
            } else {
                v.min(T4[idx] as u16)
            };
            ram.write_u16(0xe0, v.wrapping_add_signed(-T3[idx]));
        }
    }
    fn link_handle_moving_animation_full_long_entry(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_handle_moving_animation_full_long_entry();
    }
    fn mirror_bg1_bg2_offsets(&mut self, ram: &mut Ram<'_>) {
        crate::dungeon::Dungeon::new(ram.bytes_mut(), self).mirror_bg1_bg2_offs();
    }
    fn conditional_mosaic_control(&mut self, ram: &mut Ram<'_>) {
        overworld::conditional_mosaic_control(ram);
    }
    fn dungeon_custom_tile_attributes(&self, theme: usize) -> &[u8] {
        &self.assets.bytes(52)[usize::from(self.assets.words(51)[theme])..]
    }
    fn update_pegs_step1(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.update_pegs_step_1());
    }
    fn update_pegs_step2(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.update_pegs_step_2());
    }
    fn recover_peg_gfx_from_mapping(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.recover_peg_gfx_from_mapping());
    }
    fn dungeon_reset_sprites(&mut self, ram: &mut Ram<'_>) {
        crate::sprite::Sprite::new(ram.bytes_mut(), self).dungeon_reset_sprites();
    }
    fn sprite_reset_all(&mut self, ram: &mut Ram<'_>) {
        crate::sprite::Sprite::new(ram.bytes_mut(), self).sprite_reset_all();
    }
    fn load_new_sprite_gfx_set(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.load_new_sprite_gfx_set());
    }
    fn palette_assert_translucency_swap(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.palette_assert_translucency_swap());
    }
    fn room_draw_adjust_torch_lighting_change(
        &mut self,
        ram: &mut Ram<'_>,
        x: u16,
        source: u16,
        dma: u16,
    ) {
        crate::misc::room_draw_adjust_torch_lighting_change(ram, self, x, source, dma);
    }
    fn ancilla_add_blast_wall(&mut self, ram: &mut Ram<'_>) {
        crate::ancilla::Ancilla { ram, runtime: self }.add_blast_wall();
    }
    fn ancilla_spawn_falling_prize(&mut self, ram: &mut Ram<'_>, item: u8) -> i32 {
        crate::ancilla::Ancilla { ram, runtime: self }
            .add_falling_prize(0x29, item, 4)
            .map_or(-1, |k| k as i32)
    }
    fn spotlight_open(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.spotlight_open());
    }
    fn iris_spotlight_configure_table(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.iris_spotlight_configure_table());
    }
    fn overworld_reset_mosaic_always_increase(&mut self, ram: &mut Ram<'_>) {
        overworld::reset_mosaic_always_increase(ram);
    }
    fn palette_revert_translucency_swap(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.palette_revert_translucency_swap());
    }
    fn recover_position_after_drowning(&mut self, ram: &mut Ram<'_>) {
        let services = std::cell::RefCell::new(self);
        overworld::recover_position_after_drowning(
            ram,
            |ram| {
                crate::dungeon::Dungeon::new(ram.bytes_mut(), &mut **services.borrow_mut())
                    .dungeon_play_blip_and_cache_quadrant_visits();
            },
            |ram| {
                crate::player::Player {
                    ram,
                    runtime: &mut **services.borrow_mut(),
                    applied_movement_to_camera: false,
                }
                .link_reset_state_after_damaging_pit();
            },
            |ram| crate::tagalong::initialize(ram, &**services.borrow_mut()),
        );
    }
    fn zelda_is_playing_music_track(&self, track: u8) -> bool {
        crate::audio::is_playing_music_track(
            self.music_playback_state.0,
            self.music_playback_state.1,
            self.music_playback_state.2,
            self.music_unk1,
            track,
            self.music_area,
            self.music_entrance,
        )
    }
    fn adjust_water_hdma_window(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.adjust_water_hdma_window());
    }
    fn adjust_water_hdma_window_x(&mut self, ram: &mut Ram<'_>, upper: u16) {
        self.with_load_gfx(ram, |g| g.adjust_water_hdma_window_x(upper));
    }
    fn save_dungeon_keys(&mut self, ram: &mut Ram<'_>) {
        crate::dungeon::Dungeon::new(ram.bytes_mut(), self).save_dungeon_keys();
    }
    fn graphics_load_chr_half_slot(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.graphics_load_chr_half_slot());
    }
    fn dungeon_room_overlay(&self, index: usize) -> &[u8] {
        &self.assets.bytes(48)[usize::from(self.assets.words(49)[index])..]
    }
    fn sprite_load_graphics_properties_light_world_only(&mut self, ram: &mut Ram<'_>) {
        overworld::sprite_load_graphics_properties_light_world_only(ram, self.assets);
    }
    fn oam_allocate_from_region_b(&mut self, ram: &mut Ram<'_>, bytes: u8) -> (u16, u16) {
        crate::sprite::Sprite::new(ram.bytes_mut(), self).oam_allocate_from_region_b(bytes);
        (ram.read_u16(0x90), ram.read_u16(0x92))
    }
    fn handle_indoor_camera_and_doors(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .handle_indoor_camera_and_doors();
    }
    fn ancilla_sfx2_near(&mut self, ram: &mut Ram<'_>, sound: u8) {
        crate::misc::ancilla_sfx2_near(ram, sound);
    }
    fn link_move_position(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_move_position();
    }
    fn link_handle_moving_animation_start_with_dash(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_handle_moving_animation_start_with_dash();
    }
    fn follower_initialize(&mut self, ram: &mut Ram<'_>) {
        crate::tagalong::initialize(ram, self);
    }
    fn flood_dam_prep_flood_hdma(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.flood_dam_prep_flood_hdma());
    }
    fn graphics_incremental_vram_upload(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.graphics_incremental_vram_upload());
    }
    fn misc_bug_fixes_enabled(&self) -> bool {
        self.enhanced_features0 & 0x1000 != 0
    }
    fn palette_filter_crystal(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.palette_filter_crystal());
    }
    fn crystal_cutscene_initialize_polyhedral(&mut self, ram: &mut Ram<'_>) {
        Ending { ram, runtime: self }.crystal_cutscene_initialize_polyhedral();
    }
    fn sprite_spawn_dynamically(&mut self, ram: &mut Ram<'_>, parent: u8, kind: u8) -> i32 {
        let mut info = crate::sprite::SpriteSpawnInfo::default();
        crate::sprite::Sprite::new(ram.bytes_mut(), self).sprite_spawn_dynamically(
            usize::from(parent),
            kind,
            &mut info,
        )
    }
    fn load_follower_graphics(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.load_follower_graphics());
    }
    fn palette_filter_restore_bg_subtractive_strict(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |g| g.palette_filter_restore_bg_subtractive_strict());
    }
    fn polyhedral_initialize_thread(&mut self, ram: &mut Ram<'_>) {
        Ending { ram, runtime: self }.polyhedral_initialize_thread();
    }
    fn main_show_text_message(&mut self, ram: &mut Ram<'_>) {
        crate::misc::main_show_text_message(ram);
    }
    fn get_random_number(&mut self, ram: &mut Ram<'_>) -> u8 {
        crate::misc::get_random_number(ram)
    }
    fn break_pots_with_sword_enabled(&self) -> bool {
        self.enhanced_features0 & 32 != 0
    }
    fn sprite_spawn_immediately_smashed_terrain(
        &mut self,
        ram: &mut Ram<'_>,
        kind: u8,
        x: u16,
        y: u16,
    ) {
        crate::sprite::Sprite::new(ram.bytes_mut(), self)
            .sprite_spawn_immediately_smashed_terrain(kind, x, y);
    }
    fn ancilla_add_bush_poof(&mut self, ram: &mut Ram<'_>, x: u16, y: u16) {
        crate::ancilla::Ancilla { ram, runtime: self }.add_bush_poof(x, y);
    }
    fn link_handle_velocity(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_handle_velocity();
    }
    fn apply_links_movement_to_camera(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .apply_links_movement_to_camera();
    }
    fn link_check_for_edge_screen_transition(&mut self, ram: &mut Ram<'_>) -> bool {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_check_for_edge_screen_transition()
    }
    fn handle_dungeon_landing_from_pit(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .handle_dungeon_landing_from_pit();
    }

    fn sprite_main(&mut self, ram: &mut Ram<'_>) {
        UnifiedGameplayServices::run_sprite_main(self, ram);
    }

    fn link_oam_main(&mut self, ram: &mut Ram<'_>) {
        UnifiedGameplayServices::run_link_oam_main(self, ram);
    }

    fn did_press_button_for_map(&self, ram: &Ram<'_>) -> bool {
        if ram.read_u8(0x656) != 0 {
            ram.read_u8(0xf4) & 0x20 != 0
        } else {
            ram.read_u8(0xf6) & 0x40 != 0
        }
    }

    fn display_select_menu(&mut self, ram: &mut Ram<'_>) {
        crate::messaging::display_select_menu_core(ram);
    }

    fn hud_handle_item_switch_inputs(&mut self, ram: &mut Ram<'_>) {
        Hud { ram, runtime: self }.handle_item_switch_inputs();
    }

    fn link_main(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_main();
    }
}

impl PlayerOamHooks for NativeGameServices<'_> {
    fn enhanced_features0(&self) -> u32 {
        self.enhanced_features0
    }

    fn swimming_animation(&self, index: usize) -> u8 {
        // kSwimmingTab1 from player_oam.c. Keep the production service
        // explicit so it cannot silently inherit a placeholder hook.
        [2, 0, 1, 0][index]
    }
}

impl FileSelectRuntime for NativeGameServices<'_> {
    fn force_blank(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .enable_force_blank();
    }
    fn erase_triforce_tilemaps(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .erase_tile_maps_triforce();
    }
    fn load_file_select_palettes(&mut self, ram: &mut Ram<'_>, sram: &[u8]) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut FileSelectGfxHooks { sram },
        }
        .palette_load_for_file_select();
    }
    fn load_file_select_graphics(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .load_file_select_graphics();
        crate::overworld::decompress_enemy_damage_subclasses(ram, self.assets);
    }
    fn copy_selected_save_to_wram(&mut self, ram: &mut Ram<'_>, sram: &[u8], slot: usize) {
        // CopySaveToWRAM clears the bird-travel scratch slot before loading a
        // selected file. These values can survive a save/quit into file select.
        for address in [0x1abf, 0x1acf, 0x1adf, 0x1aef, 0x1aff] {
            ram.write_u8(address, 0);
        }
        ram.slice_mut(0xf000, 0x500)
            .copy_from_slice(&sram[slot * 0x500..(slot + 1) * 0x500]);
        ram.write_u16(0xc00d, 7);
        ram.write_u16(0xc013, 7);
        ram.write_u16(0xc00f, 0);
        ram.write_u16(0xc015, 0);
        ram.write_u16(0x219, 0x6040);
        ram.write_u16(0x21d, 0x4841);
        ram.write_u16(0x21f, 0x7f);
        ram.write_u16(0x221, 0xffff);
        if self.enhanced_features0 & 0x1000 != 0 {
            ram.write_u8(0xc011, 0);
        }
        ram.write_u8(0x204, 0x80);
        ram.write_u8(0x10, 5);
        ram.write_u8(0x11, 0);
        ram.write_u8(0x10e, 0);
        ram.write_u8(0x710, 0);
        ram.write_u8(0x010a, 0);
        ram.write_u8(0x15, 0);
        ram.write_u8(0xab2, 0);
    }
    fn persist_sram(&mut self, sram: &[u8]) {
        if let Ok(sram) = <&[u8; 0x2000]>::try_from(sram) {
            let _ = crate::host::write_sram(std::path::Path::new("saves/sram.dat"), sram);
        }
    }
    fn initialize_file_select_graphics(&mut self, ram: &mut Ram<'_>) {
        let mut gfx = LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        };
        gfx.palette_load_dungeon_set();
        gfx.palette_load_owbg3();
        gfx.palette_load_hud();
        gfx.load_default_graphics();
        gfx.initialize_tilesets();
    }
}

impl MiscHooks for NativeGameServices<'_> {
    fn call(&mut self, ram: &mut Ram<'_>, call: MiscCall) {
        match call {
            MiscCall::HudSearchEquipped => Hud { ram, runtime: self }.search_for_equipped_item(),
            MiscCall::HudRebuild => Hud { ram, runtime: self }.rebuild(),
            MiscCall::HudUpdateEquipped => Hud { ram, runtime: self }.update_equipped_item(),
            MiscCall::ModulePreDungeon => {
                crate::dungeon::Dungeon::new(ram.bytes_mut(), self).module_pre_dungeon();
            }
            MiscCall::LoadOwMusicIfNeeded => {
                crate::dungeon::Dungeon::new(ram.bytes_mut(), self).load_ow_music_if_needed();
            }
            MiscCall::InitializeMirrorHdma => {
                let mut setup = None;
                overworld::initialize_mirror_hdma(ram, |value| setup = Some(value));
                self.pending_mirror_hdma = setup;
            }
            MiscCall::PaletteWhiteFilter => {
                self.with_load_gfx(ram, |gfx| gfx.palette_filter_initialize_white_filter());
            }
            MiscCall::OverworldLoadGfx => overworld::load_gfx_and_screen_size(ram, self.assets),
            MiscCall::LinkResetProperties => {
                crate::player::Player {
                    ram,
                    runtime: self,
                    applied_movement_to_camera: false,
                }
                .link_reset_properties_a();
            }
            MiscCall::MirrorBuildWaving => {
                let services = std::cell::RefCell::new(self);
                overworld::mirror_warp_build_waving_hdma_table(ram, |ram| {
                    services.borrow_mut().with_load_gfx(ram, |gfx| {
                        gfx.mirror_warp_run_animation_submodules();
                    });
                });
            }
            MiscCall::MirrorBuildDewaving => {
                let services = std::cell::RefCell::new(self);
                overworld::mirror_warp_build_dewaving_hdma_table(
                    ram,
                    |ram| {
                        services.borrow_mut().with_load_gfx(ram, |gfx| {
                            gfx.mirror_warp_run_animation_submodules();
                        });
                    },
                    overworld::set_fixed_color_and_scroll,
                );
            }
            MiscCall::ReloadSheets => {
                self.with_load_gfx(ram, |gfx| gfx.reload_previously_loaded_sheets());
            }
            MiscCall::HudRebuildIndoor => Hud { ram, runtime: self }.rebuild_indoor(),
            MiscCall::ResetAncillaCutscene => {
                let services = std::cell::RefCell::new(self);
                overworld::reset_ancilla_and_cutscene(ram, |ram, selected| {
                    crate::ancilla::Ancilla {
                        ram,
                        runtime: &mut **services.borrow_mut(),
                    }
                    .terminate_select_interactives(selected);
                });
            }
            MiscCall::OverworldSetSongList => overworld::set_song_list(ram, self.assets),
            MiscCall::SpriteMain => UnifiedGameplayServices::run_sprite_main(self, ram),
            MiscCall::LinkOamMain => UnifiedGameplayServices::run_link_oam_main(self, ram),
            MiscCall::DungeonResetTorchAndPlayer => {
                crate::dungeon::Dungeon::new(ram.bytes_mut(), self)
                    .dungeon_reset_torch_background_and_player_inner();
            }
            MiscCall::HudRefillLogic => Hud { ram, runtime: self }.refill_logic(),
            MiscCall::LinkAnimateVictorySpin => {
                crate::player::Player {
                    ram,
                    runtime: self,
                    applied_movement_to_camera: false,
                }
                .link_animate_victory_spin();
            }
            MiscCall::LinkMain => DungeonHooks::link_main(self, ram),
            MiscCall::DungeonPrepExitSpotlight => self.prep_exit_with_spotlight(ram),
            MiscCall::SpotlightConfigure => {
                let services = std::cell::RefCell::new(self);
                overworld::spotlight_configure_table_and_control(
                    ram,
                    |ram| services.borrow_mut().configure_spotlight_table(ram),
                    |ram| services.borrow_mut().open_spotlight_next(ram),
                );
            }
            MiscCall::PaletteRevertSwap => {
                self.with_load_gfx(ram, |gfx| gfx.palette_revert_translucency_swap());
            }
            MiscCall::OpenSpotlightNext2 => {
                let services = std::cell::RefCell::new(self);
                let assets = services.borrow().assets;
                overworld::open_spotlight_next2(
                    ram,
                    |ram| {
                        let mut services = services.borrow_mut();
                        LoadGfx {
                            wram: ram.bytes_mut(),
                            vram: services.scratch_vram.as_mut(),
                            assets,
                            hooks: &mut NoGfxHooks,
                        }
                        .enable_force_blank();
                    },
                    |ram| {
                        crate::dungeon::Dungeon::new(ram.bytes_mut(), &mut **services.borrow_mut())
                            .dungeon_reset_torch_background_and_player_inner();
                    },
                );
            }
            MiscCall::EnableForceBlank => self.with_load_gfx(ram, |gfx| gfx.enable_force_blank()),
            MiscCall::EraseTileMaps => {
                self.with_load_gfx(ram, |gfx| gfx.erase_tile_maps_normal());
            }
            MiscCall::LoadDefaultGraphics => {
                self.with_load_gfx(ram, |gfx| gfx.load_default_graphics());
            }
            MiscCall::SpriteLoadGraphicsProperties => {
                overworld::sprite_load_graphics_properties(ram, self.assets);
            }
            MiscCall::DecompressSword => {
                self.with_load_gfx(ram, |gfx| gfx.decompress_sword_graphics());
            }
            MiscCall::DecompressShield => {
                self.with_load_gfx(ram, |gfx| gfx.decompress_shield_graphics());
            }
            MiscCall::LinkInitialize => {
                crate::player::Player {
                    ram,
                    runtime: self,
                    applied_movement_to_camera: false,
                }
                .link_initialize();
            }
            MiscCall::LoadFollowerGraphics => {
                self.with_load_gfx(ram, |gfx| gfx.load_follower_graphics());
            }
            MiscCall::DungeonLoadPalettes => {
                self.with_load_gfx(ram, |gfx| gfx.dungeon_load_palettes());
            }
            MiscCall::PaletteUpdateGloves => {
                self.with_load_gfx(ram, |gfx| gfx.palette_update_gloves_color());
            }
            MiscCall::RefreshIcon => Hud { ram, runtime: self }.refresh_icon(),
            MiscCall::PaletteLoadShield => {
                self.with_load_gfx(ram, |gfx| gfx.palette_load_shield());
            }
            MiscCall::PaletteLoadSword => {
                self.with_load_gfx(ram, |gfx| gfx.palette_load_sword());
            }
            MiscCall::DecodeAnimatedTile(tile) => {
                self.with_load_gfx(ram, |gfx| gfx.decode_animated_sprite_tile_variable(tile));
            }
            MiscCall::SaveDungeonKeys => {
                crate::dungeon::Dungeon::new(ram.bytes_mut(), self).save_dungeon_keys();
            }
            MiscCall::FlagRoomQuadrants => {
                crate::dungeon::Dungeon::new(ram.bytes_mut(), self)
                    .dungeon_flag_room_data_quadrants();
            }
            MiscCall::SpriteResetAll => {
                crate::sprite::Sprite::new(ram.bytes_mut(), self).sprite_reset_all();
            }
            MiscCall::ResetAfterDeath(speed) => {
                crate::player::Player {
                    ram,
                    runtime: self,
                    applied_movement_to_camera: false,
                }
                .reset_some_things_after_death(speed);
            }
            MiscCall::TerminateInteractives(selected) => {
                crate::ancilla::Ancilla { ram, runtime: self }
                    .terminate_select_interactives(selected);
            }
            MiscCall::AddVictorySpin => {
                let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_victory_spin();
            }
            MiscCall::CapePoof(kind, limit) => {
                let _ = crate::ancilla::Ancilla { ram, runtime: self }.add_cape_poof(kind, limit);
            }
            MiscCall::MsCutscene(kind, limit) => {
                let _ = crate::ancilla::Ancilla { ram, runtime: self }
                    .ancilla_add_ms_cutscene(kind, limit);
            }
            MiscCall::HdmaSetup {
                addr6,
                addr7,
                transfer_unit,
                reg6,
                reg7,
                indirect_bank,
            } => self.pending_host_actions.push(NativeHostAction::HdmaSetup {
                addr6,
                addr7,
                transfer_unit,
                reg6,
                reg7,
                indirect_bank,
            }),
            MiscCall::RenderText => self.pending_host_actions.push(NativeHostAction::RenderText),
            MiscCall::DeathFunc15(count_as_death) => self
                .pending_host_actions
                .push(NativeHostAction::DeathFunc15(count_as_death)),
        }
    }
    fn route_main_module(&mut self, _ram: &mut Ram<'_>, _module: u8) {
        self.pending_host_actions
            .push(NativeHostAction::RouteMainModule);
    }
    fn load_song_bank(&mut self, _ram: &mut Ram<'_>, bank: u8) {
        self.pending_host_actions
            .push(NativeHostAction::LoadSongBank(bank));
    }
    fn predefined_tile_word(&self, word_index: usize) -> u16 {
        self.assets.words(69)[word_index]
    }
    fn refill_magic(&mut self, ram: &mut Ram<'_>) -> bool {
        Hud { ram, runtime: self }.refill_magic_power()
    }
    fn refill_health(&mut self, ram: &mut Ram<'_>) -> bool {
        Hud { ram, runtime: self }.refill_health()
    }
    fn add_ancilla(&mut self, ram: &mut Ram<'_>, kind: u8, limit: u8) -> i32 {
        (crate::ancilla::Ancilla { ram, runtime: self })
            .add_ancilla(kind, limit)
            .map_or(-1, |slot| slot as i32)
    }
    fn set_ancilla_xy(&mut self, ram: &mut Ram<'_>, ancilla: usize, x: u16, y: u16) {
        let mut context = crate::ancilla::Ancilla { ram, runtime: self };
        context.set_x(ancilla, x);
        context.set_y(ancilla, y);
    }
    fn sprite_x(&self, ram: &Ram<'_>, sprite: usize) -> u16 {
        u16::from(ram.read_u8(0xd10 + sprite)) | u16::from(ram.read_u8(0xd30 + sprite)) << 8
    }
    fn dungeon_item_tile_action(&mut self, ram: &mut Ram<'_>, x: u16, y: u16) -> u8 {
        crate::dungeon::Dungeon::new(ram.bytes_mut(), self).handle_item_tile_action_dungeon(x, y)
    }
    fn overworld_item_tile_action(&mut self, ram: &mut Ram<'_>, x: u16, y: u16) -> u8 {
        let assets = self.assets;
        let services = std::cell::RefCell::new(self);
        overworld::tool_and_tile_interaction(
            ram,
            assets,
            x,
            y,
            |ram, kind, sx, sy| {
                crate::sprite::Sprite::new(ram.bytes_mut(), &mut **services.borrow_mut())
                    .sprite_spawn_immediately_smashed_terrain(kind as u8, sx, sy);
            },
            |ram, sx, sy| {
                let _ = crate::ancilla::Ancilla {
                    ram,
                    runtime: &mut **services.borrow_mut(),
                }
                .add_bush_poof(sx, sy);
            },
        ) as u8
    }
}

impl HudRuntime for NativeGameServices<'_> {
    fn render(&mut self, operation: HudRenderOp, ram: &mut Ram<'_>) {
        match operation {
            HudRenderOp::SwitchLetterTiles(button) => {
                ram.write_u8(0x100, button);
            }
        }
    }

    fn display_select_menu(&mut self, ram: &mut Ram<'_>) {
        Hud { ram, runtime: self }.run_module();
    }
}

impl SpotlightServices for NativeGameServices<'_> {
    fn prep_exit_with_spotlight(&mut self, ram: &mut Ram<'_>) {
        let services = std::cell::RefCell::new(self);
        let assets = services.borrow().assets;
        overworld::dungeon_prep_exit_with_spotlight(
            ram,
            |entrance| assets.dungeon_entrance(false, usize::from(entrance)).music,
            |ram| {
                Hud {
                    ram,
                    runtime: &mut **services.borrow_mut(),
                }
                .floor_indicator()
            },
            |ram| {
                services
                    .borrow_mut()
                    .with_load_gfx(ram, |gfx| gfx.iris_spotlight_close())
            },
        );
    }
    fn configure_spotlight_table(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.iris_spotlight_configure_table());
    }
    fn open_spotlight_next(&mut self, ram: &mut Ram<'_>) {
        let services = std::cell::RefCell::new(self);
        let assets = services.borrow().assets;
        overworld::open_spotlight_next(
            ram,
            |ram| {
                let mut services = services.borrow_mut();
                LoadGfx {
                    wram: ram.bytes_mut(),
                    vram: services.scratch_vram.as_mut(),
                    assets,
                    hooks: &mut NoGfxHooks,
                }
                .enable_force_blank();
            },
            |ram| {
                let mut services = services.borrow_mut();
                crate::player::Player {
                    ram,
                    runtime: &mut **services,
                    applied_movement_to_camera: false,
                }
                .link_item_reset_from_overworld_things();
            },
        );
    }
    fn spotlight_open(&mut self, ram: &mut Ram<'_>) {
        self.with_load_gfx(ram, |gfx| gfx.spotlight_open());
    }
    fn link_handle_velocity(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_handle_velocity();
    }
    fn link_handle_moving_animation_full_long_entry(&mut self, ram: &mut Ram<'_>) {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_handle_moving_animation_full_long_entry();
    }
}

impl OverworldServices for NativeGameServices<'_> {
    fn overworld_call(&mut self, ram: &mut Ram<'_>, call: OverworldCall) {
        match call {
            OverworldCall::LoadTransitionAuxGfx => LoadGfx {
                wram: ram.bytes_mut(),
                vram: self.scratch_vram.as_mut(),
                assets: self.assets,
                hooks: &mut NoGfxHooks,
            }
            .load_trans_aux_gfx(),
            OverworldCall::PrepareTransitionAuxGfx => LoadGfx {
                wram: ram.bytes_mut(),
                vram: self.scratch_vram.as_mut(),
                assets: self.assets,
                hooks: &mut NoGfxHooks,
            }
            .prep_trans_aux_gfx(),
            OverworldCall::LoadEventOverlay => overworld::overworld_load_event_overlay(ram),
            OverworldCall::TriggerMapLoadStripe(axis, count) => {
                overworld::trigger_and_finish_map_load_stripe(ram, self.assets, axis, count);
            }
            OverworldCall::LoadNewSpriteGfxSet => LoadGfx {
                wram: ram.bytes_mut(),
                vram: self.scratch_vram.as_mut(),
                assets: self.assets,
                hooks: &mut NoGfxHooks,
            }
            .load_new_sprite_gfx_set(),
            OverworldCall::ReloadOverworldSprites => {
                crate::sprite::Sprite::new(ram.bytes_mut(), self).sprite_reload_all_overworld();
            }
            OverworldCall::ReloadOverworldSpritesJustLoad => {
                crate::sprite::Sprite::new(ram.bytes_mut(), self)
                    .sprite_overworld_reload_all_just_load();
            }
            OverworldCall::HandleMovingAnimation => {
                crate::player::Player {
                    ram,
                    runtime: self,
                    applied_movement_to_camera: false,
                }
                .link_handle_moving_animation_full_long_entry();
            }
            OverworldCall::IncrementalVramUpload => LoadGfx {
                wram: ram.bytes_mut(),
                vram: self.scratch_vram.as_mut(),
                assets: self.assets,
                hooks: &mut NoGfxHooks,
            }
            .graphics_incremental_vram_upload(),
            OverworldCall::InitializeSpriteSlots => {
                crate::sprite::Sprite::new(ram.bytes_mut(), self).sprite_initialize_slots();
            }
            OverworldCall::DisableFollower => crate::tagalong::disable(ram),
            OverworldCall::ResetTransitionPropsAndInterface => {
                crate::dungeon::Dungeon::new(ram.bytes_mut(), self)
                    .reset_transition_props_and_advance_reset_interface();
            }
            OverworldCall::ApplyPaletteFilterBounce => LoadGfx {
                wram: ram.bytes_mut(),
                vram: self.scratch_vram.as_mut(),
                assets: self.assets,
                hooks: &mut NoGfxHooks,
            }
            .apply_palette_filter_bounce(),
            OverworldCall::DecodeAnimatedSpriteTile(tile) => LoadGfx {
                wram: ram.bytes_mut(),
                vram: self.scratch_vram.as_mut(),
                assets: self.assets,
                hooks: &mut NoGfxHooks,
            }
            .decode_animated_sprite_tile_variable(tile),
            OverworldCall::LoadPalettes(background, sprites) => LoadGfx {
                wram: ram.bytes_mut(),
                vram: self.scratch_vram.as_mut(),
                assets: self.assets,
                hooks: &mut NoGfxHooks,
            }
            .overworld_load_palettes(background, sprites),
            OverworldCall::ApplySpecialOverworldPalette => LoadGfx {
                wram: ram.bytes_mut(),
                vram: self.scratch_vram.as_mut(),
                assets: self.assets,
                hooks: &mut NoGfxHooks,
            }
            .palette_special_ow(),
            OverworldCall::ResetSwimmingState => {
                crate::player::Player {
                    ram,
                    runtime: self,
                    applied_movement_to_camera: false,
                }
                .link_reset_swimming_state();
            }
            OverworldCall::DrawQuadrantsAndOverlays => {
                overworld::draw_quadrants_and_overlays(
                    ram,
                    self.assets,
                    overworld::overworld_load_event_overlay,
                );
            }
            OverworldCall::InitializeWhiteFilter => LoadGfx {
                wram: ram.bytes_mut(),
                vram: self.scratch_vram.as_mut(),
                assets: self.assets,
                hooks: &mut NoGfxHooks,
            }
            .palette_filter_initialize_white_filter(),
            OverworldCall::RunAnimationSubmodules => self.with_load_gfx(ram, |gfx| {
                gfx.mirror_warp_run_animation_submodules();
            }),
            OverworldCall::ResetHdmaAndIris => LoadGfx {
                wram: ram.bytes_mut(),
                vram: self.scratch_vram.as_mut(),
                assets: self.assets,
                hooks: &mut NoGfxHooks,
            }
            .iris_spotlight_reset_table(),
            OverworldCall::ReloadPreviouslyLoadedSheets => LoadGfx {
                wram: ram.bytes_mut(),
                vram: self.scratch_vram.as_mut(),
                assets: self.assets,
                hooks: &mut NoGfxHooks,
            }
            .reload_previously_loaded_sheets(),
            OverworldCall::CacheQuadrantVisits => {
                crate::dungeon::Dungeon::new(ram.bytes_mut(), self)
                    .dungeon_play_blip_and_cache_quadrant_visits();
            }
            OverworldCall::ResetLinkState => {
                crate::player::Player {
                    ram,
                    runtime: self,
                    applied_movement_to_camera: false,
                }
                .link_reset_state_after_damaging_pit();
            }
            OverworldCall::InitializeFollower => crate::tagalong::initialize(ram, self),
            OverworldCall::PaletteFilterBlindingWhite => {
                self.with_load_gfx(ram, |gfx| gfx.palette_filter_blinding_white());
            }
            OverworldCall::PaletteFilterWhirlpoolBlue => LoadGfx {
                wram: ram.bytes_mut(),
                vram: self.scratch_vram.as_mut(),
                assets: self.assets,
                hooks: &mut NoGfxHooks,
            }
            .palette_filter_whirlpool_blue(),
            OverworldCall::PaletteFilterIsolateWhirlpoolBlue => LoadGfx {
                wram: ram.bytes_mut(),
                vram: self.scratch_vram.as_mut(),
                assets: self.assets,
                hooks: &mut NoGfxHooks,
            }
            .palette_filter_isolate_whirlpool_blue(),
            OverworldCall::FindPartnerWhirlpoolExit => {
                let assets = self.assets;
                let services = std::cell::RefCell::new(self);
                overworld::find_partner_whirlpool_exit(ram, assets, |ram, index| {
                    overworld::load_bird_travel_position(
                        ram,
                        assets,
                        index,
                        |ram| overworld::load_gfx_and_screen_size(ram, assets),
                        |ram| {
                            crate::sprite::Sprite::new(
                                ram.bytes_mut(),
                                &mut **services.borrow_mut(),
                            )
                            .sprite_reset_all()
                        },
                        |ram| {
                            PreOverworldHooks::reload_overworld_sprites(
                                &mut **services.borrow_mut(),
                                ram,
                            )
                        },
                        |ram| {
                            TransitionHooks::reset_torch_background_and_player(
                                &mut **services.borrow_mut(),
                                ram,
                            )
                        },
                    );
                });
            }
            OverworldCall::LoadOverlayAndMap => {
                let module = ram.read_u16(0x10);
                let map_state = ram.read_u16(0x200);
                overworld::load_and_build_screen(ram, self.assets, |ram| {
                    overworld::draw_quadrants_and_overlays(
                        ram,
                        self.assets,
                        overworld::overworld_load_event_overlay,
                    );
                });
                ram.write_u16(0x200, map_state.wrapping_add(1));
                ram.write_u16(0x10, module);
            }
            OverworldCall::WhirlpoolReloadPalettes => {
                let screen = usize::from(ram.read_u8(0x8a));
                let background =
                    overworld::OverworldAssets::background_palettes(self.assets)[screen];
                let sprites = ram.read_u8(0xfd40 + screen);
                self.with_load_gfx(ram, |gfx| {
                    gfx.palette_load_sprite_main();
                    gfx.palette_load_sprite_environment();
                    gfx.palette_load_sp0l();
                    gfx.palette_load_hud();
                    gfx.palette_load_owbg_main();
                    gfx.overworld_load_palettes(background, sprites);
                    gfx.palette_set_ow_bg_color();
                });
                overworld::overworld_set_fixed_col_and_scroll(ram);
            }
            OverworldCall::WhirlpoolRestoreRedGreen => {
                self.with_load_gfx(ram, |gfx| gfx.palette_filter_whirlpool_restore_red_green());
            }
            OverworldCall::WhirlpoolRestoreBlue => {
                self.with_load_gfx(ram, |gfx| gfx.palette_filter_whirlpool_restore_blue());
            }
        }
    }
    fn is_playing_music_track(&mut self, track: u8) -> bool {
        let (state, enabled, actual_track) = self.music_playback_state;
        crate::audio::is_playing_music_track(
            state,
            enabled,
            actual_track,
            self.music_unk1,
            track,
            self.music_area,
            self.music_entrance,
        )
    }
    fn setup_mirror_hdma(&mut self, setup: overworld::MirrorHdmaSetup) {
        self.pending_mirror_hdma = Some(setup);
    }
}

impl PreOverworldHooks for NativeGameServices<'_> {
    fn load_from_dungeon(&mut self, ram: &mut Ram<'_>) {
        overworld::load_overworld_from_dungeon(ram, self.assets, |ram| {
            overworld::load_gfx_and_screen_size(ram, self.assets);
        });
    }
    fn load_from_special_overworld(&mut self, ram: &mut Ram<'_>) {
        let services = std::cell::RefCell::new(self);
        let assets = services.borrow().assets;
        overworld::load_overworld_from_special_overworld(
            ram,
            assets,
            |ram, background, sprites| {
                PreOverworldHooks::load_palettes(
                    &mut **services.borrow_mut(),
                    ram,
                    background,
                    sprites,
                );
            },
            |ram| {
                let mut services = services.borrow_mut();
                LoadGfx {
                    wram: ram.bytes_mut(),
                    vram: services.scratch_vram.as_mut(),
                    assets,
                    hooks: &mut NoGfxHooks,
                }
                .palette_special_ow();
            },
            |ram| {
                crate::player::Player {
                    ram,
                    runtime: &mut **services.borrow_mut(),
                    applied_movement_to_camera: false,
                }
                .link_reset_swimming_state();
            },
            |ram| overworld::load_gfx_and_screen_size(ram, assets),
        );
    }
    fn hud_refill_logic(&mut self, ram: &mut Ram<'_>) {
        Hud { ram, runtime: self }.refill_logic();
    }
    fn decompress_animated_tiles(&mut self, ram: &mut Ram<'_>, set: u8) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .decompress_animated_overworld_tiles(set);
    }
    fn initialize_tilesets(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .initialize_tilesets();
    }
    fn load_screen_palette_set(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .overworld_load_screens_palette_set();
    }
    fn load_palettes(&mut self, ram: &mut Ram<'_>, background: u8, sprites: u8) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .overworld_load_palettes(background, sprites);
    }
    fn set_overworld_background_color(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .palette_set_ow_bg_color();
    }
    fn load_palettes_inner(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .overworld_load_palettes_inner();
    }
    fn copy_special_palettes_to_cache(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .special_overworld_copy_palettes_to_cache();
    }
    fn initialize_follower(&mut self, ram: &mut Ram<'_>) {
        crate::tagalong::initialize(ram, self);
    }
    fn decode_animated_sprite_tile(&mut self, ram: &mut Ram<'_>, tile: u8) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .decode_animated_sprite_tile_variable(tile);
    }
    fn reload_overworld_sprites(&mut self, ram: &mut Ram<'_>) {
        crate::sprite::Sprite::new(ram.bytes_mut(), self).sprite_reload_all_overworld();
    }
    fn initialize_mirror_portal(&mut self, ram: &mut Ram<'_>) {
        crate::sprite::Sprite::new(ram.bytes_mut(), self).sprite_initialize_mirror_portal();
    }
    fn reset_torch_background_and_player(&mut self, ram: &mut Ram<'_>) {
        crate::dungeon::Dungeon::new(ram.bytes_mut(), self)
            .dungeon_reset_torch_background_and_player_inner();
    }
    fn load_bunny_palettes(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .load_gear_palettes_bunny();
    }
    fn load_overworld_music_if_needed(&mut self, ram: &mut Ram<'_>) {
        crate::dungeon::Dungeon::new(ram.bytes_mut(), self).load_ow_music_if_needed();
    }
}

impl PlayerControlHooks for NativeGameServices<'_> {
    fn did_press_map(&mut self, ram: &mut Ram<'_>) -> bool {
        DungeonHooks::did_press_button_for_map(self, ram)
    }
    fn display_select_menu(&mut self, ram: &mut Ram<'_>) {
        HudRuntime::display_select_menu(self, ram);
    }
    fn handle_item_switch_inputs(&mut self, ram: &mut Ram<'_>) {
        Hud { ram, runtime: self }.handle_item_switch_inputs();
    }
    fn animate_entrance(&mut self, ram: &mut Ram<'_>) {
        match overworld::animate_entrance(ram) {
            overworld::EntranceAnimation::PalaceOfDarkness => {
                overworld::animate_entrance_palace_of_darkness(ram, self.assets);
            }
            overworld::EntranceAnimation::SkullWoods => {
                overworld::animate_entrance_skull_woods(ram, self.assets);
            }
            overworld::EntranceAnimation::MiseryMire => {
                overworld::animate_entrance_misery_mire(ram, self.assets);
            }
            overworld::EntranceAnimation::TurtleRock => {
                let services = std::cell::RefCell::new(self);
                let assets = services.borrow().assets;
                overworld::animate_entrance_turtle_rock(
                    ram,
                    assets,
                    |ram, value| {
                        crate::dungeon::Dungeon::new(ram.bytes_mut(), &mut **services.borrow_mut())
                            .dungeon_approach_fixed_color_variable(value);
                    },
                    |ram, from, to| {
                        let mut services = services.borrow_mut();
                        LoadGfx {
                            wram: ram.bytes_mut(),
                            vram: services.scratch_vram.as_mut(),
                            assets,
                            hooks: &mut NoGfxHooks,
                        }
                        .palette_filter_restore_additive(usize::from(from), usize::from(to));
                    },
                    |ram, from, to| {
                        let mut services = services.borrow_mut();
                        LoadGfx {
                            wram: ram.bytes_mut(),
                            vram: services.scratch_vram.as_mut(),
                            assets,
                            hooks: &mut NoGfxHooks,
                        }
                        .palette_filter_restore_subtractive(usize::from(from), usize::from(to));
                    },
                );
            }
            overworld::EntranceAnimation::GanonsTower => {
                let services = std::cell::RefCell::new(self);
                let assets = services.borrow().assets;
                overworld::animate_entrance_ganons_tower(ram, assets, |ram| {
                    overworld::ganon_tower_entrance_flash(ram, |ram| {
                        services
                            .borrow_mut()
                            .with_load_gfx(ram, |gfx| gfx.palette_filter_blinding_white());
                    });
                });
            }
        }
    }
    fn link_main(&mut self, ram: &mut Ram<'_>) {
        DungeonHooks::link_main(self, ram);
    }
    fn super_bomb_indicator(&mut self, ram: &mut Ram<'_>) {
        Hud { ram, runtime: self }.super_bomb_indicator();
    }
    fn load_chr_half_slot(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .graphics_load_chr_half_slot();
    }
    fn operate_camera_scroll(&mut self, ram: &mut Ram<'_>) {
        overworld::operate_camera_scroll(ram);
    }
    fn use_entrance(&mut self, ram: &mut Ram<'_>) {
        overworld::use_entrance(ram, self.assets, crate::misc::main_show_text_message);
    }
    fn animate_dark_world_palette(&mut self, ram: &mut Ram<'_>) {
        overworld::dw_death_mountain_palette_animation(ram);
    }
    fn handle_transitions(&mut self, ram: &mut Ram<'_>) {
        overworld::handle_transitions(ram, self.assets, self);
    }
    fn scroll_and_check_special_exit(&mut self, ram: &mut Ram<'_>) {
        overworld::scroll_and_check_for_special_overworld_exit(ram, self.assets);
    }
}

impl TransitionHooks for NativeGameServices<'_> {
    fn check_edge_screen_transition(&mut self, ram: &mut Ram<'_>) -> bool {
        crate::player::Player {
            ram,
            runtime: self,
            applied_movement_to_camera: false,
        }
        .link_check_for_edge_screen_transition()
    }

    fn reset_torch_background_and_player(&mut self, ram: &mut Ram<'_>) {
        crate::dungeon::Dungeon::new(ram.bytes_mut(), self)
            .dungeon_reset_torch_background_and_player_inner();
    }

    fn is_playing_music_track(&mut self, track: u8) -> bool {
        OverworldServices::is_playing_music_track(self, track)
    }

    fn load_palettes(&mut self, ram: &mut Ram<'_>, background: u8, sprites: u8) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .overworld_load_palettes(background, sprites);
    }

    fn copy_palettes_to_cache(&mut self, ram: &mut Ram<'_>) {
        LoadGfx {
            wram: ram.bytes_mut(),
            vram: self.scratch_vram.as_mut(),
            assets: self.assets,
            hooks: &mut NoGfxHooks,
        }
        .overworld_copy_palettes_to_cache();
    }
}

#[cfg(test)]
mod structural_tests {
    use std::collections::BTreeSet;

    fn method_names(source: &str, marker: &str) -> BTreeSet<String> {
        let block = source
            .split_once(marker)
            .unwrap_or_else(|| panic!("missing source marker {marker}"))
            .1;
        let mut depth = 0usize;
        let mut entered = false;
        let mut body = String::new();
        for character in block.chars() {
            if character == '{' {
                depth += 1;
                entered = true;
            }
            if entered {
                body.push(character);
            }
            if character == '}' {
                depth -= 1;
                if entered && depth == 0 {
                    break;
                }
            }
        }
        body.lines()
            .filter_map(|line| {
                line.trim_start()
                    .strip_prefix("fn ")
                    .and_then(|rest| rest.split_once('('))
                    .map(|(name, _)| name.trim().to_owned())
            })
            .collect()
    }

    #[test]
    fn every_native_service_trait_has_an_explicit_production_surface() {
        let production = include_str!("native_services.rs");
        let traits: [(&str, &str, &str, &[&str]); 14] = [
            (
                include_str!("load_gfx.rs"),
                "pub trait LoadGfxHooks",
                "impl LoadGfxHooks for NativeGameServices",
                &["file_select_gear"],
            ),
            (
                include_str!("messaging.rs"),
                "pub trait MessagingRuntime",
                "impl MessagingRuntime for NativeGameServices",
                &[][..],
            ),
            (
                include_str!("attract.rs"),
                "pub trait AttractRuntime",
                "impl AttractRuntime for NativeGameServices",
                &["sprite_set_x", "sprite_set_y", "sprite_get_16_bit_coords"],
            ),
            (
                include_str!("ending.rs"),
                "pub trait EndingRuntime",
                "impl EndingRuntime for NativeGameServices",
                &[
                    // Live resource owners override these in EndingAdapter.
                    "credits_hdma_setup",
                    "ppu_write",
                    "save_game_file",
                    // Complete RAM-only translations supplied by the trait.
                    "sprite_set_x",
                    "sprite_set_y",
                ],
            ),
            (
                include_str!("dungeon.rs"),
                "pub trait DungeonHooks",
                "impl DungeonHooks for NativeGameServices",
                &[][..],
            ),
            (
                include_str!("player_oam.rs"),
                "pub trait PlayerOamHooks",
                "impl PlayerOamHooks for NativeGameServices",
                &[][..],
            ),
            (
                include_str!("select_file.rs"),
                "pub trait FileSelectRuntime",
                "impl FileSelectRuntime for NativeGameServices",
                &[][..],
            ),
            (
                include_str!("misc.rs"),
                "pub trait MiscHooks",
                "impl MiscHooks for NativeGameServices",
                &[][..],
            ),
            (
                include_str!("hud.rs"),
                "pub trait HudRuntime",
                "impl HudRuntime for NativeGameServices",
                &[][..],
            ),
            (
                include_str!("integration.rs"),
                "pub trait SpotlightServices",
                "impl SpotlightServices for NativeGameServices",
                &[][..],
            ),
            (
                include_str!("integration.rs"),
                "pub trait OverworldServices",
                "impl OverworldServices for NativeGameServices",
                &[][..],
            ),
            (
                include_str!("overworld.rs"),
                "pub trait PreOverworldHooks",
                "impl PreOverworldHooks for NativeGameServices",
                &[][..],
            ),
            (
                include_str!("overworld.rs"),
                "pub trait PlayerControlHooks",
                "impl PlayerControlHooks for NativeGameServices",
                &[][..],
            ),
            (
                include_str!("overworld.rs"),
                "pub trait TransitionHooks",
                "impl TransitionHooks for NativeGameServices",
                &[][..],
            ),
        ];
        for (trait_source, trait_marker, impl_marker, reviewed_defaults) in traits {
            let declared = method_names(trait_source, trait_marker);
            let implemented = method_names(production, impl_marker);
            let missing = declared
                .difference(&implemented)
                .cloned()
                .collect::<BTreeSet<_>>();
            let reviewed = reviewed_defaults
                .iter()
                .map(|name| (*name).to_owned())
                .collect();
            assert_eq!(
                missing, reviewed,
                "unreviewed inherited defaults in {impl_marker}"
            );
            assert!(
                implemented.is_subset(&declared),
                "unknown methods in {impl_marker}"
            );
        }
        for banned in [
            ["impl LoadGfxHooks for NoGfxHooks ", "{}"].concat(),
            ["resource-aware routing ", "required"].concat(),
            ["unrouted ending ", "cross-module call"].concat(),
            [
                "fn route_main_module(&mut self, _ram: &mut Ram<'_>, _module: u8) ",
                "{}",
            ]
            .concat(),
            [
                "fn load_song_bank(&mut self, _ram: &mut Ram<'_>, _bank: u8) ",
                "{}",
            ]
            .concat(),
        ] {
            assert!(
                !production.contains(&banned),
                "forbidden fallback: {banned}"
            );
        }
    }

    #[test]
    fn native_dungeon_hooks_implement_the_complete_trait_surface() {
        let declared = method_names(include_str!("dungeon.rs"), "pub trait DungeonHooks");
        let implemented = method_names(
            include_str!("native_services.rs"),
            "impl DungeonHooks for NativeGameServices",
        );
        assert_eq!(declared.len(), 111, "DungeonHooks inventory changed");
        assert_eq!(
            implemented, declared,
            "NativeGameServices inherited a DungeonHooks default"
        );
    }
}
