// Actual unchanged receipt/bottle entrypoints with real compiled assets.
// Requests: operation,item,complete WRAM; responses: complete WRAM.
// Operation0 runs AncillaAdd_ItemReceipt(0x22,4,0), operation1 bottles only.
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include "oracle/c_oracle.h"
#include "src/misc.h"
#include "src/util.h"
#include "src/variables.h"

int main(int argc, char **argv) {
  if (argc != 2) return 2;
  size_t size;
  uint8 *assets = ReadWholeFile(argv[1], &size);
  int initialized = assets && z3c_initialize(assets, size);
  free(assets);
  if (!initialized) return 1;
  for (;;) {
    uint8 request[2];
    size_t count = fread(request, 1, 2, stdin);
    if (!count && feof(stdin)) return 0;
    if (count != 2 || request[0] > 1) return 1;
    z3c_reset(false);
    if (fread(g_ram, 1, 0x20000, stdin) != 0x20000) return 1;
    if (request[0] == 0) {
      if (request[1] >= 76 || link_receiveitem_index != request[1]) return 2;
      // The original promoted-int shift is defined through count31. Higher
      // palace IDs are excluded here, NOT clamped/masked in the C oracle.
      if ((request[1] == 0x25 || request[1] == 0x32 || request[1] == 0x33) &&
          (uint8)cur_palace_index_x2 >= 64) return 2;
      AncillaAdd_ItemReceipt(0x22, 4, 0);
    } else {
      ItemReceipt_GiveBottledItem(request[1]);
    }
    if (fwrite(g_ram, 1, 0x20000, stdout) != 0x20000 || fflush(stdout)) return 1;
  }
}
