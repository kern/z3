#include "oracle/c_oracle.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>


static uint32_t first_difference(const uint8_t *left, const uint8_t *right, size_t length) {
  for (size_t i = 0; i < length; i++) {
    if (left[i] != right[i])
      return (uint32_t)i;
  }
  return UINT32_MAX;
}

static void write_u32_le(uint8_t *dst, uint32_t value) {
  for (int i = 0; i < 4; i++)
    dst[i] = (uint8_t)(value >> (i * 8));
}

enum { kReplayAudioFrames = 534, kReplayAudioChannels = 2,
       kReplayAudioBytes = kReplayAudioFrames * kReplayAudioChannels * 2 };

static void compare_replay_audio(const uint8_t *expected, uint8_t *result) {
  int16_t samples[kReplayAudioFrames * kReplayAudioChannels];
  z3c_render_audio(samples);
  uint32_t first = UINT32_MAX;
  uint16_t c_sample = 0, rust_sample = 0;
  for (uint32_t i = 0; i < kReplayAudioFrames * kReplayAudioChannels; i++) {
    uint16_t actual = (uint16_t)samples[i];
    uint16_t wanted = (uint16_t)expected[i * 2] | (uint16_t)expected[i * 2 + 1] << 8;
    if (actual != wanted) {
      first = i;
      c_sample = actual;
      rust_sample = wanted;
      break;
    }
  }
  write_u32_le(result, first);
  result[4] = (uint8_t)c_sample;
  result[5] = (uint8_t)(c_sample >> 8);
  result[6] = (uint8_t)rust_sample;
  result[7] = (uint8_t)(rust_sample >> 8);
}

static uint8_t *read_file(const char *path, size_t *length) {
  FILE *file = fopen(path, "rb");
  if (file == NULL || fseek(file, 0, SEEK_END) != 0)
    return NULL;
  long size = ftell(file);
  if (size < 0 || fseek(file, 0, SEEK_SET) != 0) {
    fclose(file);
    return NULL;
  }
  uint8_t *data = malloc((size_t)size);
  if (data == NULL || fread(data, 1, (size_t)size, file) != (size_t)size) {
    free(data);
    fclose(file);
    return NULL;
  }
  fclose(file);
  *length = (size_t)size;
  return data;
}

