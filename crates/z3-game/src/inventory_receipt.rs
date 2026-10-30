//! Typed receipt identities and live, lossless inventory targets.
//! Addresses and ordered grant data stay at this canonical-WRAM boundary.
use crate::ram::Ram;

macro_rules! target_fields {
    ($($name:ident => $address:literal;)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum InventoryTarget { $($name,)* }
        impl InventoryTarget {
            fn address(self) -> usize { match self { $(Self::$name => $address,)* } }
        }
    };
}

/// Unknown raw IDs remain representable; lookup panics only at spec(), AFTER
/// allocation and immobilization stores at the existing runtime boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReceiptItem(u8);
impl ReceiptItem {
    pub const fn from_raw(raw: u8) -> Self {
        Self(raw)
    }
    pub const fn raw(self) -> u8 {
        self.0
    }
    pub fn spec(self) -> &'static ReceiptSpec {
        &RECEIPTS[usize::from(self.0)]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReceiptSpec {
    pub target: InventoryTarget,
    pub initial_value: i8,
}

include!("inventory_receipt/target_schema.rs");

impl InventoryTarget {
    /// Ordinary receipt writes use ONE byte even when a field has word aliases.
    pub fn byte(self, ram: &Ram<'_>) -> u8 {
        ram.read_u8(self.address())
    }
    pub fn set_byte(self, ram: &mut Ram<'_>, value: u8) {
        ram.write_u8(self.address(), value);
    }
    /// Caller evaluates the unchanged palace shift BEFORE entering this method.
    pub fn insert_dungeon_bits(self, ram: &mut Ram<'_>, bits: u16) {
        ram.write_u16(self.address(), ram.read_u16(self.address()) | bits);
    }
}

// Receiver-based conveniences retain two-phase borrowing for the original
// read/modify/write expression; a free mutable-RAM argument would borrow too
// early for target.set_byte(ram, target.byte(ram).wrapping_add(1)).
impl Ram<'_> {
    pub fn inventory_byte(&self, target: InventoryTarget) -> u8 {
        target.byte(self)
    }
    pub fn set_inventory_byte(&mut self, target: InventoryTarget, value: u8) {
        target.set_byte(self, value);
    }
    pub fn insert_inventory_dungeon_bits(&mut self, target: InventoryTarget, bits: u16) {
        target.insert_dungeon_bits(self, bits);
    }
}

const BOTTLE_ITEMS: [u8; 7] = [0x16, 0x2b, 0x2c, 0x2d, 0x3d, 0x3c, 0x48];
const POTION_ITEMS: [u8; 5] = [0x2e, 0x2f, 0x30, 0xff, 0xe];

#[derive(Clone, Copy)]
struct BottleSlot(u8);
impl BottleSlot {
    fn all() -> impl Iterator<Item = Self> {
        (0..4).map(Self)
    }
    fn contents(self, ram: &Ram<'_>) -> u8 {
        ram.read_u8(0xf35c + usize::from(self.0))
    }
    fn set_contents(self, ram: &mut Ram<'_>, value: u8) {
        ram.write_u8(0xf35c + usize::from(self.0), value);
    }
}

/// Original forward first-match slot writes. Raw itemff is meaningful here:
/// the potion list maps it to value6; do not reject it as an invalid receipt.
pub fn give_bottled_receipt(ram: &mut Ram<'_>, raw_item: u8) {
    if let Some(index) = BOTTLE_ITEMS.iter().position(|&item| item == raw_item) {
        for slot in BottleSlot::all() {
            if slot.contents(ram) < 2 {
                slot.set_contents(ram, index as u8 + 2);
                return;
            }
        }
    }
    if let Some(index) = POTION_ITEMS.iter().position(|&item| item == raw_item) {
        for slot in BottleSlot::all() {
            if slot.contents(ram) == 2 {
                slot.set_contents(ram, index as u8 + 3);
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3_core::WRAM_SIZE;
    #[test]
    fn byte_target_and_word_mask_share_live_bytes_and_keep_unknown_bits() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        InventoryTarget::Compass.set_byte(&mut ram, 0x12);
        assert_eq!(ram.read_u16(0xf364), 0xa512);
        InventoryTarget::Compass.insert_dungeon_bits(&mut ram, 0x4000);
        assert_eq!(ram.read_u16(0xf364), 0xe512);
        assert_eq!(InventoryTarget::Compass.byte(&ram), 0x12);
    }
    #[test]
    fn first_bottle_target_aliases_slot_zero_and_ff_is_not_rejected() {
        let mut bytes = [0xa5; WRAM_SIZE];
        let mut ram = Ram::new(&mut bytes);
        InventoryTarget::FirstBottle.set_byte(&mut ram, 2);
        give_bottled_receipt(&mut ram, 0xff);
        assert_eq!(InventoryTarget::FirstBottle.byte(&ram), 6);
        assert_eq!(ram.read_u8(0xf35d), 0xa5);
    }
    #[test]
    fn duplicate_targets_keep_ordered_rows_and_signed_initial_values() {
        assert_eq!(
            ReceiptItem::from_raw(0x34).spec().target,
            InventoryTarget::RupeesGoal
        );
        assert_eq!(
            ReceiptItem::from_raw(0x35).spec().target,
            InventoryTarget::RupeesGoal
        );
        assert_eq!(ReceiptItem::from_raw(0x35).spec().initial_value, -5);
        assert_eq!(ReceiptItem::from_raw(0x40).spec().initial_value, -100);
    }
}

#[cfg(test)]
mod frozen_tests;
#[cfg(test)]
mod oracle_tests;
