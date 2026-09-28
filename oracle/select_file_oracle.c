#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "oracle/c_oracle.h"
#include "src/types.h"
#include "src/select_file.h"

static uint8_t *read_file(const char *path, size_t *length) {
  FILE *file = fopen(path, "rb");
  if (file == NULL || fseek(file, 0, SEEK_END) != 0)
    return NULL;
  long size = ftell(file);
  if (size < 0 || fseek(file, 0, SEEK_SET) != 0) {
    fclose(file);
    return NULL;
  }
  uint8_t *bytes = malloc((size_t)size);
  if (bytes == NULL || fread(bytes, 1, (size_t)size, file) != (size_t)size) {
    free(bytes);
    fclose(file);
    return NULL;
  }
  fclose(file);
  *length = (size_t)size;
  return bytes;
}

static void fill_save(uint8_t *save, uint8_t factor, uint8_t add) {
  for (int i = 0; i < 0x500; i++)
    save[i] = (uint8_t)(i * factor + add);
  save[0x4fe] = save[0x4ff] = 0;
}

int main(int argc, char **argv) {
  if (argc != 2) {
    fprintf(stderr, "usage: %s zelda3_assets.dat\n", argv[0]);
    return 2;
  }
  size_t assets_size;
  uint8_t *assets = read_file(argv[1], &assets_size);
  if (assets == NULL || !z3c_initialize(assets, assets_size)) {
    free(assets);
    return 1;
  }
  free(assets);

  const size_t state_size = z3c_core_state_size();
  uint8_t *state = malloc(state_size);
  if (state == NULL)
    return 1;
  memset(state, 0xa5, state_size);
  uint8_t *sram = state + Z3C_WRAM_SIZE;

  fill_save(sram + 0x000, 7, 3);
  fill_save(sram + 0xf00, 11, 5);
  Intro_FixCksum(sram + 0xf00);

  fill_save(sram + 0x500, 13, 9);
  Intro_FixCksum(sram + 0x500);
  fill_save(sram + 0x1400, 17, 1);

  fill_save(sram + 0xa00, 19, 7);
  fill_save(sram + 0x1900, 23, 11);

  if (fwrite(state, state_size, 1, stdout) != 1 ||
      !z3c_import_core_state(state, state_size))
    return 1;
  Intro_ValidateSram();
  if (!z3c_export_core_state(state, state_size) ||
      fwrite(state, state_size, 1, stdout) != 1)
    return 1;
  free(state);
  return fflush(stdout) == 0 ? 0 : 1;
}
