#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "src/misc.h"
#include "src/overlord.h"
#include "src/sprite.h"
#include "src/sprite_main.h"
#include "src/variables.h"

uint8 g_ram[0x20000];
const uint16 kSinusLookupTable[256] = {0};

uint16 Overlord_GetX(int k);
uint16 Overlord_GetY(int k);

uint8 CalculateSfxPan_Arbitrary(uint8 a) { return a; }
int GarnishAlloc(void) { return -1; }
uint8 GetRandomNumber(void) { return 0; }
uint8 GetTileAttribute(uint8 floor, uint16 *x, uint16 y) {
  (void)floor; (void)x; (void)y; return 0;
}
void SpriteSfx_QueueSfx2WithPan(int k, uint8 a) { (void)k; (void)a; }
void SpriteSfx_QueueSfx3WithPan(int k, uint8 a) { (void)k; (void)a; }
void Sprite_SetX(int k, uint16 x) { (void)k; (void)x; }
void Sprite_SetY(int k, uint16 y) { (void)k; (void)y; }
int Sprite_SpawnDynamically(int k, uint8 what, SpriteSpawnInfo *info) {
  (void)k; (void)what; (void)info; return -1;
}
int Sprite_SpawnDynamicallyEx(int k, uint8 what, SpriteSpawnInfo *info, int j) {
  (void)k; (void)what; (void)info; (void)j; return -1;
}
void Sprite_TransmuteToBomb(int k) { (void)k; }

static void setup(int scenario) {
  memset(g_ram, 0, sizeof(g_ram));
  switch (scenario) {
  case 1:
    link_x_coord = 0x200;
    link_y_coord = 0x180;
    Overlord_SetX(2, 0x210);
    Overlord_SetY(2, 0x170);
    overlord_type[2] = 0x17;
    break;
  case 3:
    frame_counter = 0;
    BG2HOFS_copy2 = 0x100;
    BG2VOFS_copy2 = 0x100;
    Overlord_SetX(1, 0x500);
    Overlord_SetY(1, 0x500);
    overlord_type[1] = 7;
    overlord_offset_sprite_pos[1] = 9;
    overworld_sprite_was_loaded[1] = 0xff;
    break;
  case 4:
    sprite_state[0] = 1;
    sprite_ai_state[0] = 0;
    sprite_state[1] = 1;
    sprite_ai_state[1] = 2;
    break;
  case 5:
    for (int i = 0; i < 6; i++)
      sprite_ai_state[i] = (uint8)(i + 1);
    break;
  }
}

static void run(int scenario) {
  switch (scenario) {
  case 0:
    Overlord_SetX(3, 0x1234);
    Overlord_SetY(3, 0xabcd);
    WORD(g_ram[0x1ff0]) = Overlord_GetX(3);
    WORD(g_ram[0x1ff2]) = Overlord_GetY(3);
    break;
  case 1: Overlord17_PotTrap(2); break;
  case 2: Overlord01_PositionTarget(3); break;
  case 3: Overlord_CheckIfActive(1); break;
  case 4: g_ram[0x1ff0] = ArmosCoordinator_CheckKnights(); break;
  case 5: ArmosCoordinator_DisableCoercion(0); break;
  }
}

int main(void) {
  for (int scenario = 0; scenario < 6; scenario++) {
    setup(scenario);
    if (fwrite(g_ram, sizeof(g_ram), 1, stdout) != 1)
      return 1;
    run(scenario);
    if (fwrite(g_ram, sizeof(g_ram), 1, stdout) != 1)
      return 1;
  }
  return fflush(stdout) == 0 ? 0 : 1;
}
