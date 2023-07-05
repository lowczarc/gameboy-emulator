pub mod consts;
pub mod opcodes;
pub mod state;

use crate::state::{GBState, MemError};

pub fn exec_opcode(state: &mut GBState) -> Result<(), MemError> {
    let opcode = state.mem.r(state.cpu.pc)?;
    state.cpu.pc += 1;

    let n1 = (opcode >> 3) & 0b111;
    let n2 = opcode & 0b111;

    match opcode >> 6 {
        0b00 => opcodes::op00(state, n1, n2)?,
        0b01 => opcodes::op01(state, n1, n2)?,
        0b10 => todo!(), // Arithmetic
        0b11 => todo!(),
        _ => panic!(),
    }

    println!("DEBUG: {:?}", state.cpu);
    Ok(())
}

fn main() {
    let mut state = GBState::new();

    exec_opcode(&mut state).unwrap();
    exec_opcode(&mut state).unwrap();
    exec_opcode(&mut state).unwrap();
}
