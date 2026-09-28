#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "src/ancilla.h"
#include "src/misc.h"
#include "src/sprite.h"
#include "src/sprite_main.h"
#include "src/tagalong.h"
#include "src/variables.h"

uint8 g_ram[0x20000];

int AncillaAdd_SuperBombExplosion(uint8 a, uint8 y) { (void)a; (void)y; return -1; }
uint16 Ancilla_GetX(int k) { (void)k; return 0; }
uint16 Ancilla_GetY(int k) { (void)k; return 0; }
void Ancilla_MoveX(int k) { (void)k; }
void Ancilla_MoveY(int k) { (void)k; }
ProjectSpeedRet Ancilla_ProjectSpeedTowardsPlayer(int k, uint8 vel) {
  (void)k; (void)vel; return (ProjectSpeedRet){0, 0};
}
void Main_ShowTextMessage(void) {}
void OldMan_RevertToSprite(int k) { (void)k; }
void SpritePrep_LoadProperties(int k) { (void)k; }
uint16 Sprite_GetX(int k) { (void)k; return 0; }
uint16 Sprite_GetY(int k) { (void)k; return 0; }
void Sprite_SetX(int k, uint16 x) { (void)k; (void)x; }
void Sprite_SetY(int k, uint16 y) { (void)k; (void)y; }
int Sprite_SpawnDynamically(int k, uint8 what, SpriteSpawnInfo *info) {
  (void)k; (void)what; (void)info; return -1;
}

static void setup(int scenario) {
  memset(g_ram, 0, sizeof(g_ram));
  switch (scenario) {
  case 0:
    main_module_index = 9;
    submodule_index = 0x23;
    break;
  case 1:
    link_player_handler_state = 0;
    break;
  case 2:
    tagalong_var2 = 3;
    tagalong_x_lo[3] = 0x78;
    tagalong_x_hi[3] = 0x19;
    tagalong_y_lo[3] = 0x5c;
    tagalong_y_hi[3] = 0x15;
    tagalong_z[3] = 0;
    break;
  case 3:
    timer_tagalong_reacquire = 0;
    saved_tagalong_x = link_x_coord = 0x100;
    saved_tagalong_y = link_y_coord = 0x180;
    break;
  case 4:
    timer_tagalong_reacquire = 1;
    break;
  case 5:
    follower_indicator = 9;
    break;
  }
}

static void run(int scenario) {
  switch (scenario) {
  case 0: g_ram[0x1ff0] = Tagalong_IsFollowing(); break;
  case 1: g_ram[0x1ff0] = Follower_ValidateMessageFreedom(); break;
  case 2: g_ram[0x1ff0] = Follower_CheckBlindTrigger(); break;
  case 3:
  case 4: g_ram[0x1ff0] = Follower_CheckProximityToLink(); break;
  case 5: Follower_Disable(); break;
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
