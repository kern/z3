#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "src/poly.h"
#include "src/types.h"

uint8 g_ram[0x20000];

int main(void) {
  memset(g_ram, 0xaa, sizeof(g_ram));
  g_ram[0x1f02] = 0;
  g_ram[0x1f03] = 0;
  g_ram[0x1f04] = 0;
  g_ram[0x1f05] = 0;
  g_ram[0x1f06] = 64;
  g_ram[0x1f07] = 64;
  Poly_RunFrame();
  return fwrite(g_ram, sizeof(g_ram), 1, stdout) == 1 ? 0 : 1;
}
