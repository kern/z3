//! Live, named views of the sprite pool's structure-of-arrays storage.
//!
//! Slot identity and iteration order are independent of activity: inactive
//! slots retain every byte. Values are lossless bytes, including unknown
//! state/flag values. The canonical WRAM image remains the only storage.

use crate::position::{Height, SplitPosition};
use crate::ram::Ram;
use crate::velocity::VelocityByte;
use z3_core::WRAM_SIZE;

pub const SPRITE_CAPACITY: usize = 16;

const X_POSITION: SplitPosition<16> = SplitPosition::world(
    layout::SPRITE_X_SUBPIXEL,
    layout::SPRITE_X_LO,
    layout::SPRITE_X_HI,
);
const Y_POSITION: SplitPosition<16> = SplitPosition::world(
    layout::SPRITE_Y_SUBPIXEL,
    layout::SPRITE_Y_LO,
    layout::SPRITE_Y_HI,
);
const HEIGHT: SplitPosition<8> = SplitPosition::height(layout::SPRITE_Z_SUBPOS, layout::SPRITE_Z);

/// A validated slot in the physical sprite pool. Slots are never compacted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpriteSlot(u8);

impl SpriteSlot {
    #[must_use]
    pub fn new(index: usize) -> Option<Self> {
        (index < SPRITE_CAPACITY).then_some(Self(index as u8))
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    pub fn all() -> impl ExactSizeIterator<Item = Self> + DoubleEndedIterator {
        (0..SPRITE_CAPACITY as u8).map(Self)
    }
}

#[derive(Clone, Copy)]
pub struct SpritePool<'a> {
    bytes: &'a [u8],
}

impl<'a> SpritePool<'a> {
    #[must_use]
    pub fn get(self, slot: SpriteSlot) -> SpriteRef<'a> {
        self.at(slot.index())
    }

    /// Addresses a legacy index without clamping it to the physical pool.
    ///
    /// Some translated calls intentionally read with a sentinel parent such
    /// as 0xff. Preserve the original `field_base + index` behavior there;
    /// use [`Self::get`] for a validated physical slot.
    #[must_use]
    pub fn at(self, index: usize) -> SpriteRef<'a> {
        SpriteRef {
            bytes: self.bytes,
            index,
        }
    }

    /// Finds the highest inactive physical slot without clearing its bytes.
    #[must_use]
    pub fn find_inactive_rev(self) -> Option<SpriteSlot> {
        SpriteSlot::all()
            .rev()
            .find(|&slot| self.get(slot).state() == 0)
    }
}

pub struct SpritePoolMut<'a> {
    bytes: &'a mut [u8],
}

impl<'a> SpritePoolMut<'a> {
    #[must_use]
    pub fn get(self, slot: SpriteSlot) -> SpriteMut<'a> {
        self.at(slot.index())
    }

    /// Mutable counterpart of [`SpritePool::at`], with identical indexing.
    #[must_use]
    pub fn at(self, index: usize) -> SpriteMut<'a> {
        SpriteMut {
            bytes: self.bytes,
            index,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SpriteRef<'a> {
    bytes: &'a [u8],
    index: usize,
}

pub struct SpriteMut<'a> {
    bytes: &'a mut [u8],
    index: usize,
}

impl SpriteMut<'_> {
    pub fn approach_velocity(&mut self, x: u8, y: u8) {
        for (field, target) in [(SpriteField::VelocityX, x), (SpriteField::VelocityY, y)] {
            let value = self.as_read_only().field(field);
            if value != target {
                self.set_field(
                    field,
                    VelocityByte::from_bits(value)
                        .approaching(VelocityByte::from_bits(target))
                        .bits(),
                );
            }
        }
    }

    #[must_use]
    pub fn as_read_only(&self) -> SpriteRef<'_> {
        SpriteRef {
            bytes: self.bytes,
            index: self.index,
        }
    }
}

