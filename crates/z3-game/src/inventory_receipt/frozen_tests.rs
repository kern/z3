use crate::{
    misc::{self, MiscCall, MiscHooks},
    player::{Player, PlayerRuntime},
    ram::Ram,
};
use z3_core::WRAM_SIZE;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    Allocate(u8, u8, Vec<u8>),
    Call(MiscCall, Vec<u8>),
    Position(usize, u16, u16, Vec<u8>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Hooks {
    allocation: i32,
    replace_item: Option<u8>,
    mutate: bool,
    events: Vec<Event>,
}
impl Hooks {
    fn snapshot(ram: &Ram<'_>) -> Vec<u8> {
        ram.slice(0, WRAM_SIZE).to_vec()
    }
}
impl MiscHooks for Hooks {
    fn call(&mut self, ram: &mut Ram<'_>, call: MiscCall) {
        self.events.push(Event::Call(call, Self::snapshot(ram)));
        if self.mutate {
            ram.write_u8(0x2e9, ram.read_u8(0x2e9).wrapping_add(1) & 3);
            ram.write_u8(0x2d8, ram.read_u8(0x2d8) ^ 0xff);
            ram.write_u16(0x22, ram.read_u16(0x22).wrapping_add(17));
            ram.write_u16(0x20, ram.read_u16(0x20).wrapping_sub(9));
            ram.write_u8(0xf340, ram.read_u8(0xf340).wrapping_add(1));
        }
    }
    fn add_ancilla(&mut self, ram: &mut Ram<'_>, a: u8, y: u8) -> i32 {
        self.events.push(Event::Allocate(a, y, Self::snapshot(ram)));
        ram.write_u8(0x1000, 0x5a);
        if let Some(item) = self.replace_item {
            ram.write_u8(0x2d8, item);
        }
        self.allocation
    }
    fn set_ancilla_xy(&mut self, ram: &mut Ram<'_>, k: usize, x: u16, y: u16) {
        self.events
            .push(Event::Position(k, x, y, Self::snapshot(ram)));
    }
    fn route_main_module(&mut self, _: &mut Ram<'_>, _: u8) {
        unreachable!()
    }
    fn load_song_bank(&mut self, _: &mut Ram<'_>, _: u8) {
        unreachable!()
    }
    fn predefined_tile_word(&self, _: usize) -> u16 {
        unreachable!()
    }
    fn refill_magic(&mut self, _: &mut Ram<'_>) -> bool {
        unreachable!()
    }
    fn refill_health(&mut self, _: &mut Ram<'_>) -> bool {
        unreachable!()
    }
    fn sprite_x(&self, _: &Ram<'_>, _: usize) -> u16 {
        unreachable!()
    }
    fn dungeon_item_tile_action(&mut self, _: &mut Ram<'_>, _: u16, _: u16) -> u8 {
        unreachable!()
    }
    fn overworld_item_tile_action(&mut self, _: &mut Ram<'_>, _: u16, _: u16) -> u8 {
        unreachable!()
    }
}

fn image(item: u8, method: u8, seed: u8) -> Box<[u8; WRAM_SIZE]> {
    let mut bytes = Box::new([seed; WRAM_SIZE]);
    bytes[0x2d8] = item;
    bytes[0x2e9] = method;
    bytes[0x40c] = item & 31;
    bytes
}
fn compare(initial: Box<[u8; WRAM_SIZE]>, hooks: Hooks) {
    let mut old = initial.clone();
    let mut new = initial;
    let mut old_hooks = hooks.clone();
    let mut new_hooks = hooks;
    let a = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        misc::receipt_frozen::ancilla_add_item_receipt(
            &mut Ram::new(&mut old),
            &mut old_hooks,
            0x22,
            4,
            -1,
        )
    }));
    let b = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        misc::ancilla_add_item_receipt(&mut Ram::new(&mut new), &mut new_hooks, 0x22, 4, -1)
    }));
    assert_eq!(a.is_err(), b.is_err());
    assert_eq!(new, old);
    assert_eq!(new_hooks, old_hooks);
}

#[test]
fn frozen_receipts_preserve_all_ids_alloc_failure_partial_panics_and_live_hooks() {
    for item in 0..=255 {
        for allocation in [-1, 0, 9] {
            for mutate in [false, true] {
                compare(
                    image(item, item & 3, 0xa5),
                    Hooks {
                        allocation,
                        replace_item: None,
                        mutate,
                        events: Vec::new(),
                    },
                );
            }
        }
    }
    for item in 0..76 {
        for method in 0..4 {
            for seed in [0, 1, 99, 255] {
                for mutate in [false, true] {
                    compare(
                        image(item, method, seed),
                        Hooks {
                            allocation: 0,
                            replace_item: None,
                            mutate,
                            events: Vec::new(),
                        },
                    );
                }
            }
        }
    }
    for replacement in [Some(0), Some(0x25), Some(255)] {
        for allocation in [-1, 0] {
            compare(
                image(255, 1, 0),
                Hooks {
                    allocation,
                    replace_item: replacement,
                    mutate: true,
                    events: Vec::new(),
                },
            );
        }
    }
}

