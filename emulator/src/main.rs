pub mod consts;
pub mod io;
pub mod opcodes;
pub mod state;
pub mod tests;

use crate::state::{GBState, MemError};

pub fn exec_opcode(state: &mut GBState, counter: u128) -> Result<(), MemError> {
    let opcode = state.mem.r(state.cpu.pc)?;
    state.cpu.pc += 1;

    let n1 = (opcode >> 3) & 0b111;
    let n2 = opcode & 0b111;

    match opcode >> 6 {
        0b00 => opcodes::op00(state, n1, n2)?,
        0b01 => opcodes::op01(state, n1, n2)?,
        0b10 => opcodes::op10(state, n1, n2)?, // Arithmetic
        0b11 => opcodes::op11(state, n1, n2)?,
        _ => panic!(),
    }

    Ok(())
}

fn main() {
    let mut state = GBState::new();
    let mut counter = 0u128;

    loop {
        exec_opcode(&mut state, counter).unwrap();
        counter += 1;
    }
}
