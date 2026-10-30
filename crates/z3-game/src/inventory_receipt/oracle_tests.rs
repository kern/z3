use crate::{assets::RuntimeAssets, integration::NativeGameServices, misc, ram::Ram};
use std::{
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};
use z3_core::WRAM_SIZE;

struct Oracle {
    child: Child,
    input: BufWriter<ChildStdin>,
    output: BufReader<ChildStdout>,
}
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
impl Oracle {
    fn new() -> Self {
        let mut child = Command::new(root().join("build/c-oracle/inventory_receipt_oracle"))
            .current_dir(root())
            .arg(root().join("zelda3_assets.dat"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("build inventory receipt C oracle first");
        Self {
            input: BufWriter::new(child.stdin.take().unwrap()),
            output: BufReader::new(child.stdout.take().unwrap()),
            child,
        }
    }
    fn run(&mut self, operation: u8, item: u8, bytes: &[u8; WRAM_SIZE]) -> Box<[u8; WRAM_SIZE]> {
        self.input.write_all(&[operation, item]).unwrap();
        self.input.write_all(bytes).unwrap();
        self.input.flush().unwrap();
        let mut result = Box::new([0; WRAM_SIZE]);
        self.output.read_exact(result.as_mut_slice()).unwrap();
        result
    }
    fn finish(mut self) {
        drop(self.input);
        assert!(self.child.wait().unwrap().success());
    }
}
fn receipt_image(item: u8, palace: u8) -> Box<[u8; WRAM_SIZE]> {
    let mut bytes = Box::new([0; WRAM_SIZE]);
    bytes[0x2d8] = item;
    bytes[0x40c] = palace;
    bytes
}
fn run_native(bytes: &mut [u8; WRAM_SIZE], assets: &RuntimeAssets<'_>) {
    let mut services = NativeGameServices::new(assets);
    misc::ancilla_add_item_receipt(&mut Ram::new(bytes), &mut services, 0x22, 4, 0);
}

#[test]
#[ignore = "requires original C inventory_receipt_oracle and local assets"]
fn receipt_palace_mask_matches_original_c_promoted_shift() {
    let asset_bytes = std::fs::read(root().join("zelda3_assets.dat")).unwrap();
    let assets = RuntimeAssets::parse(&asset_bytes).unwrap();
    let mut oracle = Oracle::new();
    let mut references = Vec::new();
    for palace in 0..64 {
        let reference = oracle.run(0, 0x25, &receipt_image(0x25, palace));
        let mask = (0x8000u32 >> (palace >> 1)) as u16;
        assert_eq!(
            &reference[0xf364..0xf366],
            &mask.to_le_bytes(),
            "C palace{palace}"
        );
        references.push(reference);
    }
    eprintln!(
        "unchanged C validated all64 palace bytes0..63, including promoted shifts16..31 ->zero"
    );
    for (palace, reference) in references.iter().enumerate() {
        let mut rust = receipt_image(0x25, palace as u8);
        // RED before correction: current u16 shift panics at palace32 even
        // though the unchanged C promoted-int shift is defined and returns0.
        run_native(rust.as_mut(), &assets);
        assert_eq!(
            &rust[0xf364..0xf366],
            &reference[0xf364..0xf366],
            "palace{palace}"
        );
    }
    oracle.finish();
}

fn assert_whole_wram(
    actual: &[u8; WRAM_SIZE],
    expected: &[u8; WRAM_SIZE],
    context: impl std::fmt::Debug,
) {
    if let Some(address) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "inventory C mismatch {context:?} WRAM{address:#x}: Rust{:02x} C{:02x}",
            actual[address], expected[address]
        );
    }
}

#[test]
#[ignore = "requires original C inventory_receipt_oracle and local assets"]
fn inventory_receipts_match_original_c_whole_wram() {
    let asset_bytes = std::fs::read(root().join("zelda3_assets.dat")).unwrap();
    let assets = RuntimeAssets::parse(&asset_bytes).unwrap();
    let mut oracle = Oracle::new();
    let mut cases = 0usize;
    for item in 0..76 {
        for method in 0..4 {
            for initial in [0, 1, 2] {
                let mut rust = receipt_image(item, 12);
                rust[0x2e9] = method;
                let mut ram = Ram::new(&mut rust);
                ram.write_u16(0x22, 0x120);
                ram.write_u16(0x20, 0x220);
                for address in [0xf364, 0xf366, 0xf368] {
                    ram.write_u16(address, 0xa55a);
                }
                ram.set_inventory_byte(super::ReceiptItem::from_raw(item).spec().target, initial);
                let reference = oracle.run(0, item, &rust);
                run_native(rust.as_mut(), &assets);
                assert_whole_wram(&rust, &reference, ("receipt", item, method, initial));
                cases += 1;
            }
        }
    }
    // Defined promoted-int mask domain, with unknown neighbor bits retained.
    for item in [0x25, 0x32, 0x33] {
        for palace in 0..64 {
            let mut rust = receipt_image(item, palace);
            let mut ram = Ram::new(&mut rust);
            ram.write_u16(
                super::ReceiptItem::from_raw(item).spec().target.address(),
                0x5aa5,
            );
            let reference = oracle.run(0, item, &rust);
            run_native(rust.as_mut(), &assets);
            assert_whole_wram(&rust, &reference, ("dungeon mask", item, palace));
            cases += 1;
        }
    }
    for item in 0..76 {
        let mut rust = receipt_image(item, 0);
        rust[0xc4a..0xc4f].fill(1);
        let reference = oracle.run(0, item, &rust);
        run_native(rust.as_mut(), &assets);
        assert_whole_wram(&rust, &reference, ("allocation failure", item));
        cases += 1;
    }
    // Public bottle helper includes rawff and unknown IDs, not just the76
    // ordinary receipt entries. All five-valued four-slot patterns for known
    // bottle/potion IDs; one asymmetric pattern for every other raw byte.
    let known = [
        0x16, 0x2b, 0x2c, 0x2d, 0x3d, 0x3c, 0x48, 0x2e, 0x2f, 0x30, 0xff, 0x0e,
    ];
    let values = [0, 1, 2, 3, 255];
    for item in 0..=255 {
        let count = if known.contains(&item) { 625 } else { 1 };
        for pattern in 0..count {
            let mut rust = Box::new([0xa5; WRAM_SIZE]);
            let mut code = if count == 1 { 117 } else { pattern };
            for slot in 0..4 {
                rust[0xf35c + slot] = values[code % 5];
                code /= 5;
            }
            let reference = oracle.run(1, item, &rust);
            super::give_bottled_receipt(&mut Ram::new(&mut rust), item);
            assert_whole_wram(&rust, &reference, ("bottle", item, pattern));
            cases += 1;
        }
    }
    eprintln!(
        "original C inventory receipts: {cases} whole-WRAM cases, {} compared bytes",
        cases * WRAM_SIZE
    );
    oracle.finish();
}
