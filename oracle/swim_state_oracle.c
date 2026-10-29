// Whole-WRAM vectors against unchanged exported player.c operations.
// This uses the original target/compiler's unaligned uint16 WRAM aliases;
// it does not claim portable ISO-C alignment guarantees. Stroke requests are
// bounded to table-safe, nonnegative downstream velocity cases (the original
// negative-velocity display calculation left-shifts a negative promoted int).
#include <stdint.h>
#include <stdio.h>
#include "src/player.h"
#include "src/variables.h"

int main(void) {
  void (*const operations[])(void) = {
    ResetAllAcceleration, Link_ResetSwimmingState, Link_FlagMaxAccels,
    Link_SetIceMaxAccel, Link_SetMomentum, Link_SetTheMaxAccel,
    Link_HandleSwimAccels, HandleSwimStrokeAndSubpixels,
  };
  for (;;) {
    int operation = getchar();
    if (operation == EOF) return ferror(stdin) ? 1 : 0;
    if (operation > 7 || fread(g_ram, 1, 0x20000, stdin) != 0x20000) return 1;
    if (operation == 4 && link_flag_moving > 2) return 2;
    if (operation == 7) {
      if (link_flag_moving > 2 || swimcoll_var5[0] > 3 || swimcoll_var5[1] > 3 ||
          swimcoll_var11[0] < 1 || swimcoll_var11[0] > 3 || swimcoll_var11[1] != 1)
        return 2;
    }
    operations[operation]();
    if (fwrite(g_ram, 1, 0x20000, stdout) != 0x20000 || fflush(stdout)) return 1;
  }
}
