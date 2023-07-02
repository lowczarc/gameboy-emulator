use crate::state::{GBState, MemError};

pub fn ldrr(state: &mut GBState, n1: u8, n2: u8) -> Result<(), MemError> {
    state.w_reg(n1, state.r_reg(n2)?)
}

pub fn ldr8(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    let p = state.r_mem(state.cpu.pc)?;
    state.cpu.pc += 1;

    state.w_reg(n1, p)
}

pub fn op00(state: &mut GBState, n1: u8, n2: u8) -> Result<(), MemError> {
    match n2 {
        0b110 => ldr8(state, n1),
        _ => panic!("Unimplemented"),
    }
}

pub fn op01(state: &mut GBState, n1: u8, n2: u8) -> Result<(), MemError> {
    if n1 == 0b110 && n2 == 0b110 {
        // TODO: Implement HALT
        panic!("Unimplemented");
    } else {
        ldrr(state, n1, n2)
    }
}
