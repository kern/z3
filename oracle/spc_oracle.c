#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "snes/apu.h"
#include "snes/spc.h"

enum { kRequestSize = 8 + 0x10000, kResponseSize = 9 + 0x10000 };

uint8_t apu_cpuRead(Apu *apu, uint16_t address) {
  return apu->ram[address];
}

void apu_cpuWrite(Apu *apu, uint16_t address, uint8_t value) {
  apu->ram[address] = value;
}

static uint8_t get_flags(const Spc *spc) {
  return (uint8_t)spc->c | (uint8_t)spc->z << 1 | (uint8_t)spc->i << 2 |
         (uint8_t)spc->h << 3 | (uint8_t)spc->b << 4 | (uint8_t)spc->p << 5 |
         (uint8_t)spc->v << 6 | (uint8_t)spc->n << 7;
}

static void set_flags(Spc *spc, uint8_t flags) {
  spc->c = flags & 1;
  spc->z = flags & 2;
  spc->i = flags & 4;
  spc->h = flags & 8;
  spc->b = flags & 16;
  spc->p = flags & 32;
  spc->v = flags & 64;
  spc->n = flags & 128;
}

int main(void) {
  Apu apu;
  memset(&apu, 0, sizeof(apu));
  Spc *spc = spc_init(&apu);
  if (spc == NULL)
    return 1;

  uint8_t *request = malloc(kRequestSize);
  uint8_t *response = malloc(kResponseSize);
  if (request == NULL || response == NULL)
    return 1;

  while (fread(request, kRequestSize, 1, stdin) == 1) {
    spc->a = request[0];
    spc->x = request[1];
    spc->y = request[2];
    spc->sp = request[3];
    spc->pc = (uint16_t)request[4] | (uint16_t)request[5] << 8;
    set_flags(spc, request[6]);
    spc->stopped = request[7] != 0;
    memcpy(apu.ram, request + 8, sizeof(apu.ram));

    int cycles = spc_runOpcode(spc);
    response[0] = spc->a;
    response[1] = spc->x;
    response[2] = spc->y;
    response[3] = spc->sp;
    response[4] = (uint8_t)spc->pc;
    response[5] = (uint8_t)(spc->pc >> 8);
    response[6] = get_flags(spc);
    response[7] = spc->stopped;
    response[8] = (uint8_t)cycles;
    memcpy(response + 9, apu.ram, sizeof(apu.ram));
    if (fwrite(response, kResponseSize, 1, stdout) != 1 || fflush(stdout) != 0)
      return 1;
  }

  free(response);
  free(request);
  spc_free(spc);
  return feof(stdin) ? 0 : 1;
}
