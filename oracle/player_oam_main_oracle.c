#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "src/player.h"
#include "src/player_oam.h"
#include "src/variables.h"

uint8 g_ram[0x20000];
const uint8 kSwimmingTab1[4] = {2, 0, 1, 0};

static void setup(int scenario) {
  memset(g_ram, 0, sizeof(g_ram));
  link_x_coord = 0x80;
  link_y_coord = 0x70;
  link_sword_type = 0xff;
  switch (scenario) {
  case 1:
    link_player_handler_state = kPlayerState_AsleepInBed;
    link_pose_during_opening = 1;
    break;
  case 2:
    link_force_hold_sword_up = 1;
    link_direction_facing = 6;
    link_sword_type = 2;
    break;
  case 3:
    link_is_bunny_mirror = 1;
    link_animation_steps = 3;
    link_direction_facing = 4;
    break;
  case 4:
    link_player_handler_state = kPlayerState_Swimming;
    joypad1H_last = 1;
    byte_7E02CC = 2;
    break;
  case 5:
    link_grabbing_wall = 1;
    link_direction_facing = 6;
    link_animation_steps = 2;
    break;
  case 6:
    link_player_handler_state = kPlayerState_TurtleRock;
    link_auxiliary_state = 1;
    byte_7E034E = 0;
    link_state_bits = 2;
    break;
  case 7:
    link_is_in_deep_water = 1;
    break;
  case 8:
    submodule_index = 14;
    main_module_index = 1;
    link_actual_vel_x = 0;
    break;
  case 9:
    submodule_index = 18;
    which_staircase_index = 4;
    link_animation_steps = 5;
    break;
  }
}

int main(void) {
  for (int scenario = 0; scenario < 10; scenario++) {
    setup(scenario);
    LinkOam_Main();
    if (fwrite(g_ram, sizeof(g_ram), 1, stdout) != 1)
      return 1;
  }
  return fflush(stdout) == 0 ? 0 : 1;
}
