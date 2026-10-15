// Emits both dungeon VRAM-address conversions for every uint16 position, then
// accepts requests: position LE16, four tile LE16 values, attribute byte, and a
// complete initial WRAM image. Replies contain the complete resulting WRAM.
#include <stdint.h>
#include <stdio.h>

#include "src/types.h"
#include "src/dungeon.h"
#include "src/variables.h"

static uint16_t read16(const uint8_t *p) {
  return (uint16_t)p[0] | (uint16_t)p[1] << 8;
}

static int write16(uint16_t value) {
  uint8_t bytes[2] = {(uint8_t)value, (uint8_t)(value >> 8)};
  return fwrite(bytes, 1, 2, stdout) == 2;
}

int main(void) {
  for (uint32_t position = 0; position <= UINT16_MAX; position++) {
    if (!write16(Dungeon_MapVramAddrNoSwap((uint16_t)position)) ||
        !write16(Dungeon_MapVramAddr((uint16_t)position)))
      return 1;
  }
  if (fflush(stdout) != 0)
    return 1;
  for (;;) {
    uint8_t request[11];
    size_t count = fread(request, 1, sizeof(request), stdin);
    if (count == 0 && feof(stdin))
      return 0;
    if (count != sizeof(request) || fread(g_ram, 1, 0x20000, stdin) != 0x20000)
      return 1;
    // Keep this harness to in-image attribute/tile accesses and aligned
    // upload offsets, as used by the game's ordinary update path.
    uint16_t position = read16(request);
    uint16_t upload = read16(g_ram + 0x1000);
    if (position > 0xdfbe || (upload & 1) != 0)
      return 1;
    Dungeon_Store2x2(position, read16(request + 2), read16(request + 4),
                     read16(request + 6), read16(request + 8), request[10]);
    if (fwrite(g_ram, 1, 0x20000, stdout) != 0x20000 || fflush(stdout) != 0)
      return 1;
  }
}
