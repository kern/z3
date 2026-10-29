// Focused SFX oracle: actual static handlers from the unchanged reference.
// Stream requests: operation/channel/argument + raw[65536] + mapped[65536].
// Responses: return byte + raw + passive mapped + DSP[3024] + history
// countLE32/addresses[256]/values[256], nonmapped scalar bytes (timer,
// last-written EDL, input ports[4], voice indices[8]), stereo PCM[534*2]LE16.
// Optional argv PORT COMMAND FRAMES exercises public initialize/sample APIs.
// All programs are trusted bounded fixtures, never arbitrary malformed logs.
#include "src/spc_player.c"

enum { kRamSize = 65536, kDspSize = 3024, kSamples = 534 * 2 };

static int emit(const void *bytes, size_t size) {
  return fwrite(bytes, 1, size, stdout) == size;
}

static int response(SpcPlayer *player, uint8_t result, const int16_t *samples) {
  _Static_assert(sizeof(Dsp) - offsetof(Dsp, ram) == kDspSize,
                 "C DSP snapshot layout changed");
  SpcPlayer mapped = *player;
  SpcPlayer_CopyVariablesToRam(&mapped);
  DspRegWriteHistory *history = player->reg_write_history;
  uint8_t count[4] = {history->count, history->count >> 8,
                      history->count >> 16, history->count >> 24};
  uint8_t pcm[kSamples * 2];
  uint8_t scalars[14] = {player->timer_cycles, player->last_written_edl};
  memcpy(scalars + 2, player->input_ports, 4);
  for (int i = 0; i < 8; i++)
    scalars[6 + i] = player->channel[i].index;
  for (int i = 0; i < kSamples; i++) {
    pcm[i * 2] = samples[i];
    pcm[i * 2 + 1] = (uint16_t)samples[i] >> 8;
  }
  return emit(&result, 1) && emit(player->ram, kRamSize) &&
         emit(mapped.ram, kRamSize) && emit(player->dsp->ram, kDspSize) &&
         emit(count, 4) && emit(history->addr, 256) &&
         emit(history->val, 256) && emit(scalars, sizeof(scalars)) &&
         emit(pcm, sizeof(pcm)) && fflush(stdout) == 0;
}

static int public_samples(int port, int command, int frames) {
  if ((port != 2 && port != 3) ||
      (command != 0x40 && command != 0x80 && command != 0xc0) ||
      frames < 1 || frames > 32)
    return 2;
  SpcPlayer player = {0};
  Dsp dsp = {0};
  DspRegWriteHistory history = {0};
  player.dsp = &dsp;
  dsp.apu_ram = player.ram;
  SpcPlayer_Initialize(&player);
  player.reg_write_history = &history;
  memset(player.ram, 0, kRamSize);
  unsigned base = port == 2 ? 0x1820 : 0x191c;
  player.ram[base - 2] = 0;
  player.ram[base - 1] = 0x40;
  player.ram[base + 510] = 0;
  player.ram[base + 511] = 0x50;
  // Both targets are finite End tokens; all chain-table bytes remain zero.
  if (port == 2)
    player.ram[0x18dc] = 1;
  player.input_ports[port] = command;
  for (int frame = 0; frame < frames; frame++) {
    int16_t samples[kSamples];
    SpcPlayer_GenerateSamples(&player);
    dsp_getSamples(&dsp, samples, 534, 2);
    if (!response(&player, 0, samples))
      return 1;
  }
  return 0;
}

int main(int argc, char **argv) {
  if (argc == 4)
    return public_samples(atoi(argv[1]), atoi(argv[2]), atoi(argv[3]));
  if (argc != 1)
    return 2;
  uint8_t raw_ram[kRamSize];
  for (;;) {
    uint8_t request[3];
    size_t size = fread(request, 1, 3, stdin);
    if (size == 0 && feof(stdin))
      return 0;
    if (size != 3 || request[0] > 9 || request[1] >= 8 ||
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
    Channel *channel = &player.channel[request[1]];
    uint8_t result = 0;
    switch (request[0]) {
    case 0:
      if (request[2] > 1)
        return 1;
      Sfx_ChannelTick(&player, channel, request[2] != 0);
      break;
    case 1: Port1_StartNewSound(&player); break;
    case 2: Port2_StartNewSound(&player); break;
    case 3: Port3_StartNewSound(&player); break;
    case 4: Port1_Play_Inner(&player); break;
    case 5: result = Port2_AllocateChan(&player) - player.channel; break;
    case 6: result = Port3_AllocateChan(&player) - player.channel; break;
    case 7: Port2_HandleCmd(&player); break;
    case 8: Port3_HandleCmd(&player); break;
    case 9: Port1_HandleCmd(&player); break;
    }
    int16_t samples[kSamples] = {0};
    if (!response(&player, result, samples))
      return 1;
  }
}
