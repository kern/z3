#include "oracle/c_oracle.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static uint64_t fnv1a64(const uint8_t *data, size_t length) {
  uint64_t hash = UINT64_C(14695981039346656037);
  for (size_t i = 0; i < length; i++) {
    hash ^= data[i];
    hash *= UINT64_C(1099511628211);
  }
  return hash;
}

static uint8_t *read_file(const char *path, size_t *length) {
  FILE *file = fopen(path, "rb");
  if (file == NULL || fseek(file, 0, SEEK_END) != 0)
    return NULL;
  long size = ftell(file);
  if (size < 0 || fseek(file, 0, SEEK_SET) != 0) {
    fclose(file);
    return NULL;
  }
  uint8_t *data = malloc((size_t)size);
  if (data == NULL || fread(data, 1, (size_t)size, file) != (size_t)size) {
    free(data);
    fclose(file);
    return NULL;
  }
  fclose(file);
  *length = (size_t)size;
  return data;
}

static void print_state_hash(uint32_t frame, uint8_t *state, size_t size) {
  if (!z3c_export_core_state(state, size)) {
    fprintf(stderr, "failed to export oracle state\n");
    exit(1);
  }
  printf("frame=%u fnv1a64=%016llx\n", frame,
         (unsigned long long)fnv1a64(state, size));
}

int main(int argc, char **argv) {
  if (argc != 2) {
    fprintf(stderr, "usage: %s zelda3_assets.dat\n", argv[0]);
    return 2;
  }

  size_t assets_size;
  uint8_t *assets = read_file(argv[1], &assets_size);
  if (assets == NULL || !z3c_initialize(assets, assets_size)) {
    fprintf(stderr, "failed to initialize oracle\n");
    free(assets);
    return 1;
  }
  free(assets);

  size_t state_size = z3c_core_state_size();
  uint8_t *state = malloc(state_size);
  if (state == NULL)
    return 1;

  print_state_hash(0, state, state_size);
  for (uint32_t frame = 1; frame <= 600; frame++) {
    // A deterministic input pulse exercises the input edge path while keeping
    // this smoke test independent of host keyboard/controller state.
    uint16_t input = frame >= 180 && frame < 184 ? 0x1000 : 0;
    z3c_step_raw(input, 1);
    if (frame == 1 || frame == 60 || frame == 180 || frame == 600)
      print_state_hash(frame, state, state_size);
  }

  const size_t pitch = 256 * sizeof(uint32_t);
  const size_t pixels_size = pitch * 224;
  uint8_t *pixels = calloc(pixels_size, 1);
  if (pixels == NULL)
    return 1;
  z3c_render(pixels, pitch, 0);
  printf("frame=600 pixels-fnv1a64=%016llx\n",
         (unsigned long long)fnv1a64(pixels, pixels_size));
  free(pixels);

  free(state);
  return 0;
}
