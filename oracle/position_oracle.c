// Differential movement oracle. Each request is [operation, slot] followed by
// the complete initial WRAM image; the reply is the complete final WRAM image.
// EOF between requests terminates cleanly. No game assets are needed.
#include <stdint.h>
#include <stdio.h>

#include "src/ancilla.h"
#include "src/sprite.h"
#include "src/variables.h"

int main(void) {
  void (*const operations[])(int) = {
    Ancilla_MoveX, Ancilla_MoveY, Ancilla_MoveZ,
    Sprite_MoveX, Sprite_MoveY, Sprite_MoveZ,
  };
  for (;;) {
    uint8_t request[2];
    size_t count = fread(request, 1, sizeof(request), stdin);
    if (count == 0 && feof(stdin))
      return 0;
    if (count != sizeof(request) || request[0] >= 6 ||
        request[1] >= (request[0] < 3 ? 10 : 16))
      return 1;
    if (fread(g_ram, 1, 0x20000, stdin) != 0x20000)
      return 1;
    operations[request[0]](request[1]);
    if (fwrite(g_ram, 1, 0x20000, stdout) != 0x20000 || fflush(stdout) != 0)
      return 1;
  }
}
