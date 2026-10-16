// Actual StateRecorder_Save/Load FILE APIs from the unchanged C library.
// Only complete, valid snapshots are sent here; malformed-file error behavior
// is deliberately not an oracle (the reference aborts/asserts).
#include "oracle/c_oracle.h"
#include "src/util.h"
#include "src/zelda_rtl.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Exact private ABI from src/zelda_rtl.c. The checker audits this declaration
// token-for-token against the original typedef before starting this process.
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

extern void StateRecorder_Save(StateRecorder *sr, FILE *f);
extern void StateRecorder_Load(StateRecorder *sr, FILE *f, bool replay_mode);

static int read_u32(uint32 *v) {
  uint8 b[4];
  if (fread(b, 1, 4, stdin) != 4)
    return 0;
  *v = (uint32)b[0] | (uint32)b[1] << 8 | (uint32)b[2] << 16 |
       (uint32)b[3] << 24;
  return 1;
}

static int write_u32(uint32 v) {
  uint8 b[4] = {v, v >> 8, v >> 16, v >> 24};
  return fwrite(b, 1, 4, stdout) == 4;
}

static int read_array(ByteArray *array, uint32 size) {
  ByteArray_Resize(array, size);
  return size == 0 || fread(array->data, 1, size, stdin) == size;
}

static int emit_save(StateRecorder *sr) {
  FILE *file = tmpfile();
  if (!file)
    return 0;
  StateRecorder_Save(sr, file);
  long size = ftell(file);
  if (size < 0 || (unsigned long)size > UINT32_MAX ||
      fseek(file, 0, SEEK_SET) != 0) {
    fclose(file);
    return 0;
  }
  uint8 buffer[8192];
  int ok = fwrite(g_zenv.ram + 0x1fff0, 1, 4, stdout) == 4 &&
           write_u32(sr->replay_mode) && write_u32((uint32)size);
  size_t count;
  while (ok && (count = fread(buffer, 1, sizeof(buffer), file)) != 0)
    ok = fwrite(buffer, 1, count, stdout) == count;
  ok = ok && !ferror(file) && fflush(stdout) == 0;
  fclose(file);
  return ok;
}

int main(int argc, char **argv) {
  if (argc != 2)
    return 2;
  size_t assets_size;
  uint8 *assets = ReadWholeFile(argv[1], &assets_size);
  int initialized = assets && z3c_initialize(assets, assets_size);
  free(assets);
  if (!initialized)
    return 1;
  StateRecorder initial = {0};
  if (!emit_save(&initial))
    return 1;
  for (;;) {
    int operation = getchar();
    if (operation == EOF)
      return ferror(stdin) ? 1 : 0;
    StateRecorder sr = {0};
    int ok = 0;
    if (operation == 0) {
      // total, inputs, elapsed, completed cursor, replay frame, replay flag,
      // log length, base length, and an installed WRAM marker.
      uint32 fields[9];
      for (int i = 0; i < 9; i++)
        if (!read_u32(&fields[i]))
          return 1;
      sr.total_frames = fields[0];
      sr.last_inputs = fields[1];
      sr.frames_since_last = fields[2];
      sr.replay_pos_last_complete = fields[3];
      sr.replay_frame_counter = fields[4];
      sr.replay_mode = fields[5] != 0;
      if (!read_array(&sr.log, fields[6]) ||
          !read_array(&sr.base_snapshot, fields[7]))
        return 1;
      for (int i = 0; i < 4; i++)
        g_zenv.ram[0x1fff0 + i] = fields[8] >> (i * 8);
      ok = emit_save(&sr);
    } else if (operation == 1) {
      int replay = getchar();
      uint32 size;
      if (replay < 0 || replay > 1 || !read_u32(&size))
        return 1;
      FILE *file = tmpfile();
      if (!file)
        return 1;
      uint8 buffer[8192];
      while (size != 0) {
        size_t count = size < sizeof(buffer) ? size : sizeof(buffer);
        if (fread(buffer, 1, count, stdin) != count ||
            fwrite(buffer, 1, count, file) != count)
          return 1;
        size -= count;
      }
      rewind(file);
      StateRecorder_Load(&sr, file, replay != 0);
      fclose(file);
      ok = emit_save(&sr);
    }
    ByteArray_Destroy(&sr.log);
    ByteArray_Destroy(&sr.base_snapshot);
    if (!ok)
      return 1;
  }
}
