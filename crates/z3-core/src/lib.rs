//! Behavior-preserving Rust port of Zelda3's deterministic game core.
//!
//! During the parity phase this crate intentionally retains the legacy memory
//! layout. Higher-level Rust state models come only after frame-by-frame parity.

pub mod assets;
pub mod c_types;
pub mod config;
pub mod legacy_memory;
pub mod util;

pub use legacy_memory::{CanonicalState, LegacyMemory, SRAM_SIZE, VRAM_WORDS, WRAM_SIZE};
