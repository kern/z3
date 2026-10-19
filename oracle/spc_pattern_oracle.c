// Trusted, bounded SPC programs only. Calls unchanged static C handlers.
// Request: operation/channel/argument, raw RAM[65536], mapped RAM[65536].
// Reply: return byte, raw RAM, passive mapped RAM, live DSP save block, then
// complete history count LE32/address[256]/value[256]. No comparison masks.
#include "src/spc_player.c"

enum { kRamSize = 65536, kDspSize = 3024 };

static int emit(const void *bytes, size_t size) {
  return fwrite(bytes, 1, size, stdout) == size;
}

int main(void) {
  _Static_assert(sizeof(Dsp) - offsetof(Dsp, ram) == kDspSize,
                 "C DSP snapshot layout changed");
  uint8_t raw_ram[kRamSize];
  for (;;) {
    uint8_t request[3];
    size_t count = fread(request, 1, 3, stdin);
    if (count == 0 && feof(stdin))
      return 0;
    if (count != 3 || request[0] > 3 || request[1] >= 8 ||
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
    uint8_t result = 0;
    Channel *channel = &player.channel[request[1]];
    switch (request[0]) {
    case 0:
      if (request[2] < 0xe0 || request[2] > 0xfa ||
          (request[2] >= 0xf1 && request[2] <= 0xf3))
        return 1;
      HandleEffect(&player, channel, request[2]);
      break;
    case 1:
      PitchSlideToNote_Check(&player, channel);
      break;
    case 2:
      result = WantWriteKof(&player, channel);
      break;
    case 3:
      if (request[2] == 0xff)
        return 1;
      player.new_value_from_snes[0] = request[2];
      Port0_HandleMusic(&player);
      break;
    }
    SpcPlayer mapped = player;
    SpcPlayer_CopyVariablesToRam(&mapped);
    uint8_t history_count[4] = {
        (uint8_t)history.count, (uint8_t)(history.count >> 8),
        (uint8_t)(history.count >> 16), (uint8_t)(history.count >> 24)};
    if (!emit(&result, 1) || !emit(player.ram, kRamSize) ||
        !emit(mapped.ram, kRamSize) || !emit(dsp.ram, kDspSize) ||
        !emit(history_count, 4) || !emit(history.addr, 256) ||
        !emit(history.val, 256) || fflush(stdout) != 0)
      return 1;
  }
}
