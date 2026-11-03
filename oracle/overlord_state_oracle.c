// Actual original entrypoints with initialized assets and real native hooks.
// Request: operation,raw-slot,LE16-value,source[3],reserved, complete WRAM.
// Reply: LE32 return, complete WRAM. No state filtering or stubbed hooks.
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include "oracle/c_oracle.h"
#include "src/overlord.h"
#include "src/sprite.h"
#include "src/sprite_main.h"
#include "src/util.h"
#include "src/variables.h"
uint16 Overlord_GetX(int);
uint16 Overlord_GetY(int);

int main(int argc, char **argv) {
  if (argc != 2) return 2;
  size_t size;
  uint8 *assets = ReadWholeFile(argv[1], &size);
  int initialized = assets && z3c_initialize(assets, size);
  free(assets);
  if (!initialized) return 1;
  for (;;) {
    uint8 h[8];
    size_t count = fread(h, 1, sizeof(h), stdin);
    if (!count && feof(stdin)) return 0;
    if (count != sizeof(h) || h[0] > 22) return 1;
    z3c_reset(false);
    if (fread(g_ram, 1, 0x20000, stdin) != 0x20000) return 1;
    int k = h[1];
    uint16 value = h[2] | h[3] << 8;
    uint32_t result = 0;
    switch (h[0]) {
      case 0: Overlord_SetX(k, value); result = Overlord_GetX(k); break;
      case 1: Overlord_SetY(k, value); result = Overlord_GetY(k); break;
      case 2: result = Overlord_GetX(k) | (uint32_t)Overlord_GetY(k) << 16; break;
      case 3: Overlord_CheckIfActive(k); break;
      case 4: Overlord_ExecuteSingle(k); break;
      case 5: Overlord_ExecuteAll(); break;
      case 6: Overlord_Main(); break;
      case 7: result = Sprite_CheckIfOverlordsClear(); break;
      case 8: Sprite_InitializeSlots(); break;
      case 9: Sprite_DisableAll(); break;
      case 10: Dungeon_LoadSingleOverlord(h + 4); break;
      case 11: ArmosCoordinator_Rotate(k); break;
      case 12: ArmosCoordinator_RotateKnights(k); break;
      case 13: result = ArmosCoordinator_CheckKnights(); break;
      case 14: ArmosCoordinator_DisableCoercion(k); break;
      case 15: Overlord19_ArmosCoordinator_bounce(k); break;
      case 16: result = TileRoom_SpawnTile(k); break;
      case 17: SpawnFallingTile(k); break;
      case 18: Overlord_SpawnCannonBall(k, (int16)value); break;
      case 19: Overlord_SpawnBoulder(); break;
      case 20: Ganon_HandleFireBatCircle(k); break;
      case 21: Arrghus_HandlePuffs(k); break;
      case 22: SpritePrep_Arrghi(k); break;
    }
    uint32_t raw = (uint32_t)result;
    uint8 reply[4] = {raw, raw >> 8, raw >> 16, raw >> 24};
    if (fwrite(reply, 1, 4, stdout) != 4 ||
        fwrite(g_ram, 1, 0x20000, stdout) != 0x20000 || fflush(stdout)) return 1;
  }
}
