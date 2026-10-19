#!/usr/bin/env ruby
# Reports C function definitions which do not have a mechanically matching
# snake_case Rust function. This is intentionally conservative: renamed
# adapters can be allow-listed only after their equivalence is reviewed.

ROOT = File.expand_path("..", __dir__)

PAIRS = {
  "src/ancilla.c" => %w[crates/z3-game/src/ancilla.rs crates/z3-game/src/ancilla_handlers_0.rs],
  "src/attract.c" => %w[crates/z3-game/src/attract.rs],
  "src/audio.c" => %w[crates/z3-game/src/audio.rs],
  "src/config.c" => %w[crates/z3-core/src/config.rs],
  "src/dungeon.c" => %w[crates/z3-game/src/dungeon.rs],
  "src/ending.c" => %w[crates/z3-game/src/ending.rs],
  "src/hud.c" => %w[crates/z3-game/src/hud.rs],
  "src/load_gfx.c" => %w[crates/z3-game/src/load_gfx.rs],
  "src/main.c" => %w[
    crates/z3-game/src/host.rs
    crates/z3-game/src/runtime.rs
    crates/z3-game/src/assets.rs
    crates/z3-game/src/audio.rs
    crates/z3-core/src/config.rs
    crates/z3-snes/src/system.rs
    crates/z3-native/src/lib.rs
    crates/z3-sdl/src/lib.rs
  ],
  "src/messaging.c" => %w[crates/z3-game/src/messaging.rs],
  "src/misc.c" => %w[crates/z3-game/src/misc.rs],
  "src/nmi.c" => %w[crates/z3-game/src/nmi.rs],
  "src/overlord.c" => %w[crates/z3-game/src/overlord.rs],
  "src/overworld.c" => %w[crates/z3-game/src/overworld.rs],
  "src/player.c" => %w[crates/z3-game/src/player.rs],
  "src/player_oam.c" => %w[crates/z3-game/src/player_oam.rs],
  "src/poly.c" => %w[crates/z3-game/src/poly.rs],
  "src/select_file.c" => %w[crates/z3-game/src/select_file.rs],
  "src/spc_player.c" => %w[crates/z3-game/src/spc_player.rs crates/z3-game/src/spc_player/ram_mapping.rs crates/z3-game/src/spc_player/phrase_cursor.rs crates/z3-game/src/spc_player/pattern_cursor.rs],
  "src/sprite.c" => %w[crates/z3-game/src/sprite.rs],
  "src/sprite_main.c" => %w[crates/z3-game/src/sprite_main_bank0.rs crates/z3-game/src/sprite_main_bank1.rs crates/z3-game/src/sprite_main_bank2a.rs crates/z3-game/src/sprite_main_bank2b.rs],
  "src/tagalong.c" => %w[crates/z3-game/src/tagalong.rs],
  "src/tile_detect.c" => %w[crates/z3-game/src/tile_detect.rs],
  "src/util.c" => %w[crates/z3-core/src/util.rs],
  "src/zelda_cpu_infra.c" => %w[
    crates/z3-verify/src/c_oracle.rs
    crates/z3-verify/src/lib.rs
    crates/z3-verify/src/schema.rs
    crates/z3-verify/src/legacy_snapshot.rs
    crates/z3-verify/src/bin/check_cpu.rs
    crates/z3-verify/src/bin/check_integrated_frame.rs
    crates/z3-game/src/runtime.rs
    crates/z3-snes/src/cpu.rs
  ],
  "src/zelda_rtl.c" => %w[crates/z3-game/src/runtime.rs crates/z3-game/src/runtime/recorder.rs crates/z3-game/src/runtime/recorder/recording_file.rs crates/z3-game/src/runtime/recorder/replay_log.rs crates/z3-game/src/host.rs crates/z3-game/src/assets.rs crates/z3-snes/src/ppu.rs crates/z3-verify/src/c_oracle.rs],
  "snes/apu.c" => %w[crates/z3-snes/src/apu.rs],
  "snes/cart.c" => %w[crates/z3-snes/src/cart.rs],
  "snes/cpu.c" => %w[crates/z3-snes/src/cpu.rs],
  "snes/dma.c" => %w[crates/z3-snes/src/dma.rs],
  "snes/dsp.c" => %w[crates/z3-snes/src/dsp.rs],
  "snes/input.c" => %w[crates/z3-snes/src/input.rs],
  "snes/ppu.c" => %w[crates/z3-snes/src/ppu.rs],
  "snes/snes.c" => %w[crates/z3-snes/src/system.rs crates/z3-snes/src/bus.rs],
  "snes/snes_other.c" => %w[crates/z3-snes/src/system.rs crates/z3-snes/src/bus.rs],
  "snes/spc.c" => %w[crates/z3-snes/src/spc.rs],
}.freeze

