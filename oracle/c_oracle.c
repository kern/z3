#include "oracle/c_oracle.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "src/assets.h"
#include "src/audio.h"
#include "src/config.h"
#include "src/types.h"
#include "src/zelda_rtl.h"
#include "src/spc_player.h"
#include "snes/snes.h"

const uint8 *g_asset_ptrs[kNumberOfAssets];
uint32 g_asset_sizes[kNumberOfAssets];
Config g_config;

static uint8 *g_oracle_assets;
static int g_initialized;

void NORETURN Die(const char *error) {
  fprintf(stderr, "C oracle fatal error: %s\n", error);
  abort();
}

MemBlk FindInAssetArray(int asset, int idx) {
  return FindIndexInMemblk(
      (MemBlk){g_asset_ptrs[asset], g_asset_sizes[asset]}, idx);
}

void ZeldaApuLock(void) {}
void ZeldaApuUnlock(void) {}

// The native Zelda path keeps the DMA register model but performs its game
// transfers directly.  Retaining these fail-fast bus hooks lets us link the
// exact C DMA implementation without silently accepting an unexpected entry
// into the full SNES-emulator bus.
uint8_t snes_read(Snes *snes, uint32_t adr) {
  (void)snes;
  (void)adr;
  Die("unexpected SNES bus read in native oracle");
  return 0;
}

void snes_write(Snes *snes, uint32_t adr, uint8_t val) {
  (void)snes;
  (void)adr;
  (void)val;
  Die("unexpected SNES bus write in native oracle");
}

uint8_t snes_readBBus(Snes *snes, uint8_t adr) {
  (void)snes;
  (void)adr;
  Die("unexpected SNES B-bus read in native oracle");
  return 0;
}

void snes_writeBBus(Snes *snes, uint8_t adr, uint8_t val) {
  (void)snes;
  (void)adr;
  (void)val;
  Die("unexpected SNES B-bus write in native oracle");
}

static uint32 read_u32_le(const uint8 *source) {
  return (uint32)source[0] |
         (uint32)source[1] << 8 |
         (uint32)source[2] << 16 |
         (uint32)source[3] << 24;
}

static int load_assets(const uint8 *source, size_t length) {
  static const uint8 expected_signature[] = { kAssets_Sig };
  const size_t size_table_end = 88 + kNumberOfAssets * 4;
  if (source == NULL || length < size_table_end ||
      memcmp(source, expected_signature, sizeof(expected_signature)) != 0 ||
      read_u32_le(source + 80) != kNumberOfAssets)
    return 0;

  uint8 *copy = (uint8 *)malloc(length);
  if (copy == NULL)
    return 0;
  memcpy(copy, source, length);

  size_t offset = size_table_end + read_u32_le(copy + 84);
  if (offset > length) {
    free(copy);
    return 0;
  }
  for (size_t i = 0; i < kNumberOfAssets; i++) {
    size_t size = read_u32_le(copy + 88 + i * 4);
    offset = (offset + 3) & ~(size_t)3;
    if (offset > length || size > length - offset) {
      free(copy);
      return 0;
    }
    g_asset_sizes[i] = (uint32)size;
    g_asset_ptrs[i] = copy + offset;
    offset += size;
  }

  free(g_oracle_assets);
  g_oracle_assets = copy;
  return 1;
}

int z3c_initialize(const uint8_t *assets, size_t assets_len) {
  if (g_initialized || !load_assets(assets, assets_len))
    return 0;

  memset(&g_config, 0, sizeof(g_config));
  g_config.audio_freq = 32000;
  g_config.audio_channels = 2;
  g_config.msuvolume = 100;

  ZeldaInitialize();
  ZeldaSetLanguage(NULL);
  ZeldaReset(false);
  g_initialized = 1;
  return 1;
}

void z3c_reset(int preserve_sram) {
  if (g_initialized)
    ZeldaReset(preserve_sram != 0);
}

void z3c_step_raw(uint16_t input, uint8_t run_what) {
  if (g_initialized)
    ZeldaRunFrameInternal(input, run_what);
}

int z3c_load_reference_replay(int chapter) {
  return g_initialized && ZeldaLoadReferenceReplayForTest(chapter);
}

int z3c_step_replay(void) {
  return g_initialized && ZeldaRunFrame(0);
}

void z3c_configure_features(uint32_t features, uint8_t extra_left_right) {
  if (!g_initialized)
    return;
  g_wanted_zelda_features = features;
  g_zenv.ppu->extraLeftRight = extra_left_right;
}

size_t z3c_core_state_size(void) {
  return Z3C_CORE_STATE_SIZE;
}

int z3c_export_core_state(uint8_t *destination, size_t capacity) {
  if (!g_initialized || destination == NULL || capacity < Z3C_CORE_STATE_SIZE)
    return 0;
  memcpy(destination, g_zenv.ram, Z3C_WRAM_SIZE);
  destination += Z3C_WRAM_SIZE;
  memcpy(destination, g_zenv.sram, Z3C_SRAM_SIZE);
  destination += Z3C_SRAM_SIZE;
  for (size_t i = 0; i < Z3C_VRAM_SIZE / 2; i++) {
    uint16 value = g_zenv.vram[i];
    destination[i * 2 + 0] = (uint8)value;
    destination[i * 2 + 1] = (uint8)(value >> 8);
  }
  return 1;
}

int z3c_import_core_state(const uint8_t *source, size_t length) {
  if (!g_initialized || source == NULL || length != Z3C_CORE_STATE_SIZE)
    return 0;
  memcpy(g_zenv.ram, source, Z3C_WRAM_SIZE);
  source += Z3C_WRAM_SIZE;
  memcpy(g_zenv.sram, source, Z3C_SRAM_SIZE);
  source += Z3C_SRAM_SIZE;
  for (size_t i = 0; i < Z3C_VRAM_SIZE / 2; i++)
    g_zenv.vram[i] = (uint16)source[i * 2] | (uint16)source[i * 2 + 1] << 8;
  return 1;
}

int z3c_import_snapshot(const uint8_t *source, size_t length) {
  return g_initialized && source != NULL && ZeldaLoadSnapshotForTest(source, length);
}

void z3c_render(uint8_t *pixels, size_t pitch, uint32_t render_flags) {
  if (g_initialized && pixels != NULL)
    ZeldaDrawPpuFrame(pixels, pitch, render_flags);
}

void z3c_export_cgram(uint8_t *destination) {
  for (int i = 0; i < 256; i++) {
    destination[i * 2] = (uint8_t)g_zenv.ppu->cgram[i];
    destination[i * 2 + 1] = (uint8_t)(g_zenv.ppu->cgram[i] >> 8);
  }
}

void z3c_render_audio(int16_t *samples) {
  if (g_initialized && samples != NULL)
    ZeldaRenderAudio(samples, 534, 2);
}

void z3c_export_audio_state(uint8_t *destination) {
  if (!g_initialized || destination == NULL)
    return;
  _Static_assert(sizeof(Dsp) - offsetof(Dsp, ram) == 3024,
                 "C DSP snapshot layout changed");
  SpcPlayer_CopyVariablesToRam(g_zenv.player);
  memcpy(destination, g_zenv.player->ram, 65536);
  memcpy(destination + 65536, g_zenv.player->dsp->ram, 3024);
}

void z3c_export_audio_queue(uint8_t *destination) {
  if (g_initialized && destination != NULL)
    ZeldaExportApuQueueForTest(destination);
}
