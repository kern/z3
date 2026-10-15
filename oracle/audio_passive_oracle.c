// Focused observation-purity regression; no ROM-derived fixture is embedded.
#include "oracle/c_oracle.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "src/types.h"
#include "src/zelda_rtl.h"
#include "src/spc_player.h"

static int require(int condition, const char *message) {
  if (!condition)
    fprintf(stderr, "passive audio regression: %s\n", message);
  return condition;
}

int main(int argc, char **argv) {
  if (argc != 2) {
    fprintf(stderr, "usage: %s zelda3_assets.dat\n", argv[0]);
    return 2;
  }
  FILE *file = fopen(argv[1], "rb");
  if (file == NULL)
    return 1;
  if (fseek(file, 0, SEEK_END) != 0) {
    fclose(file);
    return 1;
  }
  long size = ftell(file);
  if (size <= 0 || fseek(file, 0, SEEK_SET) != 0) {
    fclose(file);
    return 1;
  }
  uint8_t *assets = malloc((size_t)size);
  int loaded = assets != NULL && fread(assets, 1, (size_t)size, file) == (size_t)size;
  fclose(file);
  int initialized = loaded && z3c_initialize(assets, (size_t)size);
  free(assets);
  if (!initialized)
    return 1;

  // Deliberately disagree with the inline player/channel fields. Mapping the
  // live player would visibly erase these bytes, even without rendering PCM.
  memset(g_zenv.player->ram, 0xa5, sizeof(g_zenv.player->ram));
  g_zenv.player->master_volume = 0x1234;
  for (int i = 0; i < 8; i++)
    g_zenv.player->channel[i].note_ticks_left = (uint8_t)(0x30 + i);
  zelda_apu_write(0x2140, 0x19);
  zelda_apu_write(0x2142, 0x27);

  SpcPlayer before;
  Dsp dsp_before;
  memcpy(&before, g_zenv.player, sizeof(before));
  memcpy(&dsp_before, g_zenv.player->dsp, sizeof(dsp_before));
  uint8_t queue_before[75], queue_after[75];
  uint8_t snapshot[65536 + 3024], repeated[65536 + 3024];
  z3c_export_audio_queue(queue_before);
  z3c_export_audio_state_passive(snapshot);
  z3c_export_audio_state_passive(repeated);
  z3c_export_audio_state_passive(NULL);
  z3c_export_audio_queue(queue_after);
  int ok = require(memcmp(&before, g_zenv.player, sizeof(before)) == 0,
                   "live player (including RAM/channels/pointers) changed") &&
      require(memcmp(&dsp_before, g_zenv.player->dsp, sizeof(dsp_before)) == 0,
              "live DSP changed") &&
      require(memcmp(queue_before, queue_after, sizeof(queue_before)) == 0,
              "live queue changed") &&
      require(memcmp(snapshot, repeated, sizeof(snapshot)) == 0,
              "repeated observations disagree") &&
      require(memcmp(snapshot + 65536, dsp_before.ram, 3024) == 0,
              "DSP export differs from the original live block") &&
      require(snapshot[0x58] == 0x34 && snapshot[0x59] == 0x12,
              "inline player fields were not mapped to snapshot RAM");
  for (int i = 0; i < 8; i++)
    ok &= require(snapshot[0x70 + i * 2] == 0x30 + i,
                  "inline channel fields were not mapped to snapshot RAM");
  // The legacy diagnostic must remain a live-RAM mapping operation and emit
  // the same serialized bytes. Passive observation does not replace this gate.
  z3c_export_audio_state(repeated);
  ok &= require(memcmp(snapshot, repeated, sizeof(snapshot)) == 0,
                "passive and legacy serialized snapshots differ");
  ok &= require(memcmp(before.ram, g_zenv.player->ram, sizeof(before.ram)) != 0,
                "legacy diagnostic no longer maps the live RAM");
  if (ok)
    puts("passive audio snapshot purity and legacy mapping passed");
  return ok ? 0 : 1;
}
