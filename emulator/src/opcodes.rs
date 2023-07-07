use crate::state::{flag, reg, GBState, MemError};

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
    state.w_reg(n1, state.r_reg(n1)? + 1)?;
    state.cpu.r[reg::F as usize] &= !flag::N;

    if state.r_reg(n1)? == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }

    if state.r_reg(n1)? & 0xf == 0x0 {
        state.cpu.r[reg::F as usize] |= flag::H;
    }

    Ok(())
}

pub fn dec8(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    // Decrement 8 bit register
    state.w_reg(n1, state.r_reg(n1)? - 1)?;
    state.cpu.r[reg::F as usize] |= flag::N;

    if state.r_reg(n1)? == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }

    if state.r_reg(n1)? & 0xf == 0xf {
        state.cpu.r[reg::F as usize] |= flag::H;
    }

    Ok(())
}

pub fn ccf(state: &mut GBState) {
    // Flip carry flag
    state.cpu.r[reg::F as usize] = (state.cpu.r[reg::F as usize] & 0b10011111) ^ 0b00010000
}

pub fn scf(state: &mut GBState) {
    // Set carry flag
    state.cpu.r[reg::F as usize] = (state.cpu.r[reg::F as usize] & 0b10011111) | 0b00010000
}

pub fn daa(state: &mut GBState) {
    // Decimal Adjust Accumulator
    // Adjust the A register after a addition or substraction to keep valid BCD representation
    let nibble_low = state.cpu.r[reg::A as usize] & 0b1111;
    let nibble_high = state.cpu.r[reg::A as usize] >> 4;
    let sub_flag = (state.cpu.r[reg::F as usize] & flag::N) != 0;

    if nibble_low > 9 && !sub_flag {
        state.cpu.r[reg::A as usize] += 0x06;
    }
    if nibble_low > 9 && sub_flag {
        state.cpu.r[reg::A as usize] -= 0x06;
    }

    if nibble_high > 9 && !sub_flag {
        state.cpu.r[reg::A as usize] += 0x60;
        state.cpu.r[reg::F as usize] += flag::CY;
    }
    if nibble_high > 9 && sub_flag {
        state.cpu.r[reg::A as usize] -= 0x60;
        state.cpu.r[reg::F as usize] |= flag::CY;
    }

    if state.cpu.r[reg::A as usize] == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }

    state.cpu.r[reg::F as usize] &= !flag::H;
}

pub fn cpl(state: &mut GBState) {
    // Flip all bits in register A
    state.cpu.r[reg::F as usize] = state.cpu.r[reg::F as usize] & 0b10011111;
    state.cpu.r[reg::A as usize] ^= 0xff;
}

pub fn addr(state: &mut GBState, n2: u8) -> Result<(), MemError> {
    // ADD a register to A and store the result in A
    let res = state.r_reg(n2)? as u16 + state.cpu.r[reg::A as usize] as u16;

    state.cpu.r[reg::A as usize] = res as u8;

    state.cpu.r[reg::F as usize] = 0;

    if (state.r_reg(n2)? & 0xf) + (state.cpu.r[reg::A as usize] & 0xf) > 0xf {
        state.cpu.r[reg::F as usize] |= flag::H;
    }

    if res > 0xff {
        state.cpu.r[reg::F as usize] |= flag::CY;
    }

    if state.cpu.r[reg::A as usize] == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }

    Ok(())
}

pub fn adcr(state: &mut GBState, n2: u8) -> Result<(), MemError> {
    // ADD a register and the carry flag to A and store the result in A
    let carry = (state.cpu.r[reg::F as usize] & flag::CY) >> 4;
    let res = state.r_reg(n2)? as u16 + state.cpu.r[reg::A as usize] as u16 + carry as u16;

    state.cpu.r[reg::A as usize] = res as u8;

    state.cpu.r[reg::F as usize] = 0;

    if (state.r_reg(n2)? & 0xf) + ((state.cpu.r[reg::A as usize] & 0xf) + carry) > 0xf {
        state.cpu.r[reg::F as usize] |= flag::H;
    }

    if res > 0xff {
        state.cpu.r[reg::F as usize] |= flag::CY;
    }

    if state.cpu.r[reg::A as usize] == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }

    Ok(())
}

