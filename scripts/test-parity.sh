#!/bin/sh
set -eu

scripts/test-c-oracle.sh
cargo fmt --all --check
ruby scripts/audit-rust-port.rb
ruby scripts/test-replay-ranges.rb
cargo run -q -p z3-verify --bin check_runtime_snapshots
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -q -p z3-verify --bin check_oracle -- \
  build/c-oracle/stream_oracle zelda3_assets.dat
cargo run -q -p z3-verify --bin check_integrated_frame -- \
  build/c-oracle/stream_oracle zelda3_assets.dat 2000
cargo run -q -p z3-verify --bin check_features -- \
  build/c-oracle/stream_oracle zelda3_assets.dat
cargo run -q -p z3-verify --bin check_spc -- \
  build/c-oracle/spc_oracle
cargo run -q -p z3-verify --bin check_cpu -- \
  build/c-oracle/cpu_oracle
cargo run -q -p z3-verify --bin check_spc_player -- \
  build/c-oracle/spc_player_oracle zelda3_assets.dat
cargo run -q -p z3-verify --bin check_poly -- \
  build/c-oracle/poly_oracle
cargo run -q -p z3-verify --bin check_player_oam -- \
  build/c-oracle/player_oam_oracle
cargo run -q -p z3-verify --bin check_player_oam_main -- \
  build/c-oracle/player_oam_main_oracle
cargo run -q -p z3-verify --bin check_positions -- \
  build/c-oracle/position_oracle
cargo run -q -p z3-verify --bin check_stripes -- \
  build/c-oracle/stripe_oracle
cargo run -q -p z3-verify --bin check_sprite_state -- \
  build/c-oracle/sprite_state_oracle
cargo run -q -p z3-verify --bin check_tile_maps -- \
  build/c-oracle/tile_map_oracle
cargo run -q -p z3-verify --bin check_oam_allocator -- \
  build/c-oracle/oam_allocator_oracle
cargo run -q -p z3-verify --bin check_tile_detect -- \
  build/c-oracle/tile_detect_oracle
cargo run -q -p z3-verify --bin check_overlord -- \
  build/c-oracle/overlord_oracle
cargo run -q -p z3-verify --bin check_tagalong -- \
  build/c-oracle/tagalong_oracle
cargo run -q -p z3-verify --bin check_select_file -- \
  build/c-oracle/select_file_oracle zelda3_assets.dat
cargo run -q -p z3-verify --bin check_hud -- \
  build/c-oracle/hud_oracle zelda3_assets.dat
cargo run -q -p z3-verify --bin check_misc -- \
  build/c-oracle/misc_oracle zelda3_assets.dat
cargo run -q -p z3-verify --bin check_overworld_decompress -- \
  build/c-oracle/overworld_decompress_oracle
cargo run -q -p z3-verify --bin inspect_rom -- zelda3.sfc
scripts/test-full-replay.sh
cargo build -q -p z3-native --bin z3
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy \
  target/debug/z3 --config config/parity.ini --frames 600 zelda3.sfc
