pub mod audio;
pub mod consts;
pub mod display;
pub mod io;
pub mod opcodes;
pub mod state;
pub mod tests;

use crate::state::{GBState, MemError};
use std::env;
use std::{thread, time};

pub fn exec_opcode(state: &mut GBState) -> Result<u64, MemError> {
    let opcode = state.mem.r(state.cpu.pc)?;

    state.cpu.pc += 1;

    let n1 = (opcode >> 3) & 0b111;
    let n2 = opcode & 0b111;

    match opcode >> 6 {
        0b00 => opcodes::op00(state, n1, n2),
        0b01 => opcodes::op01(state, n1, n2),
        0b10 => opcodes::op10(state, n1, n2),
        0b11 => opcodes::op11(state, n1, n2),
        _ => panic!(),
    }
}

fn main() {
    if env::args().len() != 2 {
        println!("Usage: gameboy-emulator <rom.gb>");
        return;
    }

    let rom = env::args().nth(1);

    println!("Starting {:?}...", rom.clone().unwrap());

    let mut state = GBState::new();
    let mut cycles = 0;

    state.mem.load_rom(&rom.unwrap()).unwrap();

    loop {
        let c = exec_opcode(&mut state).unwrap();

        // The OS scheduler is not precise enough to sleep at every iteration.
        // The workaround is to sleep every 1000 cycles and keep track of the
        // remaining cycles.
        if cycles >= 10000 {
            thread::sleep(time::Duration::from_nanos(
                cycles * consts::CPU_CYCLE_LENGTH_NANOS,
            ));
            cycles = 0;
        }
        cycles += c;

        state.mem.display.update_display(c);
    }
}
