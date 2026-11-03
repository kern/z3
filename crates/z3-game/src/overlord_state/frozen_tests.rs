use super::{frozen_boss::FrozenBoss, *};
use crate::sprite_main_bank2a::{SpriteMainBank2a, SpriteMainBank2aRuntime};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Hooks {
    mutate: bool,
    events: Vec<(u8, Vec<u8>)>,
}
impl Hooks {
    fn event(&mut self, ram: &mut Ram<'_>, kind: u8) {
        self.events.push((kind, ram.slice(0, WRAM_SIZE).to_vec()));
        if self.mutate {
            // Real callbacks can modify the overlapping tail state. Reads
            // after them must remain live, not use precomputed typed values.
            ram.write_u16(0xb31, 0x4321);
            ram.write_u8(0xb2f, 0xf0);
            ram.write_u8(0xb2e, 0x81);
        }
    }
}
impl SpriteMainBank2aRuntime for Hooks {
    fn sfx3_with_pan(&mut self, ram: &mut Ram<'_>, _: usize, _: u8) {
        self.event(ram, 0);
    }
    fn is_right_of_link(&mut self, ram: &mut Ram<'_>, _: usize) -> (u8, i8) {
        self.event(ram, 1);
        (0, 0)
    }
    fn calculate_sfx_pan(&mut self, ram: &mut Ram<'_>, _: usize) -> u8 {
        self.event(ram, 2);
        0x40
    }
}
fn image() -> Box<[u8; WRAM_SIZE]> {
    Box::new(std::array::from_fn(|i| {
        (i.wrapping_mul(23) ^ (i >> 8)) as u8
    }))
}
fn compare(initial: Box<[u8; WRAM_SIZE]>, operation: u8, slot: usize, mutate: bool) {
    let mut old = initial.clone();
    let mut new = initial;
    let mut old_hooks = Hooks {
        mutate,
        ..Hooks::default()
    };
    let mut new_hooks = old_hooks.clone();
    {
        let mut ram = Ram::new(&mut old);
        let mut bank = FrozenBoss {
            ram: &mut ram,
            runtime: &mut old_hooks,
        };
        match operation {
            0 => bank.helmasaur_king_initialize(slot),
            1 => bank.helmasaur_king_reinitialize(slot),
            2 => bank.helmasaur_king_swing_tail(slot),
            3 => bank.trinexx_wag_tail(slot),
            _ => unreachable!(),
        }
    }
    {
        let mut ram = Ram::new(&mut new);
        let mut bank = SpriteMainBank2a {
            ram: &mut ram,
            runtime: &mut new_hooks,
        };
        match operation {
            0 => bank.helmasaur_king_initialize(slot),
            1 => bank.helmasaur_king_reinitialize(slot),
            2 => bank.helmasaur_king_swing_tail(slot),
            3 => bank.trinexx_wag_tail(slot),
            _ => unreachable!(),
        }
    }
    assert_eq!(new, old, "boss operation{operation} slot{slot}");
    assert_eq!(new_hooks, old_hooks);
}

#[test]
fn frozen_boss_initializers_and_tail_bytes_keep_the_complete_image() {
    for subtype in 0..=255 {
        for slot in [0, 7, 15] {
            for operation in [0, 1] {
                let mut bytes = image();
                Ram::new(&mut bytes).set_sprite_subtype2(slot, subtype);
                compare(bytes, operation, slot, false);
            }
        }
    }
    for clock in 0..=255 {
        for direction in [0, 1] {
            for bend in [0, 1, 5, 6, 255] {
                for delay in [0, 1, 8] {
                    let mut bytes = image();
                    bytes[0xb0a] = bend;
                    bytes[0xb0b] = direction;
                    bytes[0xb0c] = clock;
                    bytes[0xb0d] = delay;
                    compare(bytes, 3, 0, false);
                }
            }
        }
    }
}

#[test]
fn frozen_helmasaur_swing_keeps_odd_words_and_live_callbacks() {
    for phase in [0, 0xff, 0x100, 0x7fff, 0x8000, 0xffff] {
        for clock in 0..=4 {
            for state in [0, 1, 15, 16, 239, 240, 255] {
                for mutate in [false, true] {
                    let mut bytes = image();
                    let mut ram = Ram::new(&mut bytes);
                    ram.write_u16(0xb2d, phase);
                    ram.write_u16(0xb31, phase ^ 0xa55a);
                    ram.write_u8(0xb30, state);
                    ram.write_u8(0x1a, 0);
                    ram.set_sprite_animation_clock(0, clock);
                    ram.set_sprite_head_direction(0, (state & 1) ^ 1);
                    ram.set_sprite_scratch_d(0, 0);
                    ram.set_sprite_aux3_delay(0, 0);
                    compare(bytes, 2, 0, mutate);
                }
            }
        }
    }
}
