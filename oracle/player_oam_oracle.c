#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "src/player_oam.h"
#include "src/variables.h"

uint8 g_ram[0x20000];
const uint8 kSwimmingTab1[4] = {2, 0, 1, 0};

static void put8(uint8 value) {
  if (putchar(value) == EOF)
    _Exit(1);
}

int main(void) {
  for (int value = 0; value < 256; value++)
    put8(FindMostSignificantBit(value));

  for (int sword = 0; sword < 5; sword++) {
    for (int direction = 0; direction < 8; direction += 2) {
      for (int frame = 0; frame < 21; frame++) {
        memset(g_ram, 0, sizeof(g_ram));
        link_sword_type = sword;
        link_direction_facing = direction;
        button_b_frames = frame;
        player_oam_x_offset = 0xa5;
        player_oam_y_offset = 0x5a;
        CalculateSwordHitBox();
        put8(player_oam_x_offset);
        put8(player_oam_y_offset);
      }
    }
  }

  for (int item = 0; item <= 5; item += 5) {
    for (int index = 0; index < 511; index++) {
      SwordResult sword = {0xa5, 0x5a};
      memset(g_ram, 0, sizeof(g_ram));
      link_item_in_hand = item;
      link_dma_var3 = 0x31;
      link_dma_var4 = 0x42;
      link_dma_var5 = 0x53;
      if (item == 0) {
        put8(LinkOam_SetWeaponVRAMOffsets(index, &sword));
        put8(sword.r6);
        put8(sword.r12);
        put8(link_dma_var3);
        put8(link_dma_var4);
        put8(link_dma_var5);
      }

      sword.r6 = 0xa5;
      sword.r12 = 0x5a;
      link_dma_var3 = 0x31;
      link_dma_var4 = 0x42;
      link_dma_var5 = 0x53;
      put8(LinkOam_SetEquipmentVRAMOffsets(index, &sword));
      put8(sword.r6);
      put8(sword.r12);
      put8(link_dma_var3);
      put8(link_dma_var4);
      put8(link_dma_var5);
    }
  }

  for (int scenario = 0; scenario < 1024; scenario++) {
    memset(g_ram, 0, sizeof(g_ram));
    link_player_handler_state = scenario & 7;
    link_item_in_hand = (scenario >> 1) & 0xff;
    link_position_mode = (scenario >> 3) & 0x3f;
    button_mask_b_y = (scenario & 1) ? 0x80 : 0;
    link_sword_type = (scenario >> 4) & 3;
    link_state_bits = (scenario >> 7) & 1;
    link_force_hold_sword_up = (scenario >> 8) & 1;
    link_electrocute_on_touch = (scenario >> 9) & 1;
    put8(PlayerOam_WantInvokeSword());
  }
  return fflush(stdout) == 0 ? 0 : 1;
}