# Reviewed spelling/abstraction differences where the Rust port already has the
# same implementation. Keep this list explicit: the audit must not silently
# accept arbitrary fuzzy renames.
EQUIVALENT_NAMES = {
  # audio.c: reviewed type/method translations. The Rust runtime splits the
  # C tagged MsuPlayer into PCM and OPUZ playback objects; both use the shared
  # `mix`, while OPUZ owns the exact file open/close lifecycle.
  "ApuWriteEnt" => "write",
  "MixToBuffer" => "mix",
  "MsuPlayer_CloseFile" => "finish",
  "MsuPlayer_Mix" => "mix",
  "MsuPlayer_Open" => "open",
  "ZeldaPopApuState" => "pop_into",
  "ZeldaResetApuQueue" => "reset",
  # Oracle-only byte snapshot of the translated APU queue.
  "ZeldaExportApuQueueForTest" => "oracle_snapshot",
  # Oracle-only wrapper around the already translated snapshot loader.
  "ZeldaLoadSnapshotForTest" => "load_snes_state",
  # Oracle-only reference replay loader, paired with the Rust oracle client.
  "ZeldaLoadReferenceReplayForTest" => "load_reference_replay",

  # select_file.c: reviewed state wrappers and helpers consolidated into the
  # FileSelect state machine. Static stripe builders remain local Rust code.
  "CopyFile_ChooseSelection" => "choose_copy_source",
  "CopyFile_ChooseTarget" => "choose_copy_target",
  "CopyFile_ConfirmSelection" => "confirm_copy",
  "CopyFile_HandleConfirmation" => "confirm_copy",
  "CopyFile_SelectionAndBlinker" => "choose_copy_source",
  "CopyFile_TargetSelectionAndBlink" => "choose_copy_target",
  "FilePicker_DeleteHeaderStripe" => "delete_picker_header_stripe",
  "FileSelect_TriggerNameStripesAndAdvance" => "trigger_name_stripes",
  "FileSelect_TriggerStripesAndAdvance" => "trigger_file_select_stripes",
  "Intro_CheckCksum" => "check_checksum",
  "Intro_FixCksum" => "fix_checksum",
  "KILLFile_ChooseTarget" => "choose_erase_target",
  "KILLFile_HandleConfirmation" => "confirm_erase",
  "KILLFile_HandleSelection" => "choose_erase_target",
  "KILLFile_SetUp" => "setup_picker",
  "Module01_FileSelect" => "run_file_select_module",
  "Module02_CopyFile" => "run_copy_module",
  "Module03_KILLFile" => "run_erase_module",
  "Module04_NameFile" => "run_name_module",
  "Module_CopyFile_2" => "setup_picker",
  "Module_EraseFile_1" => "build_file_select_screen",
  "Module_NamePlayer_1" => "build_checkerboard_stripe",
  "Module_NamePlayer_2" => "run_name_module",
  "Module_SelectFile_0" => "initialize_file_select",
  "NameFile_CheckForScrollInputX" => "name_scroll_input_x",
  "NameFile_CheckForScrollInputY" => "name_scroll_input_y",
  "SelectFile_Func1" => "build_checkerboard_stripe",
  "SelectFile_Func16" => "confirm_erase",
  "SelectFile_Func17" => "draw_name_and_health",
  "SelectFile_Func5_DrawOams" => "draw_file_oam",
  "SelectFile_Func6_DrawOams2" => "draw_death_counter",

  # hud.c: reviewed private helpers and state wrappers consolidated into Hud.
  "CheckPalaceItemPosession" => "check_palace_item_possession",
  "CopyTilesForSwitchLR" => "draw_y_button_items",
  "DrawHudComponents" => "rebuild",
  "GetCurrentItemButtonPtr" => "button_address",
  "Hud_DoWeHaveThisItem" => "have_item",
  "Hud_DrawItem" => "draw_item_at",
  "Hud_EquipItemAbove" => "equip_vertical",
  "Hud_EquipItemBelow" => "equip_vertical",
  "Hud_EquipNextItem" => "equip_next",
  "Hud_EquipPrevItem" => "equip_previous",
  "Hud_GetIconForItem" => "icon_for",
  "Hud_GetItemBoxPtr" => "item_box_icon",
  "Hud_GetItemPosition" => "current_item_position",
  "Hud_GotoNextItem" => "goto_next",
  "Hud_GotoPrevItem" => "goto_previous",
  "Hud_Init" => "init_menu",
  "Hud_Module_Run" => "run_module",
  "Hud_UpdateItemBox" => "update_item_box",

  "Attract_Fade" => "fade",
  "Dungeon_SaveAndLoadLoadAllPalettes" => "dungeon_save_and_load_all_palettes",
  "Do3To4High16Bit" => "do_3_to_4_high_16",
  "Do3To4Low16Bit" => "do_3_to_4_low_16",
  "GetCompSpritePtr" => "sprite",
  "Overlord_GetX" => "get_x",
  "Overlord_GetY" => "get_y",
  "Overlord_Main" => "main",
  "Overlord_SetX" => "set_x",
  "Overlord_SetY" => "set_y",
  "PaletteFilter_RestoreBGSubstractiveStrict" =>
    "palette_filter_restore_bg_subtractive_strict",
  "LinkOam_Main" => "main",
  "Follower_AnimateMovement_preserved" => "animate_movement",
  "Follower_Main" => "main",
  "Tagalong_Draw" => "draw",
  "HandleNudgingInADoor" => "nudge_in_door",
  "Hookshot_CheckSingleLayerTileCollision" => "hookshot_single_layer",
  "Hookshot_CheckTileCollision" => "hookshot_collision",
  "Overworld_GetTileAttributeAtLocation" => "overworld_tile_attribute",
  "TileCheckForMirrorBonk" => "check_mirror_bonk",
  "KillAghanim_Init" => "kill_agahnim_init",
  "KillAghanim_Func2" => "kill_agahnim_func2",
  "KillAghanim_Func3" => "kill_agahnim_func3",
  "KillAghanim_Func4" => "kill_agahnim_func4",
  "KillAghanim_Func5" => "kill_agahnim_func5",
  "KillAghanim_Func6" => "kill_agahnim_func6",
  "KillAghanim_Func7" => "kill_agahnim_func7",
  "KillAghanim_Func8" => "kill_agahnim_func8",
  "KillAghanim_Func12" => "kill_agahnim_func12",
  "SpriteSfx_QueueSfx1WithPan" => "sprite_sfx1_with_pan",
  "SpriteSfx_QueueSfx2WithPan" => "sprite_sfx2_with_pan",
  "SpriteSfx_QueueSfx3WithPan" => "sprite_sfx3_with_pan",
  "CountBits32" => "count_ones",
  "FindCmdForGamepadButton" => "find",
  "FindCmdForSdlKey" => "find_command_for_sdl_key",
  "GamepadMap_Add" => "add",
  "GetIniSection" => "section_id",
  "HandleIniConfig" => "handle",
  "KeyMapHash_Add" => "add",
  "KeyMapHash_Find" => "find",
  "ParseBoolBit" => "set_bool_bit",
  "ParseGamepadButtonName" => "parse_gamepad_button",
  "ParseOneConfigFile" => "parse_config_file",
  "RegisterDefaultKeys" => "register_defaults",
  "CopyToVram" => "write_vram_bytes",
  "CopyToVramLow" => "write_low",
  "CopyToVramVertical" => "write_vertical",
  "Interrupt_NMI_AudioParts_Locked" => "update_audio",
  "NMI_HandleArbitraryTileMap" => "arbitrary",
  "NMI_RunTileMapUpdateDMA" => "copy_and_enable",
  "NMI_TileMapNothing" => "dispatch_update",
  "NMI_UpdateBG1Wall" => "dispatch_update",
  "NMI_UpdateBG2Left" => "dispatch_update",
  "NMI_UpdateBGChar0" => "dispatch_update",
  "NMI_UpdateBGChar1" => "dispatch_update",
  "NMI_UpdateBGChar2" => "dispatch_update",
  "NMI_UpdateBGChar3" => "dispatch_update",
  "NMI_UpdateBGChar3and4" => "dispatch_update",
  "NMI_UpdateBGChar5and6" => "dispatch_update",
  "NMI_UpdateBGCharHalf" => "dispatch_update",
  "NMI_UpdateLoadLightWorldMap" => "light_map",
  "NMI_UpdateObjChar0" => "dispatch_update",
  "NMI_UpdateObjChar2" => "dispatch_update",
  "NMI_UpdateObjChar3" => "dispatch_update",
  "NMI_UpdatePegTiles" => "dispatch_update",
  "NMI_UpdateStarTiles" => "dispatch_update",
  "NMI_UpdateSubscreenOverlay" => "dispatch_update",
  "NMI_UploadBG3Text" => "dispatch_update",
  "NMI_UploadDarkWorldMap" => "dark_map",
  "NMI_UploadGameOverText" => "dispatch_update",
  "NMI_UploadSubscreenOverlayFormer" => "dispatch_update",
  "NMI_UploadSubscreenOverlayLatter" => "dispatch_update",
  "NMI_UploadTilemap_doNothing" => "dispatch_update",
  "apu_free" => "default",
  "apu_init" => "new",
  "cart_free" => "default",
  "cart_init" => "new",
  "cart_load" => "load",
  "cart_read" => "read",
  "dma_free" => "default",
  "dma_init" => "new",
  "dma_read" => "read_register",
  "input_free" => "default",
  "input_init" => "new",
  "input_read" => "read",
  "cpu_adrAbl" => "addr",
  "cpu_adrAbs" => "addr",
  "cpu_adrAbx" => "addr",
  "cpu_adrAby" => "addr",
  "cpu_adrAlx" => "addr",
  "cpu_adrDp" => "addr",
  "cpu_adrDpx" => "addr",
  "cpu_adrDpy" => "addr",
  "cpu_adrIax" => "execute_special",
  "cpu_adrIdl" => "addr",
  "cpu_adrIdp" => "addr",
  "cpu_adrIdx" => "addr",
  "cpu_adrIdy" => "addr",
  "cpu_adrIly" => "addr",
  "cpu_adrImm" => "addr",
  "cpu_adrIsy" => "addr",
  "cpu_adrSr" => "addr",
  "cpu_asl" => "mem_op",
  "cpu_dec" => "mem_op",
  "cpu_doOpcode" => "run_opcode",
  "cpu_free" => "default",
  "cpu_inc" => "mem_op",
  "cpu_init" => "new",
  "cpu_lsr" => "mem_op",
  "cpu_rol" => "mem_op",
  "cpu_ror" => "mem_op",
  "cpu_tsb" => "tsb_trb",
  "dsp_decodeBrr" => "brr",
  "dsp_free" => "default",
  "dsp_handleEcho" => "echo",
  "dsp_handleGain" => "gain",
  "dsp_init" => "new",
  "FloatInterpolate" => "draw_mode7_upsampled",
  "PpuDrawBackground_2bpp" => "bg_pixel",
  "PpuDrawBackground_4bpp" => "bg_pixel",
  "PpuDrawBackground_mode7" => "mode7_pixel",
  "PpuDrawBackgrounds" => "pixel",
  "PpuDrawSprites" => "evaluate_sprites",
  "PpuDrawWholeLine" => "run_line",
  "PpuWindows_Calc" => "window_state",
  "PpuWindows_Clear" => "run_line",
  "for" => "run_line",
  "ppu_free" => "default",
  "ppu_getPixelForBgLayer" => "bg_pixel",
  "ppu_getPixelForMode7" => "mode7_pixel",
  "ppu_init" => "new",
  "spc_adc" => "add",
  "spc_adcm" => "alum",
  "spc_adrAbs" => "abs",
  "spc_adrAbsBit" => "abit",
  "spc_adrAbx" => "abx",
  "spc_adrAby" => "aby",
  "spc_adrDp" => "dp",
  "spc_adrDpDp" => "dpdp",
  "spc_adrDpWord" => "dpw",
  "spc_adrDpx" => "dpx",
  "spc_adrDpy" => "dpy",
  "spc_adrIdx" => "idx",
  "spc_adrIdy" => "idy",
  "spc_adrImm" => "imm",
  "spc_adrInd" => "ind",
  "spc_adrIndInd" => "ii",
  "spc_adrIndP" => "indp",
  "spc_and" => "alua",
  "spc_andm" => "alum",
  "spc_asl" => "shift",
  "spc_cmp" => "cmpreg",
  "spc_cmpm" => "alum",
  "spc_cmpx" => "cmpreg",
  "spc_cmpy" => "cmpreg",
  "spc_dec" => "incm",
  "spc_doOpcode" => "run_opcode",
  "spc_eor" => "alua",
  "spc_eorm" => "alum",
  "spc_free" => "default",
  "spc_inc" => "incm",
  "spc_init" => "new",
  "spc_lsr" => "shift",
  "spc_mov" => "load",
  "spc_movs" => "store",
  "spc_movsx" => "store",
  "spc_movsy" => "store",
  "spc_movx" => "load",
  "spc_movy" => "load",
  "spc_or" => "alua",
  "spc_orm" => "alum",
  "spc_pullByte" => "pull",
  "spc_pullWord" => "pullw",
  "spc_pushByte" => "push",
  "spc_pushWord" => "pushw",
  "spc_readOpcode" => "ro",
  "spc_readOpcodeWord" => "row",
  "spc_readWord" => "rw",
  "spc_rol" => "shift",
  "spc_ror" => "shift",
  "spc_sbc" => "add",
  "spc_sbcm" => "alum",
  "spc_setZN" => "zn",
  "spc_writeWord" => "ww",
  "snes_cpuRead" => "read",
  "snes_free" => "default",
  "snes_getAccessTime" => "read",
  "snes_init" => "new",
  "snes_printCpuLine" => "run_cpu_opcode",
  "snes_readReg" => "read_register",
  "snes_rread" => "read_raw",
  "snes_writeReg" => "write_register",
  "readHeader" => "read",
  "Asl" => "port_start_new_sound",
  "CalcTremolo" => "chan_handle_tick",
  "Chan_DoAnyFade" => "fade",
  # The C-only debug comparator is behind WITH_SPC_PLAYER_DEBUGGING=0; Rust's
  # executable differential verifier supersedes it for active builds.
  "CompareSpcImpls" => "copy_variables_to_ram",
  "HandleTremolo" => "chan_handle_tick",
  "Interrupt_Reset" => "initialize",
  "Not_Implemented" => "chan_handle_tick",
  "Port2_AllocateChan" => "allocate_sfx_channel",
  "Port2_HandleCmd" => "port_handle_cmd",
  "Port2_StartNewSound" => "port_start_new_sound",
  "Port3_AllocateChan" => "allocate_sfx_channel",
  "Port3_HandleCmd" => "port_handle_cmd",
  "Port3_StartNewSound" => "port_start_new_sound",
  "RunAudioPlayer" => "generate_samples",
  "SpcPlayer_Create" => "new",
  # The recorder's LogWriter owns the original 255-extension append loop.
  "ByteArray_AppendVl" => "variable_length",
  "EmuSyncMemoryRegion" => "import_canonical_state",
  "EmuSynchronizeWholeState" => "canonical_state",
  "IncrementCrystalCountdown" => "determine_run_what",
  "InputStateReadFromFile" => "read_next_replay_state",
  "ReadFromFile" => "decode_state_file",
  "SaveLoadSlot" => "save_slot_path",
  "SimpleHdma_GetPtr" => "resolve_hdma_pointer",
  "SimpleHdma_Init" => "from_channel",
  "StateRecoderMultiPatch_Init" => "new",
  "StateRecorder_Init" => "default",
  "StateRecorder_Load" => "decode_state_file",
  "StateRecorder_RecordCmd" => "record_command",
  "StateRecorder_RecordPatchByte" => "record_patch",
  "StateRecorder_Save" => "encode_state_file",
  "ZeldaReadSram" => "read_sram",
  "ZeldaRunGameLoop" => "run_frame_internal",
  "ZeldaSetupEmuCallbacks" => "run_core_frame",
  "ZeldaWriteSram" => "write_sram",
  "loadFunc" => "decode_state_file",
  "saveFunc" => "encode_state_file",
  "zelda_ppu_write" => "write",
  "zelda_ppu_write_word" => "write",
  "EmuInitialize" => "spawn",
  "EmuRunFrameWithCompare" => "step",
  "GetCartRamPtr" => "canonical_state",
  "GetPtr" => "region",
  "HookedFunctionRts" => "run_opcode",
  "MakeMySnapshot" => "canonical_state",
  "MakeSnapshot" => "encode",
  "PatchRom" => "write",
  "PatchRomArray" => "write",
  "PatchRomBP" => "write",
  "PatchRomByte" => "write",
  "PatchRomWord" => "write",
  "RestoreMySnapshot" => "import_canonical_state",
  "RestoreSnapshot" => "parse",
  "RomByte" => "read",
  "RunEmulatedFunc" => "run_opcode",
  "RunEmulatedFuncSilent" => "run_opcode",
  "RunEmulatedSnesFrame" => "step",
  "RunOrigAsmCodeOneLoop" => "step",
  "VerifySnapshotsEq" => "first_difference",
  "DrawPpuFrameWithPerf" => "draw",
  "FindInAssetArray" => "bytes",
  "HandleCommand_Locked" => "handle_command",
  "HandleGamepadAxisInput" => "handle_stick_axis",
  "HandleGamepadInput" => "handle_gamepad_button",
  "LoadAssets" => "load_asset_bundle",
  "LoadLinkGraphics" => "parse_link_graphics",
  "LoadRom" => "load_rom",
  "RemapSdlButton" => "find_command_for_sdl_key",
  "SwitchDirectory" => "find_project_directory",
}.freeze

