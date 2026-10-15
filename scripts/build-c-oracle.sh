#!/bin/sh
set -eu

scripts/build-c-reference.sh

mkdir -p build/c-oracle

common_flags='-O2 -Werror -Wno-deprecated-non-prototype -fPIC -I . -I/opt/homebrew/include/SDL2 -D_THREAD_SAFE -DSYSTEM_VOLUME_MIXER_AVAILABLE=0'

clang -c $common_flags oracle/c_oracle.c -o build/c-oracle/c_oracle.o

clang -dynamiclib -o build/c-oracle/libz3c_oracle.dylib \
  build/c-oracle/c_oracle.o \
  src/ancilla.o src/attract.o src/audio.o src/dungeon.o src/ending.o \
  src/hud.o src/load_gfx.o src/messaging.o src/misc.o src/nmi.o \
  src/overlord.o src/overworld.o src/player.o src/player_oam.o src/poly.o \
  src/select_file.o src/spc_player.o src/sprite.o src/sprite_main.o \
  src/tagalong.o src/tile_detect.o src/util.o src/zelda_rtl.o \
  snes/dma.o snes/dsp.o snes/ppu.o \
  third_party/opus-1.3.1-stripped/opus_decoder_amalgam.o \
  -L/opt/homebrew/lib -lSDL2 -lm

clang $common_flags oracle/smoke_oracle.c \
  -Lbuild/c-oracle -lz3c_oracle \
  -Wl,-rpath,@loader_path \
  -o build/c-oracle/smoke_oracle

clang $common_flags oracle/stream_oracle.c \
  -Lbuild/c-oracle -lz3c_oracle \
  -Wl,-rpath,@loader_path \
  -o build/c-oracle/stream_oracle

clang $common_flags oracle/spc_oracle.c snes/spc.o \
  -o build/c-oracle/spc_oracle

clang $common_flags oracle/cpu_oracle.c snes/cpu.o \
  -o build/c-oracle/cpu_oracle

clang $common_flags oracle/spc_player_oracle.c src/spc_player.o snes/dsp.o \
  -o build/c-oracle/spc_player_oracle

clang $common_flags oracle/poly_oracle.c src/poly.o \
  -o build/c-oracle/poly_oracle

clang $common_flags oracle/player_oam_oracle.c src/player_oam.o \
  -o build/c-oracle/player_oam_oracle

clang $common_flags oracle/player_oam_main_oracle.c src/player_oam.o \
  -o build/c-oracle/player_oam_main_oracle

clang $common_flags oracle/position_oracle.c \
  -Lbuild/c-oracle -lz3c_oracle \
  -Wl,-rpath,@loader_path \
  -o build/c-oracle/position_oracle

clang $common_flags oracle/stripe_oracle.c \
  -Lbuild/c-oracle -lz3c_oracle \
  -Wl,-rpath,@loader_path \
  -o build/c-oracle/stripe_oracle

clang $common_flags oracle/sprite_state_oracle.c \
  -Lbuild/c-oracle -lz3c_oracle \
  -Wl,-rpath,@loader_path \
  -o build/c-oracle/sprite_state_oracle

clang $common_flags oracle/tile_map_oracle.c \
  -Lbuild/c-oracle -lz3c_oracle \
  -Wl,-rpath,@loader_path \
  -o build/c-oracle/tile_map_oracle

clang $common_flags oracle/oam_allocator_oracle.c \
  -Lbuild/c-oracle -lz3c_oracle \
  -Wl,-rpath,@loader_path \
  -o build/c-oracle/oam_allocator_oracle

clang $common_flags oracle/tile_detect_oracle.c src/tile_detect.o \
  -o build/c-oracle/tile_detect_oracle

clang $common_flags oracle/overlord_oracle.c src/overlord.o \
  -o build/c-oracle/overlord_oracle

clang $common_flags oracle/tagalong_oracle.c src/tagalong.o \
  -o build/c-oracle/tagalong_oracle

clang $common_flags oracle/select_file_oracle.c \
  -Lbuild/c-oracle -lz3c_oracle \
  -Wl,-rpath,@loader_path \
  -o build/c-oracle/select_file_oracle

clang $common_flags oracle/hud_oracle.c \
  -Lbuild/c-oracle -lz3c_oracle \
  -Wl,-rpath,@loader_path \
  -o build/c-oracle/hud_oracle

clang $common_flags oracle/misc_oracle.c \
  -Lbuild/c-oracle -lz3c_oracle \
  -Wl,-rpath,@loader_path \
  -o build/c-oracle/misc_oracle

clang $common_flags oracle/overworld_decompress_oracle.c \
  -Lbuild/c-oracle -lz3c_oracle \
  -Wl,-rpath,@loader_path \
  -o build/c-oracle/overworld_decompress_oracle
