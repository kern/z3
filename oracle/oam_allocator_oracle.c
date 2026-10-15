// Requests: operation, selector, byte count, complete initial WRAM image.
// Replies: legacy return byte, complete final WRAM. No OAM entries are
// dereferenced, so raw pointer wrap/unaligned allocation vectors are safe.
#include <stdint.h>
#include <stdio.h>

#include "src/types.h"
#include "src/sprite.h"
#include "src/variables.h"

int main(void) {
  uint8 (*const wrappers[])(uint8) = {
    Oam_AllocateFromRegionA, Oam_AllocateFromRegionB, Oam_AllocateFromRegionC,
    Oam_AllocateFromRegionD, Oam_AllocateFromRegionE, Oam_AllocateFromRegionF,
  };
  for (;;) {
    uint8_t request[3];
    size_t count = fread(request, 1, sizeof(request), stdin);
    if (count == 0 && feof(stdin))
      return 0;
    if (count != sizeof(request) || request[0] > 2 ||
        (request[0] == 0 && request[1] >= 12) ||
        (request[0] == 1 && request[1] >= 6) ||
        fread(g_ram, 1, 0x20000, stdin) != 0x20000)
      return 1;
    uint8 result = 0;
    if (request[0] == 0)
      result = Oam_GetBufferPosition(request[2], request[1]);
    else if (request[0] == 1)
      result = wrappers[request[1]](request[2]);
    else
      Oam_ResetRegionBases();
    if (putchar(result) == EOF || fwrite(g_ram, 1, 0x20000, stdout) != 0x20000 || fflush(stdout) != 0)
      return 1;
  }
}
