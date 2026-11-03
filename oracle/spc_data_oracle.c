// Actual unchanged static instrument/pan/SFX functions and public upload/sample
// APIs. Trusted requests only: valid complete uploads with aligned headers,
// bounded SFX streams. No malformed-C or aliased-source coverage is claimed.
#include "src/spc_player.c"

enum { kRam = 65536, kDsp = 3024, kPcm = 534 * 2 };
static int emit(const void *p, size_t n) { return fwrite(p, 1, n, stdout) == n; }
static int read_bytes(void *p, size_t n) { return fread(p, 1, n, stdin) == n; }
static int response(SpcPlayer *p, const int16_t *samples, uint8_t result) {
  _Static_assert(sizeof(Dsp) - offsetof(Dsp, ram) == kDsp, "DSP ABI drift");
  SpcPlayer mapped = *p;
  SpcPlayer_CopyVariablesToRam(&mapped); // Never mutate the live echo RAM.
  DspRegWriteHistory *h = p->reg_write_history;
  uint8_t count[4] = {h->count, h->count >> 8, h->count >> 16, h->count >> 24};
  uint8_t scalars[14] = {p->timer_cycles, p->last_written_edl};
  memcpy(scalars + 2, p->input_ports, 4);
  for (int i = 0; i < 8; i++) scalars[6 + i] = p->channel[i].index;
  uint8_t pcm[kPcm * 2];
  for (int i = 0; i < kPcm; i++) {
    pcm[i * 2] = samples[i];
    pcm[i * 2 + 1] = (uint16_t)samples[i] >> 8;
  }
  return emit(&result, 1) && emit(p->ram, kRam) && emit(mapped.ram, kRam) &&
         emit(p->dsp->ram, kDsp) && emit(count, 4) && emit(h->addr, 256) &&
         emit(h->val, 256) && emit(scalars, 14) && emit(pcm, sizeof(pcm)) &&
         fflush(stdout) == 0;
}

int main(void) {
  uint8_t raw[kRam];
  for (;;) {
    int n = getchar();
    if (n == EOF) return ferror(stdin) ? 1 : 0;
    if (n < 1 || n > 32 || !read_bytes(raw, kRam)) return 1;
    SpcPlayer p = {0};
    Dsp dsp = {0};
    DspRegWriteHistory history = {0};
    p.dsp = &dsp;
    dsp.apu_ram = p.ram;
    dsp_reset(&dsp);
    p.reg_write_history = &history;
    for (int i = 0; i < 8; i++) p.channel[i].index = i;
    if (!read_bytes(p.ram, kRam)) return 1;
    SpcPlayer_CopyVariablesFromRam(&p);
    memcpy(p.ram, raw, kRam);
    uint8_t nonmapped[6];
    if (!read_bytes(nonmapped, sizeof(nonmapped)) || nonmapped[0] > 64) return 1;
    p.timer_cycles = nonmapped[0];
    p.last_written_edl = nonmapped[1];
    memcpy(p.input_ports, nonmapped + 2, 4);
    for (int step = 0; step < n; step++) {
      uint8_t op[4];
      if (!read_bytes(op, 4) || op[1] >= 8) return 1;
      unsigned arg = op[2] | op[3] << 8;
      int16_t pcm[kPcm] = {0};
      uint8_t result = 0;
      switch (op[0]) {
      case 0: Channel_SetInstrument(&p, &p.channel[op[1]], arg); break;
      case 1: WriteVolumeToDsp(&p, &p.channel[op[1]], arg); break;
      case 2:
      case 4: {
        uint8_t size_bytes[4];
        if (!read_bytes(size_bytes, 4)) return 1;
        unsigned size = size_bytes[0] | size_bytes[1] << 8 |
                        size_bytes[2] << 16 | size_bytes[3] << 24;
        if (size > 262144) return 1;
        // malloc provides uint16 alignment for the actual C header casts.
        uint8_t *payload = malloc(size ? size : 1);
        if (!payload || !read_bytes(payload, size)) return 1;
        if (op[0] == 2) {
          // Independently reject malformed/unaligned fixtures before invoking
          // C's API, which has no size parameter or error contract.
          size_t cursor = 0;
          for (;;) {
            if ((cursor & 1) || cursor + 2 > size) return 1;
            unsigned length = payload[cursor] | payload[cursor + 1] << 8;
            if (length == 0) break;
            if (cursor + 4 + length > size) return 1;
            cursor += 4 + length;
          }
          SpcPlayer_Upload(&p, payload);
          result = 1;
        } else {
          for (unsigned i = 0; i < size; i++) p.ram[(arg + i) & 0xffff] = payload[i];
        }
        free(payload);
        break;
      }
      case 3:
        SpcPlayer_GenerateSamples(&p);
        dsp_getSamples(&dsp, pcm, 534, 2);
        break;
      case 5:
        p.ram[0x7000] = 0xe0;
        p.ram[0x7001] = arg;
        p.ram[0x7002] = 0xc9; // Hold: avoids replacing the just-read preset.
        p.sfx_sound_ptr_cur = 0x7000;
        Sfx_ChannelTick(&p, &p.channel[op[1]], false);
        break;
      case 6: {
        // Real DSP writes key on the uploaded BRR directory/sample. The
        // public player sample API below observes the data, not a fake mixer.
        const uint8_t registers[] = {DIR, MVOLL, MVOLR, V0VOLL, V0VOLR,
            V0PITCHL, V0PITCHH, V0SRCN, V0ADSR1, V0GAIN, FLG, KOF, KON};
        const uint8_t values[] = {0x3c, 0x7f, 0x7f, 0x60, 0x60,
            0, 0x10, 0, 0, 0x7f, 0x20, 0, 1};
        for (unsigned i = 0; i < sizeof(registers); i++)
          Dsp_Write(&p, registers[i], values[i]);
        break;
      }
      default: return 1;
      }
      if (!response(&p, pcm, result)) return 1;
    }
  }
}