pub fn subr(state: &mut GBState, n2: u8) -> Result<(), MemError> {
    // SUB a register to A and store the result in A
    state.cpu.r[reg::F as usize] = flag::N;

    if (state.r_reg(n2)? & 0xf) > (state.cpu.r[reg::A as usize] & 0xf) {
        state.cpu.r[reg::F as usize] |= flag::H;
    }

    if state.r_reg(n2)? > state.cpu.r[reg::A as usize] {
        state.cpu.r[reg::F as usize] |= flag::CY;
    }

    state.cpu.r[reg::A as usize] = state.cpu.r[reg::A as usize] - state.r_reg(n2)?;

    if state.cpu.r[reg::A as usize] == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }

    Ok(())
}

pub fn sbcr(state: &mut GBState, n2: u8) -> Result<(), MemError> {
    // SUB a register and the carry flag to A and store the result in A
    let carry = (state.cpu.r[reg::F as usize] & flag::CY) >> 4;
    state.cpu.r[reg::F as usize] = flag::N;

    if (state.r_reg(n2)? & 0xf) > (state.cpu.r[reg::A as usize] & 0xf) - carry {
        state.cpu.r[reg::F as usize] |= flag::H;
    }

    if state.r_reg(n2)? > state.cpu.r[reg::A as usize] - carry {
        state.cpu.r[reg::F as usize] |= flag::CY;
    }

    state.cpu.r[reg::A as usize] = state.cpu.r[reg::A as usize] - state.r_reg(n2)? - carry;

    if state.cpu.r[reg::A as usize] == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }

    Ok(())
}

pub fn andr(state: &mut GBState, n2: u8) -> Result<(), MemError> {
    // AND a register to A and store the result in A
    state.cpu.r[reg::A as usize] &= state.r_reg(n2)?;

    state.cpu.r[reg::F as usize] = flag::H;

    if state.cpu.r[reg::A as usize] == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }

    Ok(())
}

pub fn xorr(state: &mut GBState, n2: u8) -> Result<(), MemError> {
    // XOR a register to A and store the result in A
    state.cpu.r[reg::A as usize] ^= state.r_reg(n2)?;

    state.cpu.r[reg::F as usize] = 0;

    if state.cpu.r[reg::A as usize] == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }

    Ok(())
}

pub fn orr(state: &mut GBState, n2: u8) -> Result<(), MemError> {
    // OR a register to A and store the result in A
    state.cpu.r[reg::A as usize] |= state.r_reg(n2)?;

    state.cpu.r[reg::F as usize] = 0;

    if state.cpu.r[reg::A as usize] == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }

    Ok(())
}

pub fn cpr(state: &mut GBState, n2: u8) -> Result<(), MemError> {
    // SUB a register to A and update the flags accordingly without updating A
    state.cpu.r[reg::F as usize] |= flag::N;

    if state.r_reg(n2)? & 0xf > state.cpu.r[reg::A as usize] & 0xf {
        state.cpu.r[reg::F as usize] |= flag::H;
    }

    if state.r_reg(n2)? > state.cpu.r[reg::A as usize] {
        state.cpu.r[reg::F as usize] |= flag::CY;
    }

    let res = state.cpu.r[reg::A as usize] - state.r_reg(n2)?;

    if res == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }

    Ok(())
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
            0b000 => todo!(), // RLCA
            0b001 => todo!(), // RRCA
            0b010 => todo!(), // RLA
            0b011 => todo!(), // RRA
            0b100 => Ok(daa(state)),
            0b101 => Ok(cpl(state)),
            0b110 => Ok(scf(state)),
            0b111 => Ok(ccf(state)),
            _ => panic!(),
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

pub fn op10(state: &mut GBState, n1: u8, n2: u8) -> Result<(), MemError> {
    match n1 {
        0b000 => addr(state, n2), // ADD
        0b001 => adcr(state, n2), // ADC
        0b010 => subr(state, n2), // SUB
        0b011 => sbcr(state, n2), // SBC
        0b100 => andr(state, n2), // AND
        0b101 => xorr(state, n2), // XOR
        0b110 => orr(state, n2),  // OR
        0b111 => cpr(state, n2),  // CP
        _ => panic!(),
    }
}

pub fn op11(_state: &mut GBState, _n1: u8, _n2: u8) -> Result<(), MemError> {
    // Need some more brain juice to understand how theses are categorized
    todo!()
}
