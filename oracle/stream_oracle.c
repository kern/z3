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
  // command and emits that state again. 0xff is not a ZeldaRunFrameInternal
  // mode, so ordinary three-byte frame commands remain backwards compatible.
  uint8_t command[3];
  while (fread(command, sizeof(command), 1, stdin) == 1) {
    if (command[2] == 0xff) {
      if (fread(state, state_size, 1, stdin) != 1 ||
          !z3c_import_core_state(state, state_size)) {
        fprintf(stderr, "oracle stream state import failed\n");
        free(state);
        return 1;
      }
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
