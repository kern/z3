#ifndef ZRUST_C_ORACLE_H_
#define ZRUST_C_ORACLE_H_

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif
enum {
  Z3C_WRAM_SIZE = 0x20000,
  Z3C_SRAM_SIZE = 0x2000,
  Z3C_VRAM_SIZE = 0x10000,
  Z3C_CORE_STATE_SIZE = Z3C_WRAM_SIZE + Z3C_SRAM_SIZE + Z3C_VRAM_SIZE,
};

// Initializes the singleton C reference using an in-memory asset container.
// The oracle owns a private copy after this call returns.
int z3c_initialize(const uint8_t *assets, size_t assets_len);

void z3c_reset(int preserve_sram);
void z3c_step_raw(uint16_t input, uint8_t run_what);
void z3c_configure_features(uint32_t features, uint8_t extra_left_right);

size_t z3c_core_state_size(void);
int z3c_export_core_state(uint8_t *destination, size_t capacity);
int z3c_import_core_state(const uint8_t *source, size_t length);
int z3c_import_snapshot(const uint8_t *source, size_t length);

// Renders the current PPU state into a caller-owned ARGB buffer.
void z3c_render(uint8_t *pixels, size_t pitch, uint32_t render_flags);

#ifdef __cplusplus
}
#endif

#endif
