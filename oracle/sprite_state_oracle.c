// Asset-free vectors against existing compiled-C sprite helpers.
#include <stdint.h>
#include <stdio.h>
#include "src/sprite.h"
#include "src/variables.h"

int main(void) {
  for (;;) {
    uint8 request[8];
    size_t count = fread(request, 1, sizeof(request), stdin);
    if (count == 0 && feof(stdin)) return 0;
    if (count != sizeof(request) || request[0] > 9 || request[1] >= 16 ||
        fread(g_ram, 1, 0x20000, stdin) != 0x20000) return 1;
    int k = request[1];
    switch (request[0]) {
    case 0: Sprite_Invert_XY_Speeds(k); break;
    case 1: Sprite_HalveSpeed_XY(k); break;
    case 2: Sprite_BounceOffWall(k); break;
    case 3: Sprite_ApproachTargetSpeed(k, request[2], request[3]); break;
    case 4: SpritePrep_ResetProperties(k); break;
    case 5: SpritePrep_LoadProperties(k); break;
    case 6: Dungeon_CacheTransSprites(); break;
    case 7:
      for (int current = 0; current < 256; current++) {
        for (int target = 0; target < 256; target++) {
          sprite_x_vel[k] = current;
          sprite_y_vel[k] = target;
          Sprite_ApproachTargetSpeed(k, target, current);
          if (putchar(sprite_x_vel[k]) == EOF || putchar(sprite_y_vel[k]) == EOF) return 1;
        }
      }
      break;
    case 8: {
      uint16 x = request[2] | (uint16)request[3] << 8;
      uint16 y = request[4] | (uint16)request[5] << 8;
      ProjectSpeedRet p = Sprite_ProjectSpeedTowardsLocation(k, x, y, request[6]);
      uint8 result[] = {p.x, p.y, p.xdiff, p.ydiff};
      if (fwrite(result, 1, 4, stdout) != 4) return 1;
      break;
    }
    case 9:
      if (putchar(Sprite_ConvertVelocityToAngle(request[2], request[3])) == EOF) return 1;
      break;
    }
    if (fwrite(g_ram, 1, 0x20000, stdout) != 0x20000 || fflush(stdout)) return 1;
  }
}
