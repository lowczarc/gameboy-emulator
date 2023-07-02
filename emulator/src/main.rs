pub mod consts;
pub mod opcodes;
pub mod state;

use crate::state::{GBState, MemError};

pub fn exec_opcode(state: &mut GBState) -> Result<(), MemError> {
    let opcode = state.r_mem(state.cpu.pc)?;
    state.cpu.pc += 1;

    let n1 = (opcode >> 3) & 0b111;
    let n2 = opcode & 0b111;

    match opcode >> 6 {
        0b00 => opcodes::op00(state, n1, n2)?,
        0b01 => opcodes::op01(state, n1, n2)?,
        _ => panic!("Unimplemented"),
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
