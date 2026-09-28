# C reference baseline

The Rust parity work is based on upstream `snesrev/zelda3` commit:

`fbbb3f967a51fafe642e6140d0753979e73b4090`

The reference build is intentionally behavior-preserving. Apple Clang 21
requires `-Wno-deprecated-non-prototype` for the upstream 2023 C declarations;
no C source change is needed.

## Local inputs

- Required US ROM SHA-256:
  `66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb`
- The ROM and generated `zelda3_assets.dat` are gitignored.
- Python is used only to extract and compile assets; it is not a game runtime
  dependency.

## Reproduce

```sh
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements.txt
scripts/build-c-reference.sh
cargo test --workspace
# Full C/Rust oracle and deterministic-state gate:
scripts/test-parity.sh
```

Initial successful local build artifacts:

- `zelda3_assets.dat`: 683,888 bytes,
  SHA-256 `0fe2e4bd75d70f06fb9a74cd3a9cb336c838149b831b56e8792114a89292c793`
- `zelda3`: 1,764,680 bytes,
  SHA-256 `d9bef0aeea8d469cd5213e52810ecb25be85834bea7ffbd0e469193723aa6b80`

The Rust container reader validates the generated bundle as 165 indexed assets,
a 3,252-byte key table, and 679,700 bytes of aggregate payload.

The reference executable was also smoke-tested with the parity configuration,
dummy SDL video/audio drivers, and the ROM comparison path enabled. It loaded
the LoROM and reached the game loop without an initial state mismatch.

The executable hash records this exact local compiler build; deterministic game
state and output hashes, rather than executable bytes, are the parity contract.