#[test]
fn undefined_c_palace_shifts_keep_frozen_rust_behavior_in_each_profile() {
    // C's promoted-int shift is undefined for these counts; preserve exactly
    // the prior Rust profile's panic/wrapping result and partial-write state.
    for item in [0x25, 0x32, 0x33] {
        for palace in 64..=255 {
            let mut bytes = image(item, 0, 0xa5);
            bytes[0x40c] = palace;
            compare(
                bytes,
                Hooks {
                    allocation: 0,
                    replace_item: None,
                    mutate: false,
                    events: Vec::new(),
                },
            );
        }
    }
}

#[test]
fn frozen_bottle_helper_keeps_all_raw_ids_and_slot_order() {
    let contents = [0, 1, 2, 3, 255];
    for item in 0..=255 {
        for code in 0..625 {
            let mut old = Box::new([0xa5; WRAM_SIZE]);
            let mut value = code;
            for slot in 0..4 {
                old[0xf35c + slot] = contents[value % 5];
                value /= 5;
            }
            let mut new = old.clone();
            misc::receipt_frozen::item_receipt_give_bottled_item(&mut Ram::new(&mut old), item);
            super::give_bottled_receipt(&mut Ram::new(&mut new), item);
            assert_eq!(new, old, "item{item} bottlepattern{code}");
        }
    }
}

#[derive(Default)]
struct Chest {
    item: u8,
    received: Option<u8>,
}
impl PlayerRuntime for Chest {
    fn open_chest(&mut self, ram: &mut Ram<'_>, _: u8) -> Option<(u8, i32)> {
        ram.write_u8(0x1000, 0x55);
        Some((self.item, 0))
    }
    fn ancilla_add_item_receipt(&mut self, ram: &mut Ram<'_>, _: u8, _: u8, _: i32) {
        self.received = Some(ram.read_u8(0x2d8));
    }
}

#[test]
fn chest_default_query_retains_legacy_ram_zero_semantics() {
    for (item, alternate, target) in [(12, 68, 0xf341), (18, 53, 0xf34a), (42, 70, 0xf341)] {
        for byte in 0..=255 {
            let mut bytes = Box::new([0; WRAM_SIZE]);
            bytes[0] = byte;
            bytes[target] = u8::from(byte == 0);
            let mut ram = Ram::new(&mut bytes);
            let mut runtime = Chest {
                item,
                ..Chest::default()
            };
            Player {
                ram: &mut ram,
                runtime: &mut runtime,
                applied_movement_to_camera: false,
            }
            .link_perform_open_chest();
            assert_eq!(
                runtime.received,
                Some(if byte == 0 { item } else { alternate })
            );
            assert_eq!(ram.read_u8(0x1000), 0x55);
        }
    }
}

struct CustomChest {
    chest: Chest,
    accepted: bool,
    queries: std::cell::RefCell<Vec<(u8, u8)>>,
}
impl PlayerRuntime for CustomChest {
    fn open_chest(&mut self, ram: &mut Ram<'_>, tile: u8) -> Option<(u8, i32)> {
        self.chest.open_chest(ram, tile)
    }
    fn receipt_item_present(&self, ram: &Ram<'_>, item: u8) -> bool {
        self.queries.borrow_mut().push((item, ram.read_u8(0x2e9)));
        self.accepted
    }
    fn ancilla_add_item_receipt(&mut self, ram: &mut Ram<'_>, a: u8, y: u8, pos: i32) {
        self.chest.ancilla_add_item_receipt(ram, a, y, pos);
    }
}

#[test]
fn semantic_chest_query_keeps_custom_callback_control_and_short_circuit_order() {
    for item in 0..76 {
        for accepted in [false, true] {
            let mut bytes = Box::new([0; WRAM_SIZE]);
            bytes[0] = u8::from(!accepted);
            let mut ram = Ram::new(&mut bytes);
            let mut runtime = CustomChest {
                chest: Chest {
                    item,
                    ..Chest::default()
                },
                accepted,
                queries: std::cell::RefCell::new(Vec::new()),
            };
            Player {
                ram: &mut ram,
                runtime: &mut runtime,
                applied_movement_to_camera: false,
            }
            .link_perform_open_chest();
            let alternate = match item {
                12 => Some(68),
                18 => Some(53),
                42 => Some(70),
                _ => None,
            };
            assert_eq!(
                runtime.chest.received,
                Some(if accepted {
                    alternate.unwrap_or(item)
                } else {
                    item
                })
            );
            assert_eq!(
                *runtime.queries.borrow(),
                if alternate.is_some() {
                    vec![(item, 1)]
                } else {
                    vec![]
                }
            );
        }
    }
}