impl Ram<'_> {
    pub fn move_sprite_x(&mut self, index: usize) {
        let velocity = self.sprite(index).velocity_x() as i8;
        if velocity == 0 {
            return;
        }
        let position = X_POSITION.load(self, index);
        X_POSITION.store(self, index, position.advanced(velocity));
    }

    pub fn move_sprite_y(&mut self, index: usize) {
        let velocity = self.sprite(index).velocity_y() as i8;
        if velocity == 0 {
            return;
        }
        let position = Y_POSITION.load(self, index);
        Y_POSITION.store(self, index, position.advanced(velocity));
    }

    pub fn move_sprite_z(&mut self, index: usize) {
        let position = Height::from_parts(
            u16::from(self.sprite(index).z()),
            self.sprite(index).subpixel_z(),
        );
        let velocity = self.sprite(index).velocity_z() as i8;
        HEIGHT.store(self, index, position.advanced(velocity));
    }

    #[must_use]
    pub fn sprites(&self) -> SpritePool<'_> {
        SpritePool {
            bytes: self.slice(0, WRAM_SIZE),
        }
    }

    #[must_use]
    pub fn sprites_mut(&mut self) -> SpritePoolMut<'_> {
        SpritePoolMut {
            bytes: self.bytes_mut(),
        }
    }

    /// Named access retaining the original index, including legacy sentinels.
    #[must_use]
    pub fn sprite(&self, index: usize) -> SpriteRef<'_> {
        self.sprites().at(index)
    }

    #[must_use]
    pub fn sprite_mut(&mut self, index: usize) -> SpriteMut<'_> {
        self.sprites_mut().at(index)
    }
}

// One reviewed location per field, matching the uint8 arrays in variables.h.
// Ram setters allow the existing read/modify/write expressions to retain
// their receiver/argument evaluation order via Rust's two-phase borrowing.
macro_rules! sprite_fields {
    ($($field:ident => $get:ident, $set:ident, $ram_set:ident => $base:ident = $address:literal;)*) => {
        /// A reviewed sprite byte field for table-driven operations.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum SpriteField { $($field,)* }

        impl SpriteField {
            fn address(self) -> usize {
                match self { $(Self::$field => $address,)* }
            }
        }

        pub(crate) mod layout {
            $(pub const $base: usize = $address;)*
        }

        impl SpriteRef<'_> {
            $(#[must_use]
            pub fn $get(self) -> u8 {
                self.bytes[layout::$base + self.index]
            })*
        }

        impl SpriteMut<'_> {
            $(#[must_use]
            pub fn $get(&self) -> u8 {
                self.as_read_only().$get()
            }

            pub fn $set(&mut self, value: u8) {
                self.bytes[layout::$base + self.index] = value;
            })*
        }

        impl Ram<'_> {
            $(pub fn $ram_set(&mut self, index: usize, value: u8) {
                self.sprite_mut(index).$set(value);
            })*
        }
    };
}

