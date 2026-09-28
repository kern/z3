use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use z3_snes::bus::ApuBus;
use z3_snes::spc::Spc;

const RAM_SIZE: usize = 0x1_0000;

struct Memory(Box<[u8; RAM_SIZE]>);

impl ApuBus for Memory {
    fn read(&mut self, address: u16) -> u8 {
        self.0[usize::from(address)]
    }

    fn write(&mut self, address: u16, value: u8) {
        self.0[usize::from(address)] = value;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executable = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: check_spc SPC_ORACLE")?;
    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut input = BufWriter::new(child.stdin.take().ok_or("missing oracle stdin")?);
    let mut output = BufReader::new(child.stdout.take().ok_or("missing oracle stdout")?);

    for (opcode, scenario) in (0..=u8::MAX).flat_map(|opcode| (0..4_u8).map(move |n| (opcode, n))) {
        let mut ram = Box::new([0; RAM_SIZE]);
        let mut random = 0x9e37_79b9_u32 ^ u32::from(opcode) ^ (u32::from(scenario) << 24);
        for byte in ram.iter_mut() {
            random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            *byte = (random >> 24) as u8;
        }
        ram[0x8000] = opcode;

        let mut spc = Spc {
            a: opcode.wrapping_mul(3).wrapping_add(scenario),
            x: opcode.wrapping_mul(5).wrapping_add(scenario),
            y: opcode.wrapping_mul(7).wrapping_add(scenario),
            sp: opcode.wrapping_add(0x80).wrapping_add(scenario),
            pc: 0x8000,
            c: false,
            z: false,
            v: false,
            n: false,
            i: false,
            h: false,
            p: false,
            b: false,
            stopped: false,
            cycles_used: 0,
        };
        spc.set_flags(opcode.rotate_left(1) ^ scenario.wrapping_mul(0x55));

        let mut request = Vec::with_capacity(8 + RAM_SIZE);
        request.extend_from_slice(&[spc.a, spc.x, spc.y, spc.sp]);
        request.extend_from_slice(&spc.pc.to_le_bytes());
        request.push(spc.flags());
        request.push(u8::from(spc.stopped));
        request.extend_from_slice(ram.as_slice());
        input.write_all(&request)?;
        input.flush()?;

        let mut memory = Memory(ram);
        let cycles = spc.run_opcode(&mut memory);

        let mut reference = vec![0; 9 + RAM_SIZE];
        output.read_exact(&mut reference)?;
        let actual_header = [
            spc.a,
            spc.x,
            spc.y,
            spc.sp,
            spc.pc as u8,
            (spc.pc >> 8) as u8,
            spc.flags(),
            u8::from(spc.stopped),
            cycles,
        ];
        if reference[..9] != actual_header {
            return Err(format!(
                "SPC opcode {opcode:02x} scenario {scenario} register mismatch: C={:02x?}, Rust={actual_header:02x?}",
                &reference[..9]
            )
            .into());
        }
        if let Some((address, (c, rust))) = reference[9..]
            .iter()
            .zip(memory.0.iter())
            .enumerate()
            .find(|(_, (c, rust))| c != rust)
        {
            return Err(format!(
                "SPC opcode {opcode:02x} scenario {scenario} memory mismatch at {address:04x}: C={c:02x}, Rust={rust:02x}"
            )
            .into());
        }
    }

    drop(input);
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("SPC oracle exited with {status}").into());
    }
    println!("all 256 SPC opcodes match the compiled C core across 4 seeded states");
    Ok(())
}