int main(int argc, char **argv) {
  if (argc != 2) {
    fprintf(stderr, "usage: %s zelda3_assets.dat\n", argv[0]);
    return 2;
  }

  size_t assets_size;
  uint8_t *assets = read_file(argv[1], &assets_size);
  if (assets == NULL || !z3c_initialize(assets, assets_size)) {
    fprintf(stderr, "failed to initialize oracle\n");
    free(assets);
    return 1;
  }
  free(assets);

  size_t state_size = z3c_core_state_size();
  uint8_t *state = malloc(state_size);
  uint8_t *expected_state = malloc(state_size);
  uint8_t *pixels = NULL, *expected_pixels = NULL;
  size_t pixel_capacity = 0;
  if (state == NULL || expected_state == NULL)
    return 1;

  // Protocol: stdin contains repeated [input_lo, input_hi, run_what] records.
  // One canonical WRAM/SRAM/VRAM state is emitted after each complete record.
  // run_what == 0xff imports one canonical state immediately following the
  // command and emits that state again. run_what == 0xfe configures the feature
  // mask and extra side-space from five following bytes, then emits state.
  // run_what == 0xfd renders using a following [height_lo, height_hi, flags,
  // scale] record
  // run_what == 0xf9 consumes a 534-sample stereo PCM frame and compares it
  // with the C APU output after the most recent replay game frame.
  // run_what == 0xf6 does the replay step and compares canonical state and
  // audio without rendering, for a faster complete-recording audio sweep.
  // run_what == 0xf5 exports SPC RAM followed by the C DSP save block.
  // run_what == 0xf4 exports the 75-byte C APU write queue and input ports.
  // run_what == 0xf3 exports mapped SPC RAM and live DSP without changing RAM.
  // run_what == 0xfc imports a complete legacy device snapshot prefixed by a
  // little-endian uint32 length and emits its canonical state.
  // record and emits the tightly packed ARGB buffer instead of core state.
  // These values are not ZeldaRunFrameInternal modes, so ordinary three-byte
  // frame commands remain backwards compatible.
  uint8_t command[3];
  while (fread(command, sizeof(command), 1, stdin) == 1) {
    if (command[2] == 0xfb) {
      int chapter = (int)command[0] | (int)command[1] << 8;
      if (!z3c_load_reference_replay(chapter)) {
        fprintf(stderr, "oracle could not load reference replay %d\n", chapter);
        return 1;
      }
    } else if (command[2] == 0xf3) {
      uint8_t audio_state[65536 + 3024];
      z3c_export_audio_state_passive(audio_state);
      if (fwrite(audio_state, sizeof(audio_state), 1, stdout) != 1 || fflush(stdout) != 0) {
        fprintf(stderr, "oracle passive audio state export failed\n");
        return 1;
      }
      continue;
    } else if (command[2] == 0xf4) {
      uint8_t queue[75];
      z3c_export_audio_queue(queue);
      if (fwrite(queue, sizeof(queue), 1, stdout) != 1 || fflush(stdout) != 0) {
        fprintf(stderr, "oracle audio queue export failed\n");
        return 1;
      }
      continue;
    } else if (command[2] == 0xf5) {
      uint8_t audio_state[65536 + 3024];
      z3c_export_audio_state(audio_state);
      if (fwrite(audio_state, sizeof(audio_state), 1, stdout) != 1 || fflush(stdout) != 0) {
        fprintf(stderr, "oracle audio state export failed\n");
        return 1;
      }
      continue;
    } else if (command[2] == 0xf6) {
      if (command[0] != 0 || command[1] != 0) {
        fprintf(stderr, "oracle replay audio step command header invalid\n");
        return 1;
      }
      uint8_t expected_audio[kReplayAudioBytes], result[16];
      if (fread(expected_state, state_size, 1, stdin) != 1 ||
          fread(expected_audio, sizeof(expected_audio), 1, stdin) != 1) {
        fprintf(stderr, "oracle replay expected state/audio read failed\n");
        return 1;
      }
      int replaying = z3c_step_replay();
      if (!z3c_export_core_state(state, state_size)) {
        fprintf(stderr, "oracle replay state export failed\n");
        return 1;
      }
      write_u32_le(result, first_difference(state, expected_state, state_size));
      compare_replay_audio(expected_audio, result + 4);
      write_u32_le(result + 12, (uint32_t)replaying);
      if (fwrite(result, sizeof(result), 1, stdout) != 1 || fflush(stdout) != 0) {
        fprintf(stderr, "oracle replay state/audio comparison write failed\n");
        return 1;
      }
      continue;
    } else if (command[2] == 0xf9) {
      if (command[0] != 0 || command[1] != 0) {
        fprintf(stderr, "oracle replay audio command header invalid\n");
        return 1;
      }
      uint8_t expected[kReplayAudioBytes], result[8];
      if (fread(expected, sizeof(expected), 1, stdin) != 1) {
        fprintf(stderr, "oracle replay expected audio read failed\n");
        return 1;
      }
      compare_replay_audio(expected, result);
      if (fwrite(result, sizeof(result), 1, stdout) != 1 || fflush(stdout) != 0) {
        fprintf(stderr, "oracle replay audio comparison write failed\n");
        return 1;
      }
      continue;
    } else if (command[2] == 0xfa) {
      uint8_t render[4];
      if (fread(render, sizeof(render), 1, stdin) != 1) {
        fprintf(stderr, "oracle replay render configuration failed\n");
        return 1;
      }
      size_t width = (size_t)command[0] | (size_t)command[1] << 8;
      size_t height = (size_t)render[0] | (size_t)render[1] << 8;
      size_t scale = render[3];
      size_t pixel_bytes = width * height * scale * scale * 4;
      if ((scale != 1 && scale != 4) || pixel_bytes > (1 << 26)) {
        fprintf(stderr, "oracle replay invalid render size\n");
        return 1;
      }
      if (pixel_bytes > pixel_capacity) {
        uint8_t *new_pixels = realloc(pixels, pixel_bytes);
        uint8_t *new_expected = realloc(expected_pixels, pixel_bytes);
        if (new_pixels == NULL || new_expected == NULL) {
          fprintf(stderr, "oracle replay pixel allocation failed\n");
          return 1;
        }
        pixels = new_pixels;
        expected_pixels = new_expected;
        pixel_capacity = pixel_bytes;
      }
      if (fread(expected_state, state_size, 1, stdin) != 1 ||
          fread(expected_pixels, pixel_bytes, 1, stdin) != 1) {
        fprintf(stderr, "oracle replay expected frame read failed\n");
        return 1;
      }
      int replaying = z3c_step_replay();
      memset(pixels, 0, pixel_bytes);
      z3c_render(pixels, width * scale * 4, render[2]);
      if (!z3c_export_core_state(state, state_size)) {
        fprintf(stderr, "oracle replay state export failed\n");
        return 1;
      }
      uint8_t result[12];
      write_u32_le(result, first_difference(state, expected_state, state_size));
      write_u32_le(result + 4, first_difference(pixels, expected_pixels, pixel_bytes));
      write_u32_le(result + 8, (uint32_t)replaying);
      if (fwrite(result, sizeof(result), 1, stdout) != 1 || fflush(stdout) != 0) {
        fprintf(stderr, "oracle replay comparison write failed\n");
        return 1;
      }
      continue;
    } else if (command[2] == 0xf7) {
      uint8_t cgram[512];
      z3c_export_cgram(cgram);
      if (fwrite(cgram, sizeof(cgram), 1, stdout) != 1 || fflush(stdout) != 0)
        return 1;
      continue;
    } else if (command[2] == 0xf8) {
      // Export the current state after a failed compact replay comparison.
    } else if (command[2] == 0xff) {
      if (fread(state, state_size, 1, stdin) != 1 ||
          !z3c_import_core_state(state, state_size)) {
        fprintf(stderr, "oracle stream state import failed\n");
        free(state);
        return 1;
      }
    } else if (command[2] == 0xfc) {
      uint8_t length_bytes[4];
      if (fread(length_bytes, sizeof(length_bytes), 1, stdin) != 1) {
        fprintf(stderr, "oracle stream snapshot length failed\n");
        free(state);
        return 1;
      }
      size_t snapshot_size = (size_t)length_bytes[0] | (size_t)length_bytes[1] << 8 |
                             (size_t)length_bytes[2] << 16 | (size_t)length_bytes[3] << 24;
      uint8_t *snapshot = malloc(snapshot_size);
      if (snapshot == NULL || fread(snapshot, snapshot_size, 1, stdin) != 1 ||
          !z3c_import_snapshot(snapshot, snapshot_size)) {
        fprintf(stderr, "oracle stream snapshot import failed\n");
        free(snapshot);
        free(state);
        return 1;
      }
      free(snapshot);
    } else if (command[2] == 0xfe) {
      uint8_t config[5];
      if (fread(config, sizeof(config), 1, stdin) != 1) {
        fprintf(stderr, "oracle stream feature configuration failed\n");
        free(state);
        return 1;
      }
      uint32_t features = (uint32_t)config[0] | (uint32_t)config[1] << 8 |
                          (uint32_t)config[2] << 16 | (uint32_t)config[3] << 24;
      z3c_configure_features(features, config[4]);
    } else if (command[2] == 0xfd) {
      uint8_t render[4];
      if (fread(render, sizeof(render), 1, stdin) != 1) {
        fprintf(stderr, "oracle stream render configuration failed\n");
        free(state);
        return 1;
      }
      size_t width = (size_t)command[0] | (size_t)command[1] << 8;
      size_t height = (size_t)render[0] | (size_t)render[1] << 8;
      uint32_t flags = render[2];
      size_t scale = render[3];
      if (scale != 1 && scale != 4) {
        fprintf(stderr, "oracle stream invalid render scale\n");
        free(state);
        return 1;
      }
      size_t pixel_bytes = width * height * scale * scale * 4;
      uint8_t *pixels = calloc(pixel_bytes, 1);
      if (pixels == NULL) {
        free(state);
        return 1;
      }
      z3c_render(pixels, width * scale * 4, flags);
      int failed = fwrite(pixels, pixel_bytes, 1, stdout) != 1 || fflush(stdout) != 0;
      free(pixels);
      if (failed) {
        fprintf(stderr, "oracle stream render I/O failed\n");
        free(state);
        return 1;
      }
      continue;
    } else {
      uint16_t input = (uint16_t)command[0] | (uint16_t)command[1] << 8;
      z3c_step_raw(input, command[2]);
    }
    if (!z3c_export_core_state(state, state_size) ||
        fwrite(state, state_size, 1, stdout) != 1 || fflush(stdout) != 0) {
      fprintf(stderr, "oracle stream I/O failed\n");
      free(state);
      return 1;
    }
  }
  if (!feof(stdin)) {
    fprintf(stderr, "oracle stream input failed\n");
    free(state);
    return 1;
  }

  free(state);
  free(expected_state);
  free(pixels);
  free(expected_pixels);
  return 0;
}
