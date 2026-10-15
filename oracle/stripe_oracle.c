// Asset-free differential harness for the unmodified C stripe decoder.
#include <stdint.h>
#include <stdio.h>
#include "src/nmi.h"
#include "src/zelda_rtl.h"

int main(void) {
  static uint16 words[0x8000];
  static uint8 image[0x10000], stream[4096];
  g_zenv.vram = words;
  for (;;) {
    uint8 header[4];
    size_t count = fread(header, 1, 4, stdin);
    if (count == 0 && feof(stdin))
      return 0;
    if (count != 4)
      return 1;
    uint32 length = header[0] | (uint32)header[1] << 8 |
                    (uint32)header[2] << 16 | (uint32)header[3] << 24;
    if (!length || length > sizeof(stream) ||
        fread(image, 1, sizeof(image), stdin) != sizeof(image) ||
        fread(stream, 1, length, stdin) != length)
      return 1;
    for (int i = 0; i < 0x8000; i++)
      words[i] = image[i * 2] | (uint16)image[i * 2 + 1] << 8;
    HandleStripes14(stream);
    for (int i = 0; i < 0x8000; i++) {
      image[i * 2] = words[i];
      image[i * 2 + 1] = words[i] >> 8;
    }
    if (fwrite(image, 1, sizeof(image), stdout) != sizeof(image) || fflush(stdout))
      return 1;
  }
}
