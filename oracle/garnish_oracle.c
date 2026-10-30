// Asset-free original-C Garnish entrypoints. Request: op, raw slot, LE16 value,
// LE32 signed limit, then complete initial WRAM. Reply: LE32 return + all WRAM.
// Lifecycle fixtures are deliberately bounded by the Rust differential driver;
// never send NULL-handler kind13 or invalid per-handler table indices.
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "src/types.h"
#include "src/sprite.h"
#include "src/sprite_main.h"
#include "src/variables.h"

int main(void) {
  for (;;) {
    uint8_t h[8];
    size_t n = fread(h, 1, sizeof(h), stdin);
    if (!n && feof(stdin)) return 0;
    if (n != sizeof(h) || h[0] > 11 ||
        fread(g_ram, 1, 0x20000, stdin) != 0x20000) return 1;
    uint32_t bits = (uint32_t)h[4] | (uint32_t)h[5] << 8 |
                    (uint32_t)h[6] << 16 | (uint32_t)h[7] << 24;
    int32_t limit;
    memcpy(&limit, &bits, sizeof(limit));
    // The original C condition dereferences before checking k>=0. Bound the
    // complete physical lookup domain, including defined negative addresses.
    if (h[0] == 3 && (limit < -0x1f800 || limit > 0x7ff)) return 1;
    uint16 value = (uint16)h[2] | (uint16)h[3] << 8;
    int result = 0;
    switch (h[0]) {
      case 0: result = GarnishAllocForce(); break;
      case 1: result = GarnishAlloc(); break;
      case 2: result = GarnishAllocLow(); break;
      case 3: result = GarnishAllocLimit(limit); break;
      case 4: result = GarnishAllocOverwriteOldLow(); break;
      case 5: result = GarnishAllocOverwriteOld(); break;
      case 6: Garnish_SetX(h[1], value); break;
      case 7: Garnish_SetY(h[1], value); break;
      case 8: Garnish_ExecuteSingle(h[1]); break;
      case 9: Garnish_ExecuteUpperSlots(); break;
      case 10: Garnish_ExecuteLowerSlots(); break;
      case 11: Garnish13_PyramidDebris(h[1]); break;
    }
    uint32_t returned = (uint32_t)result;
    uint8_t reply[4] = {(uint8_t)returned, (uint8_t)(returned >> 8),
                        (uint8_t)(returned >> 16), (uint8_t)(returned >> 24)};
    if (fwrite(reply, 1, sizeof(reply), stdout) != sizeof(reply) ||
        fwrite(g_ram, 1, 0x20000, stdout) != 0x20000 || fflush(stdout)) return 1;
  }
}