sprite_fields! {
    YLow => y_low, set_y_low, set_sprite_y_low => SPRITE_Y_LO = 0xd00;
    XLow => x_low, set_x_low, set_sprite_x_low => SPRITE_X_LO = 0xd10;
    YHigh => y_high, set_y_high, set_sprite_y_high => SPRITE_Y_HI = 0xd20;
    XHigh => x_high, set_x_high, set_sprite_x_high => SPRITE_X_HI = 0xd30;
    VelocityY => velocity_y, set_velocity_y, set_sprite_velocity_y => SPRITE_Y_VEL = 0xd40;
    VelocityX => velocity_x, set_velocity_x, set_sprite_velocity_x => SPRITE_X_VEL = 0xd50;
    SubpixelY => subpixel_y, set_subpixel_y, set_sprite_subpixel_y => SPRITE_Y_SUBPIXEL = 0xd60;
    SubpixelX => subpixel_x, set_subpixel_x, set_sprite_subpixel_x => SPRITE_X_SUBPIXEL = 0xd70;
    AiState => ai_state, set_ai_state, set_sprite_ai_state => SPRITE_AI_STATE = 0xd80;
    ScratchA => scratch_a, set_scratch_a, set_sprite_scratch_a => SPRITE_A = 0xd90;
    ScratchB => scratch_b, set_scratch_b, set_sprite_scratch_b => SPRITE_B = 0xda0;
    ScratchC => scratch_c, set_scratch_c, set_sprite_scratch_c => SPRITE_C = 0xdb0;
    Graphics => graphics, set_graphics, set_sprite_graphics => SPRITE_GRAPHICS = 0xdc0;
    State => state, set_state, set_sprite_state => SPRITE_STATE = 0xdd0;
    ScratchD => scratch_d, set_scratch_d, set_sprite_scratch_d => SPRITE_D = 0xde0;
    MainDelay => main_delay, set_main_delay, set_sprite_main_delay => SPRITE_DELAY_MAIN = 0xdf0;
    Aux1Delay => aux1_delay, set_aux1_delay, set_sprite_aux1_delay => SPRITE_DELAY_AUX1 = 0xe00;
    Aux2Delay => aux2_delay, set_aux2_delay, set_sprite_aux2_delay => SPRITE_DELAY_AUX2 = 0xe10;
    Kind => kind, set_kind, set_sprite_kind => SPRITE_TYPE = 0xe20;
    Subtype => subtype, set_subtype, set_sprite_subtype => SPRITE_SUBTYPE = 0xe30;
    Flags2 => flags2, set_flags2, set_sprite_flags2 => SPRITE_FLAGS2 = 0xe40;
    Health => health, set_health, set_sprite_health => SPRITE_HEALTH = 0xe50;
    Flags3 => flags3, set_flags3, set_sprite_flags3 => SPRITE_FLAGS3 = 0xe60;
    WallCollision => wall_collision, set_wall_collision, set_sprite_wall_collision => SPRITE_WALLCOLL = 0xe70;
    Subtype2 => subtype2, set_subtype2, set_sprite_subtype2 => SPRITE_SUBTYPE2 = 0xe80;
    ScratchE => scratch_e, set_scratch_e, set_sprite_scratch_e => SPRITE_E = 0xe90;
    ScratchF => scratch_f, set_scratch_f, set_sprite_scratch_f => SPRITE_F = 0xea0;
    HeadDirection => head_direction, set_head_direction, set_sprite_head_direction => SPRITE_HEAD_DIR = 0xeb0;
    AnimationClock => animation_clock, set_animation_clock, set_sprite_animation_clock => SPRITE_ANIM_CLOCK = 0xec0;
    ScratchG => scratch_g, set_scratch_g, set_sprite_scratch_g => SPRITE_G = 0xed0;
    Aux3Delay => aux3_delay, set_aux3_delay, set_sprite_aux3_delay => SPRITE_DELAY_AUX3 = 0xee0;
    HitTimer => hit_timer, set_hit_timer, set_sprite_hit_timer => SPRITE_HIT_TIMER = 0xef0;
    Pause => pause, set_pause, set_sprite_pause => SPRITE_PAUSE = 0xf00;
    Aux4Delay => aux4_delay, set_aux4_delay, set_sprite_aux4_delay => SPRITE_DELAY_AUX4 = 0xf10;
    Floor => floor, set_floor, set_sprite_floor => SPRITE_FLOOR = 0xf20;
    RecoilY => recoil_y, set_recoil_y, set_sprite_recoil_y => SPRITE_Y_RECOIL = 0xf30;
    RecoilX => recoil_x, set_recoil_x, set_sprite_recoil_x => SPRITE_X_RECOIL = 0xf40;
    OamFlags => oam_flags, set_oam_flags, set_sprite_oam_flags => SPRITE_OAM_FLAGS = 0xf50;
    Flags4 => flags4, set_flags4, set_sprite_flags4 => SPRITE_FLAGS4 = 0xf60;
    Z => z, set_z, set_sprite_z => SPRITE_Z = 0xf70;
    VelocityZ => velocity_z, set_velocity_z, set_sprite_velocity_z => SPRITE_Z_VEL = 0xf80;
    SubpixelZ => subpixel_z, set_subpixel_z, set_sprite_subpixel_z => SPRITE_Z_SUBPOS = 0xf90;
    Stunned => stunned, set_stunned, set_sprite_stunned => SPRITE_STUNNED = 0xb58;
    Flags => flags, set_flags, set_sprite_flags => SPRITE_FLAGS = 0xb6b;
    ObjectPriority => object_priority, set_object_priority, set_sprite_object_priority => SPRITE_OBJ_PRIO = 0xb89;
    IgnoreProjectile => ignore_projectile, set_ignore_projectile, set_sprite_ignore_projectile => SPRITE_IGNORE_PROJECTILE = 0xba0;
    Unknown2 => unknown2, set_unknown2, set_sprite_unknown2 => SPRITE_UNK2 = 0xbb0;
    ScratchN => scratch_n, set_scratch_n, set_sprite_scratch_n => SPRITE_N = 0xbc0;
    Flags5 => flags5, set_flags5, set_sprite_flags5 => SPRITE_FLAGS5 = 0xbe0;
    Room => room, set_room, set_sprite_room => SPRITE_ROOM = 0xc9a;
    DeflectionBits => deflection_bits, set_deflection_bits, set_sprite_deflection_bits => SPRITE_DEFL_BITS = 0xcaa;
    DeathAction => death_action, set_death_action, set_sprite_death_action => SPRITE_DIE_ACTION = 0xcba;
    BumpDamage => bump_damage, set_bump_damage, set_sprite_bump_damage => SPRITE_BUMP_DAMAGE = 0xcd2;
    Damage => damage, set_damage, set_sprite_damage => SPRITE_GIVE_DAMAGE = 0xce2;
    ScratchI => scratch_i, set_scratch_i, set_sprite_scratch_i => SPRITE_I = 0x1f9c2;
    Unknown3 => unknown3, set_unknown3, set_sprite_unknown3 => SPRITE_UNK3 = 0x1fa1c;
    Unknown4 => unknown4, set_unknown4, set_sprite_unknown4 => SPRITE_UNK4 = 0x1fa2c;
    Unknown5 => unknown5, set_unknown5, set_sprite_unknown5 => SPRITE_UNK5 = 0x1fa3c;
    Unknown1 => unknown1, set_unknown1, set_sprite_unknown1 => SPRITE_UNK1 = 0x1fa4c;
}

