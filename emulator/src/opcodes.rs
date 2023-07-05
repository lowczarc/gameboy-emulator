use crate::state::{reg, GBState, MemError};

pub fn ldrr(state: &mut GBState, n1: u8, n2: u8) -> Result<(), MemError> {
    // Load a register into another register
    // LD r, r
    state.w_reg(n1, state.r_reg(n2)?)
}

pub fn ldr8(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    // Load an raw 8b value into a register
    let p = state.mem.r(state.cpu.pc)?;
    state.cpu.pc += 1;

    state.w_reg(n1, p)
}

pub fn ldrr16(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    // Load a raw 16b value into a register
    let p: u16 = state.mem.r(state.cpu.pc)? as u16 | ((state.mem.r(state.cpu.pc + 1)? as u16) << 8);
    state.cpu.pc += 2;

    state.cpu.w16(n1 >> 1, p);
    Ok(())
}

pub fn ldnnsp(state: &mut GBState) -> Result<(), MemError> {
    // Load SP into an arbitrary position in memory
    let p: u16 = state.mem.r(state.cpu.pc)? as u16 | ((state.mem.r(state.cpu.pc + 1)? as u16) << 8);
    state.cpu.pc += 2;

    state.mem.w(p, (state.cpu.sp & 0xff) as u8)?;
    state.mem.w(p + 1, (state.cpu.sp >> 8) as u8)
}

pub fn jr8(state: &mut GBState) -> Result<(), MemError> {
    // Unconditional relative jump
    let p = state.mem.r(state.cpu.pc)?;
    state.cpu.pc += 1;
    state.cpu.pc = (state.cpu.pc as i16 + p as i8 as i16) as u16;

    Ok(())
}

pub fn jrcc8(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    // Conditional relative jump
    let p = state.mem.r(state.cpu.pc)?;
    state.cpu.pc += 1;

    if state.cpu.check_flag(n1 & 0b11) {
        state.cpu.pc = (state.cpu.pc as i16 + p as i8 as i16) as u16;
    }

    Ok(())
}

pub fn ld00a(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    // Load register A into or from memory pointed by rr (BC, DE or HL(+/-))
    // LD (rr), A
    // LD A, (rr)
    let ptr_reg = match n1 & 0b110 {
        0b000 => reg::B,
        0b010 => reg::C,
        _ => reg::HL,
    };

    if n1 & 0b001 == 1 {
        state.cpu.r[reg::A as usize] = state.mem.r(state.cpu.r16(ptr_reg))?;
    } else {
        state
            .mem
            .w(state.cpu.r16(ptr_reg), state.cpu.r[reg::A as usize])?;
    }

    if n1 & 0b110 == 0b100 {
        state.cpu.w16(reg::HL, state.cpu.r16(reg::HL) + 1); // (HL+)
    }

    if n1 & 0b110 == 0b110 {
        state.cpu.w16(reg::HL, state.cpu.r16(reg::HL) - 1); // (HL-)
    }

    Ok(())
}

pub fn inc8(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    // Increment 8 bit register
    state.w_reg(n1, state.r_reg(n1)? + 1)
}

pub fn dec8(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    // Decrement 8 bit register
    state.w_reg(n1, state.r_reg(n1)? + 1)
}

pub fn ccf(state: &mut GBState) {
    // Flip carry flag
    state.cpu.r[reg::F as usize] = (state.cpu.r[reg::F as usize] & 0b10011111) ^ 0b00010000
}

pub fn scf(state: &mut GBState) {
    // Set carry flag
    state.cpu.r[reg::F as usize] = (state.cpu.r[reg::F as usize] & 0b10011111) | 0b00010000
}

pub fn cpl(state: &mut GBState) {
    // Flip all bits in register A
    state.cpu.r[reg::F as usize] = state.cpu.r[reg::F as usize] & 0b10011111;
    state.cpu.r[reg::A as usize] ^= 0xff;
}

pub fn op00(state: &mut GBState, n1: u8, n2: u8) -> Result<(), MemError> {
    // Dispatcher for the instructions starting with 0b00 based on their 3 LSB
    match n2 {
        0b000 => match n1 {
            0b000 => Ok(()),
            0b001 => ldnnsp(state),
            0b010 => todo!(), // STOP
            0b011 => jr8(state),
            _ => jrcc8(state, n1),
        },
        0b001 => ldrr16(state, n1),
        0b010 => ld00a(state, n1),
        0b011 => todo!(), // 16b INC (?)
        0b100 => inc8(state, n1),
        0b101 => dec8(state, n1),
        0b110 => ldr8(state, n1),
        0b111 => match n1 {
            0b100 => todo!(), // DAA
            0b101 => Ok(cpl(state)),
            0b110 => Ok(scf(state)),
            0b111 => Ok(ccf(state)),
            _ => todo!(), // RLCA, RLA, RRCA, RRA
        },
        _ => panic!(),
    }
}

pub fn op01(state: &mut GBState, n1: u8, n2: u8) -> Result<(), MemError> {
    // Dispatcher for the instructions starting with 0b01 (LD r,r and HALT)
    if n1 == 0b110 && n2 == 0b110 {
        todo!() // HALT
    } else {
        ldrr(state, n1, n2)
    }
}
