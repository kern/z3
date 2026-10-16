// Calls the real static music handler without changing the reference sources.
// Requests: command count (1..8), commands, raw RAM[65536], mapped RAM[65536].
// Each command replies with raw RAM, passive mapped RAM, live DSP[3024], then
// history count LE32 and all 256 register addresses and 256 values. No masks.
// The trusted checker supplies only bounded, aligned, in-bounds C programs;
// malformed SPC programs can loop forever or invoke undefined reference reads.
#include "src/spc_player.c"

enum { kRamSize = 65536, kDspSize = 3024, kMaxCommands = 8 };

static int emit(const void *bytes, size_t size) {
  return fwrite(bytes, 1, size, stdout) == size;
}

int main(void) {
  _Static_assert(sizeof(Dsp) - offsetof(Dsp, ram) == kDspSize,
                 "C DSP snapshot layout changed");
  uint8_t raw_ram[kRamSize];
  for (;;) {
    int count = getchar();
    if (count == EOF)
      return ferror(stdin) ? 1 : 0;
    uint8_t commands[kMaxCommands];
    if (count < 1 || count > kMaxCommands ||
        fread(commands, 1, count, stdin) != (size_t)count ||
        fread(raw_ram, 1, kRamSize, stdin) != kRamSize)
      return 1;
    SpcPlayer player = {0};
    Dsp dsp = {0};
    DspRegWriteHistory history = {0};
    player.dsp = &dsp;
    dsp.apu_ram = player.ram;
    dsp_reset(&dsp);
    player.reg_write_history = &history;
    for (int i = 0; i < 8; i++)
      player.channel[i].index = i;
    if (fread(player.ram, 1, kRamSize, stdin) != kRamSize)
      return 1;
    SpcPlayer_CopyVariablesFromRam(&player);
    memcpy(player.ram, raw_ram, kRamSize);
    for (int i = 0; i < count; i++) {
      if (commands[i] == 0xff)
        return 1; // Reference Not_Implemented, not a parity case.
      player.new_value_from_snes[0] = commands[i];
      Port0_HandleMusic(&player);
      SpcPlayer mapped = player;
      SpcPlayer_CopyVariablesToRam(&mapped);
      uint8_t history_count[4] = {
          (uint8_t)history.count, (uint8_t)(history.count >> 8),
          (uint8_t)(history.count >> 16), (uint8_t)(history.count >> 24)};
      if (!emit(player.ram, kRamSize) || !emit(mapped.ram, kRamSize) ||
          !emit(dsp.ram, kDspSize) || !emit(history_count, 4) ||
          !emit(history.addr, 256) || !emit(history.val, 256) ||
          fflush(stdout) != 0)
        return 1;
    }
  }
}
