//! Sound-effect latches, distinct from music and the queued APU transport.
//!
//! These ports retain the C overwrite/first-writer policies and packed pan bits.
//! Constructing a request never queues, clears, or changes game state.

use crate::{misc, ram::Ram};

/// The names describe the hardware destinations rather than C's off-by-one
/// `Sfx2`/`sound_effect_1` and `Sfx3`/`sound_effect_2` spellings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundPort {
    Ambient,
    EffectsA,
    EffectsB,
}

impl SoundPort {
    const fn latch_address(self) -> usize {
        match self {
            Self::Ambient => 0x12d,
            Self::EffectsA => 0x12e,
            Self::EffectsB => 0x12f,
        }
    }

    pub fn pending(self, ram: &Ram<'_>) -> u8 {
        ram.read_u8(self.latch_address())
    }

    pub fn overwrite(self, ram: &mut Ram<'_>, request: SoundRequest) {
        ram.write_u8(self.latch_address(), request.raw());
    }

    /// Zero is the idle value, even for a previously requested zero command.
    pub fn queue_if_idle(self, ram: &mut Ram<'_>, request: SoundRequest) -> bool {
        self.queue_if_idle_with(ram, |_| request)
    }

    /// Resolve an emitter only if the port accepts its request. This retains
    /// the original hook evaluation order for first-writer-wins callers.
    pub fn queue_if_idle_with(
        self,
        ram: &mut Ram<'_>,
        request: impl FnOnce(&Ram<'_>) -> SoundRequest,
    ) -> bool {
        if self.pending(ram) != 0 {
            return false;
        }
        let request = request(ram);
        self.overwrite(ram, request);
        true
    }

    pub fn clear(self, ram: &mut Ram<'_>) {
        self.overwrite(ram, SoundRequest::from_raw(0));
    }
}

/// Lossless packed effect/pan byte. IDs are not masked: some legacy commands
/// already contain pan bits, and OR-ing a further pan must preserve those bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoundRequest(u8);

impl SoundRequest {
    pub const fn from_raw(value: u8) -> Self {
        Self(value)
    }

    pub const fn with_pan(effect: u8, pan: u8) -> Self {
        Self(effect | pan)
    }

    pub fn at_player(ram: &Ram<'_>, effect: u8) -> Self {
        Self::with_pan(effect, misc::link_calculate_sfx_pan(ram))
    }

    pub fn at_world_x(ram: &Ram<'_>, effect: u8, x: u16) -> Self {
        Self::with_pan(effect, misc::calculate_sfx_pan(ram, x))
    }

    /// Dungeon tiles use an eight-zone pan table, not the world-coordinate rule.
    pub fn at_tile_x(ram: &Ram<'_>, effect: u8, x: u8) -> Self {
        Self::with_pan(effect, misc::calculate_sfx_pan_arbitrary(ram, x))
    }

    pub const fn raw(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;

    #[test]
    fn ports_keep_first_writer_and_overwrite_policies_separate() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        for (port, address) in [
            (SoundPort::Ambient, 0x12d),
            (SoundPort::EffectsA, 0x12e),
            (SoundPort::EffectsB, 0x12f),
        ] {
            assert!(port.queue_if_idle(&mut ram, SoundRequest::from_raw(0x82)));
            assert!(!port.queue_if_idle(&mut ram, SoundRequest::from_raw(0x1c)));
            assert!(!port.queue_if_idle_with(&mut ram, |_| panic!("occupied emitter resolved")));
            assert_eq!(ram.read_u8(address), 0x82);
            port.overwrite(&mut ram, SoundRequest::from_raw(0x1c));
            assert_eq!(port.pending(&ram), 0x1c);
            port.clear(&mut ram);
            assert!(port.queue_if_idle(&mut ram, SoundRequest::from_raw(0)));
            assert!(port.queue_if_idle(&mut ram, SoundRequest::from_raw(0xff)));
        }
    }

    #[test]
    fn emitter_coordinates_and_existing_pan_bits_are_preserved() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.write_u16(0xe2, 0x300);
        ram.write_u16(0x22, 0x380);
        assert_eq!(SoundRequest::at_player(&ram, 0x1c).raw(), 0x1c);
        assert_eq!(SoundRequest::at_world_x(&ram, 0x1c, 0x330).raw(), 0x9c);
        assert_eq!(SoundRequest::at_tile_x(&ram, 0x1e, 0xe0).raw(), 0x5e);
        for value in 0..=u8::MAX {
            assert_eq!(SoundRequest::from_raw(value).raw(), value);
            assert_eq!(SoundRequest::with_pan(value, 0x80).raw(), value | 0x80);
        }
    }
}
