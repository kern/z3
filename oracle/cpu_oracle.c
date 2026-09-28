#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "snes/cpu.h"
#include "snes/snes.h"

enum { kMaxWrites = 32, kResponseSize = 22 + kMaxWrites * 5 };

typedef struct Write {
  uint32_t address;
  uint8_t value;
} Write;

typedef struct FakeBus {
  uint32_t seed;
  uint32_t opcode_address;
  uint8_t opcode;
  uint8_t count;
  Write writes[kMaxWrites];
} FakeBus;

static uint8_t seeded_byte(uint32_t address, uint32_t seed) {
  uint32_t value = address ^ seed;
  value ^= value >> 16;
  value *= UINT32_C(0x7feb352d);
  value ^= value >> 15;
  value *= UINT32_C(0x846ca68b);
  value ^= value >> 16;
  return (uint8_t)(value >> 24);
}

uint8_t snes_cpuRead(Snes *snes, uint32_t address) {
  FakeBus *bus = (FakeBus *)snes;
  address &= 0xffffff;
  for (int i = bus->count - 1; i >= 0; i--)
    if (bus->writes[i].address == address)
      return bus->writes[i].value;
  return address == bus->opcode_address ? bus->opcode
                                        : seeded_byte(address, bus->seed);
}

void snes_cpuWrite(Snes *snes, uint32_t address, uint8_t value) {
  FakeBus *bus = (FakeBus *)snes;
  if (bus->count == kMaxWrites) {
    fprintf(stderr, "CPU oracle write log overflow\n");
    abort();
  }
  bus->writes[bus->count++] = (Write){address & 0xffffff, value};
}

void HookedFunctionRts(int is_long) { (void)is_long; }

static void put_u16(uint8_t **output, uint16_t value) {
  *(*output)++ = (uint8_t)value;
  *(*output)++ = (uint8_t)(value >> 8);
}

int main(void) {
  uint8_t request[2], response[kResponseSize];
  while (fread(request, sizeof(request), 1, stdin) == 1) {
    uint8_t opcode = request[0], scenario = request[1];
    FakeBus bus = {0};
    bus.seed = UINT32_C(0x9e3779b9) ^ opcode ^ (uint32_t)scenario << 24;
    bus.opcode = opcode;

    Cpu *cpu = cpu_init(&bus, 0);
    if (cpu == NULL)
      return 1;
    cpu->a = (uint16_t)(opcode * 257u + scenario);
    cpu->x = (uint16_t)(opcode * 131u + scenario);
    cpu->y = (uint16_t)(opcode * 67u + scenario);
    cpu->sp = (uint16_t)(0x1f0 + scenario);
    cpu->dp = (uint16_t)(scenario * 0x101);
    cpu->k = 0;
    cpu->pc = 0x8000;
    cpu->db = (uint8_t)(scenario * 0x55);
    cpu->e = (scenario & 1) != 0;
    cpu_setFlags(cpu, (uint8_t)(opcode ^ scenario * 0x55));
    cpu->irqWanted = cpu->nmiWanted = false;
    cpu->waiting = cpu->stopped = false;
    cpu->spBreakpoint = 0;
    cpu->in_emu = false;
    if (opcode == 0) {
      cpu->k = 7;
      cpu->pc = 0xb269;
    }
    bus.opcode_address = (uint32_t)cpu->k << 16 | cpu->pc;

    int cycles = cpu_runOpcode(cpu);
    uint8_t *out = response;
    put_u16(&out, cpu->a);
    put_u16(&out, cpu->x);
    put_u16(&out, cpu->y);
    put_u16(&out, cpu->sp);
    put_u16(&out, cpu->pc);
    put_u16(&out, cpu->dp);
    *out++ = cpu->k;
    *out++ = cpu->db;
    *out++ = cpu_getFlags(cpu);
    *out++ = cpu->e;
    *out++ = cpu->irqWanted;
    *out++ = cpu->nmiWanted;
    *out++ = cpu->waiting;
    *out++ = cpu->stopped;
    *out++ = (uint8_t)cycles;
    *out++ = bus.count;
    for (int i = 0; i < kMaxWrites; i++) {
      uint32_t address = bus.writes[i].address;
      *out++ = (uint8_t)address;
      *out++ = (uint8_t)(address >> 8);
      *out++ = (uint8_t)(address >> 16);
      *out++ = (uint8_t)(address >> 24);
      *out++ = bus.writes[i].value;
    }
    if (fwrite(response, sizeof(response), 1, stdout) != 1 || fflush(stdout) != 0)
      return 1;
    cpu_free(cpu);
  }
  return feof(stdin) ? 0 : 1;
}
