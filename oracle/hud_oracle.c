#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "oracle/c_oracle.h"
#include "src/hud.h"

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

int main(int argc, char **argv) {
  if (argc != 2)
    return 2;
  size_t assets_size;
  uint8_t *assets = read_file(argv[1], &assets_size);
  if (assets == NULL || !z3c_initialize(assets, assets_size)) {
    free(assets);
    return 1;
  }
  free(assets);

  const size_t state_size = z3c_core_state_size();
  uint8_t *state = calloc(state_size, 1);
  if (state == NULL)
    return 1;
  state[0xf343] = 9;
  state[0xf360] = 0xd2;
  state[0xf361] = 0x04;
  state[0xf362] = 0xd2;
  state[0xf363] = 0x04;
  state[0xf36c] = 0x50;
  state[0xf36d] = 0x34;
  state[0xf36e] = 0x60;
  state[0xf36f] = 3;
  state[0xf370] = 1;
  state[0xf371] = 2;
  state[0xf377] = 27;
  state[0xf37b] = 1;

  if (fwrite(state, state_size, 1, stdout) != 1 ||
      !z3c_import_core_state(state, state_size))
    return 1;
  Hud_Rebuild();
  if (!z3c_export_core_state(state, state_size) ||
      fwrite(state, state_size, 1, stdout) != 1)
    return 1;
  free(state);
  return fflush(stdout) == 0 ? 0 : 1;
}
