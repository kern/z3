#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "oracle/c_oracle.h"
#include "src/misc.h"

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
    free(bytes); fclose(file); return NULL;
  }
  fclose(file);
  *length = (size_t)size;
  return bytes;
}

static uint64_t hash_state(const uint8_t *bytes, size_t size) {
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

int main(int argc, char **argv) {
  if (argc != 2)
    return 2;
  size_t assets_size;
  uint8_t *assets = read_file(argv[1], &assets_size);
  if (assets == NULL || !z3c_initialize(assets, assets_size)) {
    free(assets); return 1;
  }
  free(assets);
  const size_t state_size = z3c_core_state_size();
  uint8_t *state = calloc(state_size, 1);
  if (state == NULL)
    return 1;
  for (int scenario = 0; scenario < 256; scenario++) {
    memset(state, 0, state_size);
    state[0x1a] = (uint8_t)(scenario * 29 + 7);
    state[0xfa1] = (uint8_t)(scenario * 53 + 11);
    state[0xe2] = (uint8_t)(scenario * 17);
    state[0xe3] = (uint8_t)(scenario >> 3);
    if (!z3c_import_core_state(state, state_size))
      return 1;
    uint8_t random = GetRandomNumber();
    uint8_t pan = CalculateSfxPan_Arbitrary((uint8_t)scenario);
    if (!z3c_export_core_state(state, state_size))
      return 1;
    state[0x1ff0] = random;
    state[0x1ff1] = pan;
    write64(hash_state(state, state_size));
  }
  free(state);
  return fflush(stdout) == 0 ? 0 : 1;
}
