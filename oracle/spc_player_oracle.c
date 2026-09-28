#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "src/types.h"
#include "src/spc_player.h"

enum {
  kAssetCount = 165,
  kRamCompareSize = 0xc000,
  kSampleCount = 534 * 2,
  kResponseSize = kRamCompareSize + kSampleCount * 2 + 4 + 256 + 256,
};

static uint32_t read_u32(const uint8_t *p) {
  return (uint32_t)p[0] | (uint32_t)p[1] << 8 | (uint32_t)p[2] << 16 |
         (uint32_t)p[3] << 24;
}

static uint8_t *read_file(const char *path, size_t *size) {
  FILE *f = fopen(path, "rb");
  if (f == NULL || fseek(f, 0, SEEK_END) != 0)
    return NULL;
  long n = ftell(f);
  if (n < 0 || fseek(f, 0, SEEK_SET) != 0) {
    fclose(f);
    return NULL;
  }
  uint8_t *data = malloc((size_t)n);
  if (data == NULL || fread(data, (size_t)n, 1, f) != 1) {
    free(data);
    fclose(f);
    return NULL;
  }
  fclose(f);
  *size = (size_t)n;
  return data;
}

static const uint8_t *asset_zero(const uint8_t *data, size_t size) {
  size_t offset = 88 + kAssetCount * 4 + read_u32(data + 84);
  offset = (offset + 3) & ~(size_t)3;
  return offset + read_u32(data + 88) <= size ? data + offset : NULL;
}

static void set_frame_command(SpcPlayer *player, int frame) {
  if (frame == 1)
    player->input_ports[0] = 1;
  else if (frame == 3)
    player->input_ports[1] = 1;
  else if (frame == 5)
    player->input_ports[2] = 1;
  else if (frame == 7)
    player->input_ports[3] = 1;
}

int main(int argc, char **argv) {
  if (argc != 2)
    return 2;
  size_t assets_size;
  uint8_t *assets = read_file(argv[1], &assets_size);
  const uint8_t *bank = assets == NULL ? NULL : asset_zero(assets, assets_size);
  if (bank == NULL)
    return 1;

  SpcPlayer *player = SpcPlayer_Create();
  DspRegWriteHistory history = {0};
  player->reg_write_history = &history;
  SpcPlayer_Upload(player, bank);
  SpcPlayer_Initialize(player);
  history.count = 0;

  uint8_t *response = malloc(kResponseSize);
  int16_t samples[kSampleCount];
  if (response == NULL)
    return 1;
  for (int frame = 0; frame < 10; frame++) {
    set_frame_command(player, frame);
    SpcPlayer_GenerateSamples(player);
    dsp_getSamples(player->dsp, samples, 534, 2);
    SpcPlayer_CopyVariablesToRam(player);

    uint8_t *out = response;
    memcpy(out, player->ram, kRamCompareSize);
    out += kRamCompareSize;
    for (int i = 0; i < kSampleCount; i++) {
      uint16_t sample = (uint16_t)samples[i];
      *out++ = (uint8_t)sample;
      *out++ = (uint8_t)(sample >> 8);
    }
    uint32_t count = history.count;
    *out++ = (uint8_t)count;
    *out++ = (uint8_t)(count >> 8);
    *out++ = (uint8_t)(count >> 16);
    *out++ = (uint8_t)(count >> 24);
    memcpy(out, history.addr, 256);
    out += 256;
    memcpy(out, history.val, 256);
    if (fwrite(response, kResponseSize, 1, stdout) != 1)
      return 1;
    history.count = 0;
  }
  free(response);
  free(assets);
  return 0;
}
