// Defined recorder operations from the unchanged C implementation. This is
// not a malformed-log oracle: the trusted checker supplies bounded programs.
#include "oracle/c_oracle.h"
#include "src/util.h"
#include "src/zelda_rtl.h"
#include "src/spc_player.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

// Source-audited against src/zelda_rtl.c, not an independent implementation.
typedef struct StateRecorder {
  uint16 last_inputs;
  uint32 frames_since_last;
  uint32 total_frames;
  uint32 replay_pos, replay_pos_last_complete;
  uint32 replay_frame_counter;
  uint32 replay_next_cmd_at;
  uint8 replay_cmd;
  bool replay_mode;
  ByteArray log;
  ByteArray base_snapshot;
} StateRecorder;

extern void StateRecorder_Record(StateRecorder *, uint16);
extern void StateRecorder_RecordPatchByte(StateRecorder *, uint32, const uint8 *, int);
extern uint16 StateRecorder_ReadNextReplayState(StateRecorder *);
extern void StateRecorder_StopReplay(StateRecorder *);
extern void StateRecorder_ClearKeyLog(StateRecorder *);
extern void StateRecorder_Save(StateRecorder *, FILE *);

static FILE *wire;

static int read_u32(uint32 *value) {
  uint8 b[4];
  if (fread(b, 1, 4, stdin) != 4)
    return 0;
  *value = (uint32)b[0] | (uint32)b[1] << 8 | (uint32)b[2] << 16 |
           (uint32)b[3] << 24;
  return 1;
}

static int write_u32(uint32 value) {
  uint8 b[4] = {value, value >> 8, value >> 16, value >> 24};
  return fwrite(b, 1, 4, wire) == 4;
}

static int emit(const StateRecorder *sr, uint32 returned) {
  uint32 values[] = {returned, sr->last_inputs, sr->frames_since_last,
    sr->total_frames, sr->replay_pos, sr->replay_pos_last_complete,
    sr->replay_frame_counter, sr->replay_next_cmd_at, sr->replay_cmd,
    sr->replay_mode, sr->log.size, sr->base_snapshot.size};
  for (int i = 0; i < 12; i++)
    if (!write_u32(values[i]))
      return 0;
  return fwrite(g_zenv.ram, 1, 0x20000, wire) == 0x20000 &&
    fwrite(sr->log.data, 1, sr->log.size, wire) == sr->log.size &&
    fwrite(sr->base_snapshot.data, 1, sr->base_snapshot.size, wire) == sr->base_snapshot.size &&
    fflush(wire) == 0;
}

static int canonicalize_snapshot_boundary(StateRecorder *sr) {
  FILE *file = tmpfile();
  if (!file)
    return 0;
  StateRecorder_Save(sr, file);
  return fclose(file) == 0;
}

int main(int argc, char **argv) {
  if (argc != 2)
    return 2;
  // Original ClearKeyLog prints a diagnostic to stdout. Keep the actual C
  // call, but move all legacy diagnostics away from the binary wire.
  wire = fdopen(dup(STDOUT_FILENO), "wb");
  if (!wire || !freopen("/dev/null", "w", stdout))
    return 1;
  size_t assets_size;
  uint8 *assets = ReadWholeFile(argv[1], &assets_size);
  int initialized = assets && z3c_initialize(assets, assets_size);
  free(assets);
  if (!initialized)
    return 1;
  StateRecorder sr = {0};
  for (;;) {
    int operation = getchar();
    if (operation == EOF)
      return ferror(stdin) ? 1 : 0;
    uint32 returned = UINT32_MAX;
    if (operation == 0) {
      ByteArray_Destroy(&sr.log);
      ByteArray_Destroy(&sr.base_snapshot);
      memset(&sr, 0, sizeof(sr));
      z3c_reset(false);
      uint32 values[10];
      for (int i = 0; i < 10; i++)
        if (!read_u32(&values[i]))
          return 1;
      sr.last_inputs = values[0];
      sr.frames_since_last = values[1];
      sr.total_frames = values[2];
      sr.replay_pos = values[3];
      sr.replay_pos_last_complete = values[4];
      sr.replay_frame_counter = values[5];
      sr.replay_next_cmd_at = values[6];
      sr.replay_cmd = values[7];
      sr.replay_mode = values[8] != 0;
      ByteArray_Resize(&sr.log, values[9]);
      if (fread(sr.log.data, 1, sr.log.size, stdin) != sr.log.size ||
          fread(g_zenv.ram, 1, 0x20000, stdin) != 0x20000 ||
          !canonicalize_snapshot_boundary(&sr))
        return 1;
    } else if (operation == 1) {
      uint32 inputs;
      if (!read_u32(&inputs))
        return 1;
      StateRecorder_Record(&sr, inputs);
    } else if (operation == 2) {
      uint32 address, size;
      if (!read_u32(&address) || !read_u32(&size) || size == 0 || size > 4096)
        return 1;
      uint8 bytes[4096];
      if (fread(bytes, 1, size, stdin) != size)
        return 1;
      StateRecorder_RecordPatchByte(&sr, address, bytes, size);
    } else if (operation == 3) {
      returned = StateRecorder_ReadNextReplayState(&sr);
    } else if (operation == 4) {
      StateRecorder_StopReplay(&sr);
    } else if (operation == 5) {
      uint8 before_ram[0x20000], before_spc[0x10000];
      memcpy(before_ram, g_zenv.ram, sizeof(before_ram));
      memcpy(before_spc, g_zenv.player->ram, sizeof(before_spc));
      StateRecorder_ClearKeyLog(&sr);
      // This focused scheduler test intentionally uses an idempotent native
      // snapshot boundary. No serializer memory effects are silently masked.
      if (memcmp(before_ram, g_zenv.ram, sizeof(before_ram)) != 0 ||
          memcmp(before_spc, g_zenv.player->ram, sizeof(before_spc)) != 0) {
        fprintf(stderr, "noncanonical ClearKeyLog fixture; scheduler-only oracle refuses it\n");
        return 1;
      }
    } else {
      return 1;
    }
    if (!emit(&sr, returned))
      return 1;
  }
}
