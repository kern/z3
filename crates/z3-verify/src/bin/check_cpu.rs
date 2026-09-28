use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use z3_snes::bus::Bus;
use z3_snes::cpu::Cpu;

const MAX_WRITES: usize = 32;
const RESPONSE_SIZE: usize = 22 + MAX_WRITES * 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MemoryWrite {
    address: u32,
    value: u8,
}

struct SparseBus {
    seed: u32,
    opcode_address: u32,
    opcode: u8,
    writes: Vec<MemoryWrite>,
}

impl SparseBus {
    fn seeded_byte(address: u32, seed: u32) -> u8 {
        let mut value = address ^ seed;
        value ^= value >> 16;
        value = value.wrapping_mul(0x7feb_352d);
        value ^= value >> 15;
        value = value.wrapping_mul(0x846c_a68b);
        value ^= value >> 16;
        (value >> 24) as u8
    }
}

impl Bus for SparseBus {
    fn open_bus(&self) -> u8 {
        0
    }

    fn read(&mut self, address: u32) -> u8 {
        let address = address & 0xff_ffff;
        self.writes
            .iter()
            .rev()
            .find(|write| write.address == address)
            .map_or_else(
                || {
                    if address == self.opcode_address {
                        self.opcode
                    } else {
                        Self::seeded_byte(address, self.seed)
                    }
                },
                |write| write.value,
            )
    }

    fn write(&mut self, address: u32, value: u8) {
        assert!(self.writes.len() < MAX_WRITES);
        self.writes.push(MemoryWrite {
            address: address & 0xff_ffff,
            value,
        });
    }

    fn read_b_bus(&mut self, address: u8) -> u8 {
        self.read(0x2100 | u32::from(address))
    }

    fn write_b_bus(&mut self, address: u8, value: u8) {
        self.write(0x2100 | u32::from(address), value);
    }
}

fn put_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executable = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: check_cpu CPU_ORACLE")?;
    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut input = BufWriter::new(child.stdin.take().ok_or("missing oracle stdin")?);
    let mut output = BufReader::new(child.stdout.take().ok_or("missing oracle stdout")?);

    for (opcode, scenario) in (0..=u8::MAX).flat_map(|opcode| (0..4_u8).map(move |n| (opcode, n))) {
        let seed = 0x9e37_79b9 ^ u32::from(opcode) ^ (u32::from(scenario) << 24);
        let mut cpu = Cpu::new();
        cpu.a = u16::from(opcode)
            .wrapping_mul(257)
            .wrapping_add(u16::from(scenario));
        cpu.x = u16::from(opcode)
            .wrapping_mul(131)
            .wrapping_add(u16::from(scenario));
        cpu.y = u16::from(opcode)
            .wrapping_mul(67)
            .wrapping_add(u16::from(scenario));
        cpu.sp = 0x1f0 + u16::from(scenario);
        cpu.dp = u16::from(scenario) * 0x101;
        cpu.k = 0;
        cpu.pc = 0x8000;
        cpu.db = scenario.wrapping_mul(0x55);
        cpu.e = scenario & 1 != 0;
        cpu.set_flags(opcode ^ scenario.wrapping_mul(0x55));
        if opcode == 0 {
            cpu.k = 7;
            cpu.pc = 0xb269;
        }
        let opcode_address = u32::from(cpu.k) << 16 | u32::from(cpu.pc);
        let mut bus = SparseBus {
            seed,
            opcode_address,
            opcode,
            writes: Vec::new(),
        };

        input.write_all(&[opcode, scenario])?;
        input.flush()?;
        let cycles = cpu.run_opcode(&mut bus);

        let mut actual = Vec::with_capacity(RESPONSE_SIZE);
        for value in [cpu.a, cpu.x, cpu.y, cpu.sp, cpu.pc, cpu.dp] {
            put_u16(&mut actual, value);
        }
        actual.extend_from_slice(&[
            cpu.k,
            cpu.db,
            cpu.flags(),
            u8::from(cpu.e),
            u8::from(cpu.irq_wanted),
            u8::from(cpu.nmi_wanted),
            u8::from(cpu.waiting),
            u8::from(cpu.stopped),
            cycles,
            bus.writes.len() as u8,
        ]);
        for index in 0..MAX_WRITES {
            let write = bus.writes.get(index).copied().unwrap_or(MemoryWrite {
                address: 0,
                value: 0,
            });
            actual.extend_from_slice(&write.address.to_le_bytes());
            actual.push(write.value);
        }

        let mut reference = vec![0; RESPONSE_SIZE];
        output.read_exact(&mut reference)?;
        if reference != actual {
            let offset = reference
                .iter()
                .zip(&actual)
                .position(|(c, rust)| c != rust)
                .unwrap();
            return Err(format!(
                "CPU opcode {opcode:02x} scenario {scenario} mismatch at response byte {offset}: C={:02x}, Rust={:02x}",
                reference[offset], actual[offset]
            )
            .into());
        }
    }

    drop(input);
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("CPU oracle exited with {status}").into());
    }
    println!("all 256 CPU opcodes match the compiled C core across 4 seeded states");
    Ok(())
}