impl SpriteRef<'_> {
    #[must_use]
    pub fn field(self, field: SpriteField) -> u8 {
        self.bytes[field.address() + self.index]
    }

    #[must_use]
    pub fn signed_field(self, field: SpriteField) -> i8 {
        self.field(field) as i8
    }

    #[must_use]
    pub fn x(self) -> u16 {
        u16::from_le_bytes([self.x_low(), self.x_high()])
    }

    #[must_use]
    pub fn y(self) -> u16 {
        u16::from_le_bytes([self.y_low(), self.y_high()])
    }

    /// Word alias of scratch N: its stride is two, overlapping other slots.
    #[must_use]
    pub fn scratch_n_word(self) -> u16 {
        let address = layout::SPRITE_N + self.index * 2;
        u16::from_le_bytes([self.bytes[address], self.bytes[address + 1]])
    }
}

impl SpriteMut<'_> {
    pub fn set_field(&mut self, field: SpriteField, value: u8) {
        self.bytes[field.address() + self.index] = value;
    }

    pub fn set_signed_field(&mut self, field: SpriteField, value: i8) {
        self.set_field(field, value as u8);
    }

    #[must_use]
    pub fn x(&self) -> u16 {
        self.as_read_only().x()
    }

    #[must_use]
    pub fn y(&self) -> u16 {
        self.as_read_only().y()
    }

    pub fn set_x(&mut self, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.set_x_low(low);
        self.set_x_high(high);
    }

    pub fn set_y(&mut self, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.set_y_low(low);
        self.set_y_high(high);
    }

    pub fn set_scratch_n_word(&mut self, value: u16) {
        let address = layout::SPRITE_N + self.index * 2;
        self.bytes[address..address + 2].copy_from_slice(&value.to_le_bytes());
    }

    /// The four action countdowns tick in this order, leaving zero untouched.
    pub fn tick_action_delays(&mut self) {
        for field in [
            SpriteField::MainDelay,
            SpriteField::Aux1Delay,
            SpriteField::Aux2Delay,
            SpriteField::Aux3Delay,
        ] {
            let value = self.as_read_only().field(field);
            if value != 0 {
                self.set_field(field, value.wrapping_sub(1));
            }
        }
    }

    /// Clears only the forty transient fields reset by sprite preparation.
    /// Inactive coordinates, kind, room and all other bytes remain live.
    pub fn reset_transient(&mut self) {
        use SpriteField::*;
        for field in [
            Pause,
            ScratchE,
            VelocityX,
            VelocityY,
            VelocityZ,
            SubpixelX,
            SubpixelY,
            SubpixelZ,
            AiState,
            Graphics,
            ScratchD,
            MainDelay,
            Aux1Delay,
            Aux2Delay,
            Aux4Delay,
            HeadDirection,
            AnimationClock,
            ScratchG,
            HitTimer,
            WallCollision,
            Z,
            Health,
            ScratchF,
            RecoilX,
            RecoilY,
            ScratchA,
            ScratchB,
            ScratchC,
            Unknown2,
            Subtype2,
            IgnoreProjectile,
            ObjectPriority,
            OamFlags,
            Stunned,
            Damage,
            Unknown3,
            Unknown4,
            Unknown5,
            Unknown1,
            ScratchI,
        ] {
            self.set_field(field, 0);
        }
    }

    pub fn apply_initial_properties(&mut self, properties: SpriteInitialProperties) {
        use SpriteField::*;
        for (field, value) in [
            (Flags2, properties.flags2),
            (Health, properties.health),
            (Flags4, properties.flags4),
            (Flags5, properties.flags5),
            (DeflectionBits, properties.deflection_bits),
            (BumpDamage, properties.bump_damage),
            (Flags, properties.flags),
            (Flags3, properties.flags3),
            (OamFlags, properties.oam_flags),
        ] {
            self.set_field(field, value);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpriteInitialProperties {
    pub flags2: u8,
    pub health: u8,
    pub flags4: u8,
    pub flags5: u8,
    pub deflection_bits: u8,
    pub bump_damage: u8,
    pub flags: u8,
    pub flags3: u8,
    pub oam_flags: u8,
}

// Compatibility layout of the twenty-four transition-cache fields. This is
// not a complete entity snapshot: execution deliberately retains all effects
// on fields outside this list when the transient fields are restored.
const TRANSITION_FIELDS: [(SpriteField, usize); 24] = {
    use SpriteField::*;
    [
        (State, 0x1d00),
        (Kind, 0x1d10),
        (XLow, 0x1d20),
        (XHigh, 0x1d30),
        (YLow, 0x1d40),
        (YHigh, 0x1d50),
        (Graphics, 0x1d60),
        (ScratchA, 0x1d70),
        (HeadDirection, 0x1d80),
        (OamFlags, 0x1d90),
        (ObjectPriority, 0x1da0),
        (ScratchD, 0x1db0),
        (Flags2, 0x1dc0),
        (Floor, 0x1dd0),
        (AiState, 0x1de0),
        (Flags3, 0x1df0),
        (ScratchB, 0x1fa5c),
        (ScratchC, 0x1fa6c),
        (ScratchE, 0x1fa7c),
        (Subtype2, 0x1fa8c),
        (Z, 0x1fa9c),
        (MainDelay, 0x1faac),
        (ScratchI, 0x1facc),
        (IgnoreProjectile, 0x1fadc),
    ]
};

pub struct SpriteTransientSnapshot([u8; 24]);

impl Ram<'_> {
    pub fn cached_sprite_present(&self, index: usize) -> bool {
        self.read_u8(TRANSITION_FIELDS[0].1 + index) != 0
    }

    pub fn clear_cached_sprite(&mut self, index: usize) {
        self.write_u8(TRANSITION_FIELDS[0].1 + index, 0);
    }

    /// Saves the original fields before publishing cached fields into the live
    /// slot. Each write is immediate; this does not install a second authority.
    pub fn activate_cached_sprite(&mut self, index: usize) -> SpriteTransientSnapshot {
        let backup = TRANSITION_FIELDS.map(|(field, _)| self.sprite(index).field(field));
        for (field, cached) in TRANSITION_FIELDS {
            self.set_sprite_field(index, field, self.read_u8(cached + index));
        }
        SpriteTransientSnapshot(backup)
    }

    pub fn restore_sprite_transient(&mut self, index: usize, snapshot: SpriteTransientSnapshot) {
        for ((field, _), value) in TRANSITION_FIELDS.into_iter().zip(snapshot.0) {
            self.set_sprite_field(index, field, value);
        }
    }

    /// Captures one transition slot with the original pre-check/post-check
    /// write order. Paused and special states retain only the first six fields.
    pub fn cache_sprite_for_transition(&mut self, index: usize) {
        use SpriteField::*;
        self.clear_cached_sprite(index);
        for (field, cached) in [
            (Kind, 0x1d10),
            (XLow, 0x1d20),
            (Graphics, 0x1d60),
            (XHigh, 0x1d30),
            (YLow, 0x1d40),
            (YHigh, 0x1d50),
        ] {
            self.write_u8(cached + index, self.sprite(index).field(field));
        }
        if self.sprite(index).pause() != 0 || matches!(self.sprite(index).state(), 4 | 10) {
            return;
        }
        for &(field, cached) in TRANSITION_FIELDS
            .iter()
            .filter(|(field, _)| !matches!(field, Kind | XLow | XHigh | YLow | YHigh | Graphics))
        {
            self.write_u8(cached + index, self.sprite(index).field(field));
        }
    }

    pub fn set_sprite_field(&mut self, index: usize, field: SpriteField, value: u8) {
        self.sprite_mut(index).set_field(field, value);
    }

    pub fn set_sprite_signed_field(&mut self, index: usize, field: SpriteField, value: i8) {
        self.sprite_mut(index).set_signed_field(field, value);
    }

    pub fn set_sprite_x(&mut self, index: usize, value: u16) {
        self.sprite_mut(index).set_x(value);
    }

    pub fn set_sprite_y(&mut self, index: usize, value: u16) {
        self.sprite_mut(index).set_y(value);
    }

    pub fn set_sprite_scratch_n_word(&mut self, index: usize, value: u16) {
        self.sprite_mut(index).set_scratch_n_word(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_delay_tick_preserves_other_fields_and_slots() {
        for slot in [0, 15] {
            let mut bytes = [0xa5; WRAM_SIZE];
            let mut ram = Ram::new(&mut bytes);
            ram.set_sprite_main_delay(slot, 0);
            ram.set_sprite_aux1_delay(slot, 1);
            ram.set_sprite_aux2_delay(slot, 0xff);
            ram.set_sprite_aux3_delay(slot, 0x80);
            let mut expected = ram.slice(0, WRAM_SIZE).to_vec();
            expected[0xe00 + slot] = 0;
            expected[0xe10 + slot] = 0xfe;
            expected[0xee0 + slot] = 0x7f;
            ram.sprite_mut(slot).tick_action_delays();
            assert_eq!(ram.slice(0, WRAM_SIZE), expected);
        }
    }

    #[test]
    fn cache_activation_observes_raw_patches_and_restores_only_declared_fields() {
        let mut bytes = std::array::from_fn(|index| (index as u8).wrapping_mul(37));
        let mut ram = Ram::new(&mut bytes);
        let slot = 15;
        ram.set_sprite_state(slot, 9);
        ram.set_sprite_pause(slot, 0);
        ram.cache_sprite_for_transition(slot);
        // A replay patch to cached X is visible without any synchronization.
        ram.write_u8(0x1d20 + slot, 0xee);
        let mut expected = ram.slice(0, WRAM_SIZE).to_vec();
        let saved = ram.activate_cached_sprite(slot);
        assert_eq!(ram.sprite(slot).x_low(), 0xee);
        ram.set_sprite_x_low(slot, 0xa1);
        ram.set_sprite_aux4_delay(slot, 0x77);
        ram.set_sprite_unknown3(slot, 0x88);
        expected[0xf10 + slot] = 0x77;
        expected[0x1fa1c + slot] = 0x88;
        ram.restore_sprite_transient(slot, saved);
        assert_eq!(ram.slice(0, WRAM_SIZE), expected);
        assert!(ram.cached_sprite_present(slot));
        ram.clear_cached_sprite(slot);
        assert!(!ram.cached_sprite_present(slot));
    }

    #[test]
    fn slot_identity_reverse_search_and_inactive_bytes_are_preserved() {
        assert_eq!(SpriteSlot::new(15).unwrap().index(), 15);
        assert_eq!(SpriteSlot::new(16), None);
        assert_eq!(SpriteSlot::new(0xff), None);
        assert_eq!(
            SpriteSlot::all()
                .rev()
                .map(SpriteSlot::index)
                .collect::<Vec<_>>(),
            (0..16).rev().collect::<Vec<_>>()
        );
        let mut bytes = std::array::from_fn(|index| index as u8);
        let mut ram = Ram::new(&mut bytes);
        for slot in SpriteSlot::all() {
            ram.sprites_mut().get(slot).set_state(0xff);
        }
        assert_eq!(ram.sprites().find_inactive_rev(), None);
        ram.set_sprite_state(3, 0);
        ram.set_sprite_state(15, 0);
        let before = ram.slice(0, WRAM_SIZE).to_vec();
        assert_eq!(ram.sprites().find_inactive_rev(), SpriteSlot::new(15));
        // Searching and reading an inactive slot must not initialize or clear it.
        let slot = ram.sprites().get(SpriteSlot::new(15).unwrap());
        assert_eq!(slot.kind(), before[0xe20 + 15]);
        assert_eq!(slot.scratch_a(), before[0xd90 + 15]);
        assert_eq!(ram.slice(0, WRAM_SIZE), before);
        ram.set_sprite_state(15, 0x80);
        assert_eq!(ram.sprites().find_inactive_rev(), SpriteSlot::new(3));
    }

    #[test]
    fn raw_words_alias_adjacent_slots_and_the_next_array() {
        let mut bytes = [0; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        ram.set_sprite_scratch_n_word(7, 0xabcd);
        assert_eq!(ram.sprite(14).scratch_n(), 0xcd);
        assert_eq!(ram.sprite(15).scratch_n(), 0xab);
        assert_eq!(ram.sprite(7).scratch_n(), 0);
        assert_eq!(ram.sprite(7).scratch_n_word(), 0xabcd);
        ram.set_sprite_scratch_n(15, 0x12);
        assert_eq!(ram.sprite(7).scratch_n_word(), 0x12cd);
        ram.write_u16(0xd1f, 0x5678);
        assert_eq!(ram.sprite(15).x_low(), 0x78);
        assert_eq!(ram.sprite(0).y_high(), 0x56);
    }

    #[test]
    fn legacy_sentinel_access_keeps_the_original_address_instead_of_clamping() {
        let mut bytes = std::array::from_fn(|index| (index ^ (index >> 8)) as u8);
        let mut ram = Ram::new(&mut bytes);
        assert_eq!(ram.sprite(0xff).x_low(), ram.read_u8(0xd10 + 0xff));
        assert_eq!(ram.sprite(0xff).z(), ram.read_u8(0xf70 + 0xff));
        ram.set_sprite_floor(0xff, 0xfe);
        assert_eq!(ram.read_u8(0xf20 + 0xff), 0xfe);
        assert_ne!(ram.sprite(15).floor(), 0xfe);
    }

    #[test]
    fn named_slot_writes_match_the_whole_canonical_image() {
        let mut expected = std::array::from_fn(|index| (index.wrapping_mul(37)) as u8);
        let mut actual = expected;
        {
            let mut ram = Ram::new(&mut expected);
            for slot in [0, 7, 15, 0xff] {
                ram.write_u8(0xd10 + slot, 0xfe);
                ram.write_u8(0xd30 + slot, 0xff);
                ram.write_u8(0xd00 + slot, 0x34);
                ram.write_u8(0xd20 + slot, 0x12);
                ram.write_u8(0xd70 + slot, 0xff);
                ram.write_u8(0xd50 + slot, (-2i8) as u8);
                ram.write_u8(0xdd0 + slot, 0xff);
                ram.write_u8(0xe40 + slot, 0xa5);
                ram.write_u8(0xe50 + slot, 0x80);
                ram.write_u8(0xf50 + slot, 0x67);
                ram.write_u8(0x1f9c2 + slot, 0x98);
                ram.write_u16(0xbc0 + slot * 2, 0xabcd);
            }
        }
        {
            let mut ram = Ram::new(&mut actual);
            for slot in [0, 7, 15, 0xff] {
                ram.set_sprite_x(slot, 0xfffe);
                ram.set_sprite_y(slot, 0x1234);
                ram.set_sprite_subpixel_x(slot, 0xff);
                ram.set_sprite_velocity_x(slot, (-2i8) as u8);
                ram.set_sprite_state(slot, 0xff);
                ram.set_sprite_flags2(slot, 0xa5);
                ram.set_sprite_health(slot, 0x80);
                ram.set_sprite_oam_flags(slot, 0x67);
                ram.set_sprite_scratch_i(slot, 0x98);
                ram.set_sprite_scratch_n_word(slot, 0xabcd);
            }
        }
        assert_eq!(actual.as_slice(), expected.as_slice());
    }
}
