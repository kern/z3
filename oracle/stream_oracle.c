#include "oracle/c_oracle.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

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

  // Protocol: stdin contains repeated [input_lo, input_hi, run_what] records.
  // One canonical WRAM/SRAM/VRAM state is emitted after each complete record.
  // run_what == 0xff imports one canonical state immediately following the
  // command and emits that state again. run_what == 0xfe configures the feature
  // mask and extra side-space from five following bytes, then emits state.
  // run_what == 0xfd renders using a following [height_lo, height_hi, flags,
  // scale] record
  // run_what == 0xfc imports a complete legacy device snapshot prefixed by a
  // little-endian uint32 length and emits its canonical state.
  // record and emits the tightly packed ARGB buffer instead of core state.
  // These values are not ZeldaRunFrameInternal modes, so ordinary three-byte
  // frame commands remain backwards compatible.
  uint8_t command[3];
  while (fread(command, sizeof(command), 1, stdin) == 1) {
    if (command[2] == 0xff) {
      if (fread(state, state_size, 1, stdin) != 1 ||
          !z3c_import_core_state(state, state_size)) {
        fprintf(stderr, "oracle stream state import failed\n");
        free(state);
        return 1;
      }
    } else if (command[2] == 0xfc) {
      uint8_t length_bytes[4];
      if (fread(length_bytes, sizeof(length_bytes), 1, stdin) != 1) {
        fprintf(stderr, "oracle stream snapshot length failed\n");
        free(state);
        return 1;
      }
      size_t snapshot_size = (size_t)length_bytes[0] | (size_t)length_bytes[1] << 8 |
                             (size_t)length_bytes[2] << 16 | (size_t)length_bytes[3] << 24;
      uint8_t *snapshot = malloc(snapshot_size);
      if (snapshot == NULL || fread(snapshot, snapshot_size, 1, stdin) != 1 ||
          !z3c_import_snapshot(snapshot, snapshot_size)) {
        fprintf(stderr, "oracle stream snapshot import failed\n");
        free(snapshot);
        free(state);
        return 1;
      }
      free(snapshot);
    } else if (command[2] == 0xfe) {
      uint8_t config[5];
      if (fread(config, sizeof(config), 1, stdin) != 1) {
        fprintf(stderr, "oracle stream feature configuration failed\n");
        free(state);
        return 1;
      }
      uint32_t features = (uint32_t)config[0] | (uint32_t)config[1] << 8 |
                          (uint32_t)config[2] << 16 | (uint32_t)config[3] << 24;
      z3c_configure_features(features, config[4]);
    } else if (command[2] == 0xfd) {
      uint8_t render[4];
      if (fread(render, sizeof(render), 1, stdin) != 1) {
        fprintf(stderr, "oracle stream render configuration failed\n");
        free(state);
        return 1;
      }
      size_t width = (size_t)command[0] | (size_t)command[1] << 8;
      size_t height = (size_t)render[0] | (size_t)render[1] << 8;
      uint32_t flags = render[2];
      size_t scale = render[3];
      if (scale != 1 && scale != 4) {
        fprintf(stderr, "oracle stream invalid render scale\n");
        free(state);
        return 1;
      }
      size_t pixel_bytes = width * height * scale * scale * 4;
      uint8_t *pixels = calloc(pixel_bytes, 1);
      if (pixels == NULL) {
        free(state);
        return 1;
      }
      z3c_render(pixels, width * scale * 4, flags);
      int failed = fwrite(pixels, pixel_bytes, 1, stdout) != 1 || fflush(stdout) != 0;
      free(pixels);
      if (failed) {
        fprintf(stderr, "oracle stream render I/O failed\n");
        free(state);
        return 1;
      }
      continue;
    } else {
      uint16_t input = (uint16_t)command[0] | (uint16_t)command[1] << 8;
      z3c_step_raw(input, command[2]);
    }
    if (!z3c_export_core_state(state, state_size) ||
        fwrite(state, state_size, 1, stdout) != 1 || fflush(stdout) != 0) {
      fprintf(stderr, "oracle stream I/O failed\n");
      free(state);
      return 1;
    }
  }
  if (!feof(stdin)) {
    fprintf(stderr, "oracle stream input failed\n");
    free(state);
    return 1;
  }

  free(state);
  return 0;
}
