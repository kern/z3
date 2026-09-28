#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "src/ancilla.h"
#include "src/overworld.h"
#include "src/tile_detect.h"
#include "src/variables.h"

uint8 g_ram[0x20000];

static uint8 map8_attr[512];
static uint16 map16_to_map8[4096];

const uint8 *GetMap8toTileAttr(void) { return map8_attr; }
const uint16 *GetMap16toMap8Table(void) { return map16_to_map8; }
uint16 Ancilla_GetX(int k) { return (uint16)(0x120 + k * 17); }
uint16 Ancilla_GetY(int k) { return (uint16)(0x240 + k * 19); }

static uint64_t hash_ram(void) {
  uint64_t hash = UINT64_C(0xcbf29ce484222325);
  for (size_t i = 0; i < sizeof(g_ram); i++) {
    hash ^= g_ram[i];
    hash *= UINT64_C(0x100000001b3);
  }
  return hash;
}

static void write64(uint64_t value) {
  for (int i = 0; i < 8; i++) {
    if (putchar((uint8_t)(value >> (i * 8))) == EOF)
      _Exit(1);
  }
}

int main(void) {
  for (int tile = 0; tile < 256; tile++) {
    for (int scenario = 0; scenario < 4; scenario++) {
      memset(g_ram, 0, sizeof(g_ram));
      flag_block_link_menu = scenario & 1;
      dung_savegame_state_bits = (scenario & 2) ? 0x8000 : 0;
      player_on_somaria_platform = (scenario & 1) != 0;
      room_transitioning_flags = (scenario & 2) != 0;
      TileDetect_ExecuteInner((uint8)tile, 0x100 + tile * 2,
                              (uint16)(1 << scenario), scenario & 1);
      write64(hash_ram());
    }
  }
  return fflush(stdout) == 0 ? 0 : 1;
}