def normalized(name)
  name.downcase.delete("_")
end

def translated_name?(c_name, rust_names)
  c = normalized(c_name)
  reviewed = EQUIVALENT_NAMES[c_name]
  return true if reviewed && rust_names.include?(reviewed)

  rust_names.any? do |rust_name|
    r = normalized(rust_name)
    # The C SNES core uses a translation-unit prefix for methods whose Rust
    # receiver already supplies that namespace (`cpu_adc` -> `Cpu::adc`).
    # This is an exact namespace abstraction, not fuzzy name acceptance.
    variants = [c]
    %w[apu cart cpu dma dsp input ppu snes spc].each do |prefix|
      variants << c.delete_prefix(prefix) if c.start_with?(prefix) && c.length > prefix.length
    end
    variants.any? do |variant|
      variant == r || (r.length >= 5 && (variant.end_with?(r) || r.end_with?(variant)))
    end
  end
end

def c_functions(path)
  output = IO.popen(["ctags", "-x", path], &:read)
  raise "ctags failed for #{path}" unless $?.success?

  output.lines.each_with_object([]) do |line, functions|
    fields = line.split(nil, 4)
    if fields.length == 4 && fields[3].include?("{") && !fields[3].include?("#define")
      functions << fields[0]
    end
  end
end

def rust_functions(paths)
  paths.flat_map do |path|
    next [] unless File.file?(path)

    File.read(path).scan(/(?:pub\s+)?(?:const\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/).flatten
  end
end

total = 0
translated = 0
all_missing = {}

Dir.chdir(ROOT) do
  PAIRS.each do |c_path, rust_paths|
    source = c_functions(c_path)
    rust = rust_functions(rust_paths)
    missing = source.reject { |name| translated_name?(name, rust) }
    total += source.length
    translated += source.length - missing.length
    all_missing[c_path] = missing unless missing.empty?
    puts format("%-24s %4d/%-4d missing %d", c_path, source.length - missing.length, source.length, missing.length)
  end
end

puts format("TOTAL                    %4d/%-4d missing %d", translated, total, total - translated)
if ARGV.include?("--list")
  all_missing.each do |path, names|
    puts "\n#{path}:"
    names.each { |name| puts "  #{name}" }
  end
end

exit(all_missing.empty? ? 0 : 1)
