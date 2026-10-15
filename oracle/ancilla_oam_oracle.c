// Requests: operation, local entry index LE16, effect slot, initial WRAM.
// Replies: returned local entry index LE16, complete resulting WRAM.
// Includes one-past OAM cursors but never dereferences the incoming OamEnt.
#include <stdint.h>
#include <stdio.h>

#include "src/types.h"
#include "src/ancilla.h"
#include "src/variables.h"

int main(void) {
  for (;;) {
    uint8_t request[4];
    size_t count = fread(request, 1, sizeof(request), stdin);
    if (count == 0 && feof(stdin))
      return 0;
    if (count != sizeof(request))
      return 1;
    uint16_t index = (uint16_t)request[1] | (uint16_t)request[2] << 8;
    if (request[0] > 3 || index > 128 ||
        (request[0] == 2 && (request[3] >= 5 || index > 126)) ||
        (request[0] == 3 && (request[3] >= 10 || index > 126)) ||
        fread(g_ram, 1, 0x20000, stdin) != 0x20000)
      return 1;
    OamEnt *cursor = (OamEnt *)(g_ram + 0x800 + index * 4);
    uint16_t returned = 0;
    if (request[0] < 2) {
      OamEnt *result = request[0] == 0 ? Ancilla_AllocateOamFromCustomRegion(cursor)
                                     : HitStars_UpdateOamBufferPosition(cursor);
      returned = (uint16_t)(((uint8_t *)result - g_ram - 0x800) / 4);
    } else if (request[0] == 2) {
      AncillaDraw_QuakeInitialBolts(request[3]);
    } else {
      QuakeSpell_SpreadBolts(request[3]);
    }
    uint8_t reply[2] = {(uint8_t)returned, (uint8_t)(returned >> 8)};
    if (fwrite(reply, 1, 2, stdout) != 2 ||
        fwrite(g_ram, 1, 0x20000, stdout) != 0x20000 || fflush(stdout) != 0)
      return 1;
  }
}
