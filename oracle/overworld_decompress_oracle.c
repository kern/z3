#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "src/overworld.h"
#include "src/assets.h"

static uint64_t hash_bytes(const uint8_t *bytes, size_t size) {
  uint64_t hash = UINT64_C(0xcbf29ce484222325);
  for (size_t i = 0; i < size; i++) {
    hash ^= bytes[i];
    hash *= UINT64_C(0x100000001b3);
  }
  return hash;
}

static void write64(uint64_t value) {
  for (int i = 0; i < 8; i++)
    if (putchar((uint8_t)(value >> (i * 8))) == EOF)
      _Exit(1);
}

static void write16(uint16_t value) {
  if (putchar((uint8_t)value) == EOF || putchar((uint8_t)(value >> 8)) == EOF)
    _Exit(1);
}

static uint8_t *emit_short(uint8_t *p, int type, int length) {
  *p++ = (uint8_t)((type << 5) | (length - 1));
  return p;
}

static uint8_t *emit_long(uint8_t *p, int type, int length) {
  int encoded = length - 1;
  *p++ = (uint8_t)(0xe0 | (type << 2) | (encoded >> 8));
  *p++ = (uint8_t)encoded;
  return p;
}

int main(void) {
  for (int scenario = 0; scenario < 256; scenario++) {
    uint8_t source[2048], output[4096];
    uint8_t *p = source;

    p = emit_short(p, 0, 32);
    for (int i = 0; i < 32; i++)
      *p++ = (uint8_t)(scenario + i * 17);

    int lengths[4] = {
      1 + (scenario & 31),
      1 + (scenario * 3 & 31),
      1 + (scenario * 5 & 31),
      1 + (scenario * 7 & 31),
    };
    p = emit_short(p, 1, lengths[0]); *p++ = (uint8_t)(scenario ^ 0xa5);
    p = emit_short(p, 2, lengths[1]); *p++ = (uint8_t)scenario; *p++ = (uint8_t)~scenario;
    p = emit_short(p, 3, lengths[2]); *p++ = (uint8_t)(scenario * 11);
    p = emit_short(p, 4, lengths[3]); *p++ = 0; *p++ = (uint8_t)(scenario & 15);

    int type = scenario % 5;
    int length = 40 + scenario % 240 * 4;
    p = emit_long(p, type, length);
    if (type == 0) {
      for (int i = 0; i < length; i++)
        *p++ = (uint8_t)(scenario * 29 + i * 13);
    } else if (type == 1 || type == 3) {
      *p++ = (uint8_t)(scenario * 19 + 3);
    } else if (type == 2) {
      *p++ = (uint8_t)(scenario + 7);
      *p++ = (uint8_t)(scenario - 9);
    } else {
      *p++ = 0;
      *p++ = (uint8_t)(scenario & 31);
    }
    *p++ = 0xff;

    int written = Decompress_bank02(output, source);
    uint64_t hash = hash_bytes((const uint8_t *)&written, sizeof(written));
    hash ^= hash_bytes(output, (size_t)written);
    write64(hash);
  }
  for (uint32_t address = 0; address <= UINT16_MAX; address++)
    write16(Overworld_FindMap16VRAMAddress((uint16_t)address));
  for (int big = 0; big < 2; big++) {
    for (int area = 0; area < 64; area++) {
      memset(g_ram + 0x600, 0, 0x18);
      Overworld_SetCameraBoundaries(big, area);
      if (fwrite(g_ram + 0x600, 1, 0x18, stdout) != 0x18)
        return 1;
    }
  }
  for (int indoors = 0; indoors < 2; indoors++) {
    for (int health = 0; health < 256; health++) {
      memset(g_ram, 0, 0x10000);
      g_ram[0x1b] = (uint8_t)indoors;
      g_ram[0xf36d] = (uint8_t)health;
      TakeDamageFromPit();
      if (putchar(g_ram[0x4b]) == EOF || putchar(g_ram[0x11]) == EOF ||
          putchar(g_ram[0xf36d]) == EOF)
        return 1;
    }
  }
  for (int feature = 0; feature < 2; feature++) {
    memset(g_ram, 0xa5, 0x10000);
    g_ram[0x64c] = feature ? 4 : 0;
    ForceNonbunnyStatus();
    static const uint16_t kOffsets[] = {0x5d, 0x3f5, 0x3f6, 0x3f7, 0x56, 0x2e0, 0x372};
    for (size_t i = 0; i < sizeof(kOffsets) / sizeof(kOffsets[0]); i++)
      if (putchar(g_ram[kOffsets[i]]) == EOF)
        return 1;
  }
  static const uint16_t kPegProgress[] = {0, 2, 4, 0xffff};
  static const uint16_t kPegPositions[] = {0x826, 0x5a0, 0x81a, 0x1234};
  for (int completed = 0; completed < 2; completed++) {
    for (size_t i = 0; i < sizeof(kPegProgress) / sizeof(kPegProgress[0]); i++) {
      for (size_t j = 0; j < sizeof(kPegPositions) / sizeof(kPegPositions[0]); j++) {
        memset(g_ram, 0, 0x10000);
        overworld_screen_index = 7;
        word_7E04C8 = kPegProgress[i];
        save_ow_event_info[7] = completed ? 0x20 : 0;
        HandlePegPuzzles(kPegPositions[j]);
        write16(word_7E04C8);
        write16(WORD(sound_effect_1));
        if (putchar(save_ow_event_info[7]) == EOF || putchar(submodule_index) == EOF)
          return 1;
      }
    }
  }
  for (int progress = 0; progress <= 30; progress++) {
    if (progress == 21)
      continue;  // Completion dispatch needs the fully initialized map asset runtime.
    memset(g_ram, 0, 0x10000);
    overworld_screen_index = 98;
    word_7E04C8 = progress;
    HandlePegPuzzles(0);
    write16(word_7E04C8);
    write16(door_open_closed_counter);
    write16(big_rock_starting_address);
    if (putchar(save_ow_event_info[0x62]) == EOF || putchar(sound_effect_2) == EOF)
      return 1;
  }
  memset(g_ram, 0, 0x20000);
  for (int i = 0; i < 256; i++)
    aux_palette_buffer[i] = (uint16_t)(i ^ 0x4210);
  main_palette_buffer[0] = 0x1234;
  main_palette_buffer[32] = 0xabcd;
  Palette_AnimGetMasterSword2();
  write64(hash_bytes(g_ram + 0xc300, 0x400));
  write64(hash_bytes(g_ram + 0x1dd80, 0x200));
  write16(main_palette_buffer[32]);
  write16(palette_filter_countdown);
  write16(darkening_or_lightening_screen);
  if (putchar(subsubmodule_index) == EOF)
    return 1;
  darkening_or_lightening_screen = 0;
  palette_filter_countdown = 31;
  TS_copy = 0xa5;
  Palette_AnimGetMasterSword3();
  write64(hash_bytes(g_ram + 0xc300, 0x200));
  if (putchar(TS_copy) == EOF)
    return 1;
  static const uint8_t kScreens[] = {0x42, 0x43, 0x45, 0x47};
  static const uint8_t kFrames[] = {3, 5, 7, 36, 44, 88, 90};
  for (size_t s = 0; s < sizeof(kScreens); s++) {
    for (size_t f = 0; f < sizeof(kFrames); f++) {
      for (int event = 0; event < 2; event++) {
        for (int trigger = 0; trigger < 2; trigger++) {
          memset(g_ram, 0, 0x20000);
          overworld_screen_index = kScreens[s];
          frame_counter = kFrames[f];
          trigger_special_entrance = (uint8_t)trigger;
          save_ow_event_info[0x43] = event ? 0x20 : 0;
          for (int i = 0; i < 256; i++)
            aux_palette_buffer[i] = (uint16_t)(i * 73 + 11);
          Overworld_DwDeathMountainPaletteAnimation();
          write64(hash_bytes(g_ram + 0xc500, 0x200));
          if (putchar(flag_update_cgram_in_nmi) == EOF || putchar(sound_effect_1) == EOF)
            return 1;
        }
      }
    }
  }
  static uint16_t map16_to_map8[32 * 4];
  static uint8_t map16_source[0x2000];
  for (int tile = 0; tile < 32; tile++) {
    for (int quadrant = 0; quadrant < 4; quadrant++)
      map16_to_map8[tile * 4 + quadrant] = (uint16_t)(tile * 4 + quadrant);
    map16_source[tile * 2] = (uint8_t)tile;
    map16_source[tile * 2 + 1] = 0;
  }
  g_asset_ptrs[70] = (const uint8_t *)map16_to_map8;
  g_asset_sizes[70] = sizeof(map16_to_map8);
  memset(g_ram, 0, 0x20000);
  map16_load_src_off = 0x410;
  map16_load_dst_off = 31;
  map16_load_var2 = 0x12;
  OverworldCopyMap16ToBuffer(map16_source, 0x1000, 0, &word_7F4000);
  write64(hash_bytes(g_ram + 0x500, 0x40));
  write64(hash_bytes(g_ram + 0x12000, 0x100));
  write64(hash_bytes(g_ram + 0x14000, 4));
  for (int screen = 0; screen < 128; screen++) {
    memset(g_ram, 0, 0x10000);
    overworld_screen_index = (uint16_t)screen;
    save_ow_event_info[screen] = 2;
    Overworld_HandleOverlaysAndBombDoors();
    write64(hash_bytes(g_ram + 0x2000, 0x2000));
  }
  memset(g_ram, 0, 0x20000);
  for (int i = 0; i < 32; i++)
    dung_bg2[i * 64] = (uint16_t)i;
  BYTE(overworld_screen_trans_dir_bits2) = 1;
  map16_load_src_off = 0x3d0;
  uint16_t *stripe_end = BufferAndBuildMap16Stripes_X(uvram.data);
  write16((uint16_t)(stripe_end - uvram.data));
  write64(hash_bytes(g_ram + 0x1100, 264));
  memset(g_ram, 0, 0x20000);
  for (int i = 0; i < 32; i++)
    dung_bg2[i] = (uint16_t)i;
  BYTE(overworld_screen_trans_dir_bits2) = 4;
  map16_load_src_off = 0xf410;
  stripe_end = BufferAndBuildMap16Stripes_Y(uvram.data);
  write16((uint16_t)(stripe_end - uvram.data));
  write64(hash_bytes(g_ram + 0x1100, 260));
  static const uint16_t kCameraAccum[] = {0, 15, 0xffff};
  for (int axis = 0; axis < 4; axis++) {
    for (int negative = 0; negative < 2; negative++) {
      for (size_t a = 0; a < sizeof(kCameraAccum) / sizeof(kCameraAccum[0]); a++) {
        for (int at_boundary = 0; at_boundary < 2; at_boundary++) {
          memset(g_ram, 0, 0x10000);
          int xa = axis < 2 ? 6 : 0;
          int ya = axis * 2;
          int r8 = axis < 2 ? 0 : 4;
          uint16_t *bg = xa ? &BG2VOFS_copy2 : &BG2HOFS_copy2;
          uint16_t *boundary = &(&ow_scroll_vars0.ystart)[axis];
          *bg = 0x345;
          *boundary = at_boundary ? 0x345 : 0x567;
          (&overworld_unk1)[axis] = kCameraAccum[a];
          (&camera_y_coord_scroll_hi)[r8 >> 1] = 0x234;
          int result = OverworldCameraBoundaryCheck(xa, ya, negative ? -1 : 1, r8);
          write16((uint16_t)result);
          write64(hash_bytes(g_ram + 0xe0, 10));
          write64(hash_bytes(g_ram + 0x600, 0x2c));
          write64(hash_bytes(g_ram + 0x416, 4));
        }
      }
    }
  }
  static const uint8_t kTransitionThresholds[4] = {27, 27, 30, 30};
  for (int transition = 0; transition < 4; transition++) {
    for (int near_threshold = 0; near_threshold < 2; near_threshold++) {
      memset(g_ram, 0, 0x10000);
      overworld_screen_transition = (uint16_t)transition;
      transition_counter = near_threshold ? (uint8_t)(kTransitionThresholds[transition] - 1) : 0;
      BG2VOFS_copy2 = BG2HOFS_copy2 = 0x1000;
      link_y_coord = 0x234;
      link_x_coord = 0x456;
      (&up_down_scroll_target)[transition] = 0xffff;
      int result = OverworldScrollTransition();
      write16((uint16_t)result);
      write64(hash_bytes(g_ram, 0x1000));
    }
  }
  static const uint8_t kCameraOverlays[] = {0, 0x95, 0x97, 0x9c, 0xb5};
  static const uint8_t kCameraVelocities[] = {0, 1, 2, 0xff};
  for (size_t o = 0; o < sizeof(kCameraOverlays); o++) {
    for (size_t yv = 0; yv < sizeof(kCameraVelocities); yv++) {
      for (size_t xv = 0; xv < sizeof(kCameraVelocities); xv++) {
        memset(g_ram, 0, 0x10000);
        overlay_index = kCameraOverlays[o];
        overworld_screen_index = 5;
        link_y_coord = 0x220;
        link_x_coord = 0x330;
        link_y_vel = kCameraVelocities[yv];
        link_x_vel = kCameraVelocities[xv];
        link_z_coord = 4;
        allow_scroll_z = 1;
        camera_y_coord_scroll_low = 0x300;
        camera_y_coord_scroll_hi = 0x100;
        camera_x_coord_scroll_low = 0x400;
        camera_x_coord_scroll_hi = 0x100;
        BG2VOFS_copy2 = 0x345;
        BG2HOFS_copy2 = 0x456;
        ow_scroll_vars0.ystart = 0x111;
        ow_scroll_vars0.yend = 0x222;
        ow_scroll_vars0.xstart = 0x333;
        ow_scroll_vars0.xend = 0x444;
        BG1VOFS_subpixel = 0x1234;
        BG1VOFS_copy2 = 0x567;
        BG1HOFS_subpixel = 0x4321;
        BG1HOFS_copy2 = 0x765;
        Overworld_OperateCameraScroll();
        write64(hash_bytes(g_ram, 0x700));
      }
    }
  }
  memset(g_ram, 0, 0x20000);
  overworld_area_index_exit = 0x23;
  WORD(TM_copy_exit) = 0x1514;
  BG2VOFS_copy2_exit = 0x222;
  BG2HOFS_copy2_exit = 0x444;
  link_y_coord_exit = 0x666;
  link_x_coord_exit = 0x888;
  overworld_screen_index_exit = 0x2a;
  map16_load_src_off_exit = 0x790;
  camera_y_coord_scroll_low_exit = 0x321;
  camera_x_coord_scroll_low_exit = 0x543;
  for (int i = 0; i < 4; i++) {
    (&ow_scroll_vars0_exit.ystart)[i] = 0x1000 + i * 2;
    (&up_down_scroll_target_exit)[i] = 0x2000 + i * 2;
    (&overworld_unk1_exit)[i] = 0x3000 + i * 2;
  }
  byte_7EC164 = 0x51;
  main_tile_theme_index_exit = 0x52;
  aux_tile_theme_index_exit = 0x53;
  sprite_graphics_index_exit = 0x54;
  dungeon_room_index = 0x130;
  LoadCachedEntranceProperties();
  write64(hash_bytes(g_ram, 0x1000));
  memset(g_ram, 0xa5, 0x1000);
  OverworldEntrance_PlayJingle();
  write64(hash_bytes(g_ram, 0x1000));
  memset(g_ram, 0xa5, 0x1000);
  OverworldEntrance_AdvanceAndBoom();
  write64(hash_bytes(g_ram, 0x1000));
  memset(g_ram, 0xa5, 0x1000);
  INIDISP_copy = 14;
  Overworld_Func22();
  write64(hash_bytes(g_ram, 0x1000));
  memset(g_ram, 0xa5, 0x1000);
  subsubmodule_index = 2;
  overworld_screen_index = 0x40;
  palette_filter_countdown = 1;
  mosaic_level = 0x30;
  Overworld_Func1C();
  write64(hash_bytes(g_ram, 0x1000));
  static const uint16_t kFixedColorScreens[] = {0, 3, 0x43, 0x5b, 0x70, 0x80, 0x81, 0x1b};
  for (size_t s = 0; s < sizeof(kFixedColorScreens) / sizeof(kFixedColorScreens[0]); s++) {
    for (int sub = 0; sub < 2; sub++) {
      for (int dir = 0; dir < 2; dir++) {
        for (int room = 0; room < 2; room++) {
          memset(g_ram, 0, 0x20000);
          overworld_screen_index = kFixedColorScreens[s];
          submodule_index = sub ? 4 : 0;
          BYTE(overworld_screen_trans_dir_bits) = dir ? 8 : 0;
          dungeon_room_index = room ? 0x181 : 0;
          BG2VOFS_copy2 = 0x712;
          BG2HOFS_copy2 = 0x756;
          flag_update_cgram_in_nmi = 0xfe;
          Overworld_SetFixedColAndScroll();
          write64(hash_bytes(g_ram, 0x20000));
        }
      }
    }
  }
  memset(g_ram, 0, 0x20000);
  subsubmodule_index = 2;
  overworld_screen_index = 0x80;
  main_module_index = 11;
  palette_filter_countdown = 1;
  mosaic_level = 0x30;
  music_unk1 = 0x27;
  Module09_FadeBackInFromMosaic();
  write64(hash_bytes(g_ram, 0x20000));
  static const uint16_t kSpotlightRocks[] = {0, 0x1234, 0x9234};
  for (int room = 0; room < 2; room++) {
    for (int facing = 0; facing < 2; facing++) {
      for (size_t rock = 0; rock < sizeof(kSpotlightRocks) / sizeof(kSpotlightRocks[0]); rock++) {
        for (int dark = 0; dark < 2; dark++) {
          memset(g_ram, 0xa5, 0x1000);
          main_module_index = 9;
          dungeon_room_index = room ? 0x20 : 0x21;
          link_direction_facing = facing ? 2 : 0;
          big_rock_starting_address = kSpotlightRocks[rock];
          ow_entrance_value = rock ? 1 : 0;
          overworld_screen_index = dark ? 0x43 : 3;
          OpenSpotlight_Next2();
          write64(hash_bytes(g_ram, 0x1000));
        }
      }
    }
  }
  memset(g_ram, 0xa5, 0x1000);
  Overworld_FinishTransGfx();
  write64(hash_bytes(g_ram, 0x1000));
  memset(g_ram, 0xa5, 0x1000);
  BYTE(door_animation_step_indicator) = 3;
  Module09_09_OpenBigDoorFromExiting();
  write64(hash_bytes(g_ram, 0x1000));
  memset(g_ram, 0xa5, 0x1000);
  BYTE(door_animation_step_indicator) = 3;
  Module09_0C_OpenBigDoor();
  write64(hash_bytes(g_ram, 0x1000));
  memset(g_ram, 0xa5, 0x1000);
  door_open_closed_counter = 5;
  Overworld_DoMapUpdate32x32_conditional();
  write64(hash_bytes(g_ram, 0x1000));
  static uint16_t door_map16[0x4000];
  for (size_t i = 0; i < sizeof(door_map16) / sizeof(door_map16[0]); i++)
    door_map16[i] = (uint16_t)i;
  g_asset_ptrs[70] = (const uint8_t *)door_map16;
  g_asset_sizes[70] = sizeof(door_map16);
  memset(g_ram, 0, 0x20000);
  num_memorized_tiles = 4;
  big_rock_starting_address = 0x420;
  door_open_closed_counter = 8;
  Overworld_DoMapUpdate32x32();
  write64(hash_bytes(g_ram, 0x20000));
  static const uint16_t kOverlayGetoutRooms[] = {0x180, 0x182, 0x183, 0x185};
  for (size_t room = 0; room < sizeof(kOverlayGetoutRooms) / sizeof(kOverlayGetoutRooms[0]); room++) {
    memset(g_ram, 0, 0x20000);
    overworld_screen_index = 0x80;
    dungeon_room_index = kOverlayGetoutRooms[room];
    save_ow_event_info[0x80] = 0x40;
    map16_load_src_off = 0x1234;
    map16_load_var2 = 0x5678;
    map16_load_dst_off = 0x9abc;
    overworld_screen_transition = 3;
    overworld_screen_trans_dir_bits = 4;
    overworld_screen_trans_dir_bits2 = 8;
    Overworld_LoadOverlays2();
    write64(hash_bytes(g_ram, 0x20000));
  }
  memset(g_ram, 0, 0x20000);
  for (int i = 0; i < 256; i++)
    aux_palette_buffer[i] = (uint16_t)(i * 17 + 3);
  GanonTowerEntrance_Func1();
  write64(hash_bytes(g_ram, 0x20000));
  static uint16_t secret_offsets[128];
  static uint8_t secrets[] = {0x34, 0x12, 0x82, 0xff, 0xff};
  g_asset_ptrs[157] = (const uint8_t *)secret_offsets;
  g_asset_sizes[157] = sizeof(secret_offsets);
  g_asset_ptrs[158] = secrets;
  g_asset_sizes[158] = sizeof(secrets);
  for (int scenario = 0; scenario < 3; scenario++) {
    memset(g_ram, 0, 0x20000);
    overworld_screen_index = 2;
    secrets[2] = scenario == 2 ? 5 : 0x82;
    if (scenario == 1) {
      save_ow_event_info[2] = 2;
      enhanced_features0 = kFeatures0_MiscBugFixes;
      link_item_in_hand = 0x40;
    }
    write16(Overworld_RevealSecret(0x1234));
    write64(hash_bytes(g_ram, 0x20000));
  }
  static uint8_t map_is_small[128];
  static uint8_t aux_themes[128];
  for (int i = 0; i < 128; i++) {
    map_is_small[i] = (i % 3) == 0;
    aux_themes[i] = (uint8_t)(i * 7 + 3);
  }
  g_asset_ptrs[107] = map_is_small;
  g_asset_sizes[107] = sizeof(map_is_small);
  g_asset_ptrs[108] = aux_themes;
  g_asset_sizes[108] = sizeof(aux_themes);
  static const uint8_t gfx_screens[] = {5, 0x45};
  for (size_t i = 0; i < sizeof(gfx_screens); i++) {
    memset(g_ram, 0xa5, 0x20000);
    overworld_screen_index = gfx_screens[i];
    overworld_sprite_gfx[gfx_screens[i]] = (uint8_t)(0x31 + i);
    overworld_area_is_big = 0x20a5;
    Overworld_LoadGFXAndScreenSize();
    write64(hash_bytes(g_ram, 0x20000));
  }
  static uint16_t entrance_map16[4] = {0, 0, 0xfe, 0x14a};
  static uint16_t entrance_areas[129];
  static uint16_t entrance_positions[129];
  static uint8_t entrance_ids[129];
  entrance_ids[128] = 9;
  g_asset_ptrs[70] = (const uint8_t *)entrance_map16;
  g_asset_sizes[70] = sizeof(entrance_map16);
  g_asset_ptrs[124] = (const uint8_t *)entrance_areas;
  g_asset_sizes[124] = sizeof(entrance_areas);
  g_asset_ptrs[125] = (const uint8_t *)entrance_positions;
  g_asset_sizes[125] = sizeof(entrance_positions);
  g_asset_ptrs[126] = entrance_ids;
  g_asset_sizes[126] = sizeof(entrance_ids);
  memset(g_ram, 0, 0x20000);
  link_direction_facing = 2;
  Overworld_UseEntrance();
  write64(hash_bytes(g_ram, 0x20000));
  return fflush(stdout) == 0 ? 0 : 1;
}
