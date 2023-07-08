use crate::state::{flag, reg, GBState, MemError};

pub fn r_16b_from_pc(state: &mut GBState) -> Result<u16, MemError> {
    let p: u16 = state.mem.r(state.cpu.pc)? as u16 | ((state.mem.r(state.cpu.pc + 1)? as u16) << 8);
    state.cpu.pc += 2;

    Ok(p)
}

pub fn r_8b_from_pc(state: &mut GBState) -> Result<u8, MemError> {
    let p = state.mem.r(state.cpu.pc)?;
    state.cpu.pc += 1;

    Ok(p)
}

pub fn ldrr(state: &mut GBState, n1: u8, n2: u8) -> Result<(), MemError> {
    // Load a register into another register
    // LD r, r
    state.w_reg(n1, state.r_reg(n2)?)
}

pub fn ldr8(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    // Load an raw 8b value into a register
    let p = r_8b_from_pc(state)?;

    state.w_reg(n1, p)
}

pub fn ldrr16(state: &mut GBState, rr: u8, x: u16) {
    // Load a raw 16b value into a register
    state.cpu.w16(rr, x);
}

pub fn ldnnsp(state: &mut GBState) -> Result<(), MemError> {
    // Load SP into an arbitrary position in memory
    let p = r_16b_from_pc(state)?;

    state.mem.w(p, (state.cpu.sp & 0xff) as u8)?;
    state.mem.w(p + 1, (state.cpu.sp >> 8) as u8)
}

pub fn ldsphl(state: &mut GBState) {
    state.cpu.sp = state.cpu.r16(reg::HL);
}

pub fn ldnna(state: &mut GBState, nn: u16) -> Result<(), MemError> {
    // Load A into an arbitrary position in memory
    state.mem.w(nn, state.cpu.r[reg::A as usize])?;
    Ok(())
}

pub fn ldann(state: &mut GBState, nn: u16) -> Result<(), MemError> {
    // Load A from an arbitrary position in memory
    state.cpu.r[reg::A as usize] = state.mem.r(nn)?;
    Ok(())
}

pub fn push(state: &mut GBState, x: u16) -> Result<(), MemError> {
    state.cpu.sp -= 2;

    state.mem.w(state.cpu.sp, (x & 0xff) as u8)?;

    state.mem.w(state.cpu.sp + 1, (x >> 8) as u8)?;

    Ok(())
}

pub fn pop(state: &mut GBState) -> Result<u16, MemError> {
    let res = state.mem.r(state.cpu.sp)? as u16 | ((state.mem.r(state.cpu.sp + 1)? as u16) << 8);

    state.cpu.sp += 2;

    Ok(res)
}

pub fn jr8(state: &mut GBState) -> Result<(), MemError> {
    // Unconditional relative jump
    let p = r_8b_from_pc(state)?;

    state.cpu.pc = (state.cpu.pc as i16 + p as i8 as i16) as u16;

    Ok(())
}

pub fn jrcc8(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    // Conditional relative jump
    let p = r_8b_from_pc(state)?;

    if state.cpu.check_flag(n1 & 0b11) {
        state.cpu.pc = (state.cpu.pc as i16 + p as i8 as i16) as u16;
    }

    Ok(())
}

pub fn jp16(state: &mut GBState) -> Result<(), MemError> {
    // Unconditional absolute jump
    let p = r_16b_from_pc(state)?;

    state.cpu.pc = p;

    Ok(())
}

pub fn jphl(state: &mut GBState) {
    // Unconditional absolute jump to HL
    state.cpu.pc = state.cpu.r16(reg::HL);
}

pub fn jpcc16(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    // Conditional absolute jump
    let p = r_16b_from_pc(state)?;

    if state.cpu.check_flag(n1 & 0b11) {
        state.cpu.pc = p;
    }

    Ok(())
}

pub fn call(state: &mut GBState) -> Result<(), MemError> {
    // Unconditional function call
    let p = r_16b_from_pc(state)?;

    push(state, state.cpu.pc)?;
    state.cpu.pc = p;

    Ok(())
}

pub fn callcc(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    // Conditional function call
    let p = r_16b_from_pc(state)?;

    if state.cpu.check_flag(n1 & 0b11) {
        push(state, state.cpu.pc)?;
        state.cpu.pc = p;
    }

    Ok(())
}

pub fn ret(state: &mut GBState) -> Result<(), MemError> {
    state.cpu.pc = pop(state)?;

    Ok(())
}

pub fn retcc(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    if state.cpu.check_flag(n1 & 0b11) {
        state.cpu.pc = pop(state)?;
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
    let half_carry_flag = (state.cpu.r[reg::F as usize] & flag::H) != 0;

    if (half_carry_flag || nibble_low > 9) && !sub_flag {
        state.cpu.r[reg::A as usize] += 0x06;
    }
    if (half_carry_flag || nibble_low > 9) && sub_flag {
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

pub fn addsp8(state: &mut GBState) -> Result<(), MemError> {
    let n = r_8b_from_pc(state)? as i8;

    state.cpu.sp = (state.cpu.sp as i32 + n as i32) as u16;

    state.cpu.r[reg::F as usize] &= !(flag::N | flag::H | flag::CY);

    if (state.cpu.sp & 0xff) as i32 + n as i32 & !0xff != 0 {
        state.cpu.r[reg::F as usize] |= flag::H;
    }

    if (state.cpu.sp as i32 + n as i32) & !0xffff != 0 {
        state.cpu.r[reg::F as usize] |= flag::CY;
    }
    Ok(())
}

pub fn add(state: &mut GBState, x: u8) {
    // ADD a number to A and store the result in A
    let res = x as u16 + state.cpu.r[reg::A as usize] as u16;

    state.cpu.r[reg::A as usize] = res as u8;

    state.cpu.r[reg::F as usize] = 0;

    if (x & 0xf) + (state.cpu.r[reg::A as usize] & 0xf) > 0xf {
        state.cpu.r[reg::F as usize] |= flag::H;
    }

    if res > 0xff {
        state.cpu.r[reg::F as usize] |= flag::CY;
    }

    if state.cpu.r[reg::A as usize] == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }
}

pub fn adc(state: &mut GBState, x: u8) {
    // ADD a number and the carry flag to A and store the result in A
    let carry = (state.cpu.r[reg::F as usize] & flag::CY) >> 4;
    let res = x as u16 + state.cpu.r[reg::A as usize] as u16 + carry as u16;

    state.cpu.r[reg::A as usize] = res as u8;

    state.cpu.r[reg::F as usize] = 0;

    if (x & 0xf) + ((state.cpu.r[reg::A as usize] & 0xf) + carry) > 0xf {
        state.cpu.r[reg::F as usize] |= flag::H;
    }

    if res > 0xff {
        state.cpu.r[reg::F as usize] |= flag::CY;
    }

    if state.cpu.r[reg::A as usize] == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }
}

pub fn sub(state: &mut GBState, x: u8) {
    // SUB a number to A and store the result in A
    state.cpu.r[reg::F as usize] = flag::N;

    if (x & 0xf) > (state.cpu.r[reg::A as usize] & 0xf) {
        state.cpu.r[reg::F as usize] |= flag::H;
    }

    if x > state.cpu.r[reg::A as usize] {
        state.cpu.r[reg::F as usize] |= flag::CY;
    }

    state.cpu.r[reg::A as usize] = state.cpu.r[reg::A as usize] - x;

    if state.cpu.r[reg::A as usize] == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }
}

pub fn sbc(state: &mut GBState, x: u8) {
    // SUB a number and the carry flag to A and store the result in A
    let carry = (state.cpu.r[reg::F as usize] & flag::CY) >> 4;
    state.cpu.r[reg::F as usize] = flag::N;

    if (x & 0xf) > (state.cpu.r[reg::A as usize] & 0xf) - carry {
        state.cpu.r[reg::F as usize] |= flag::H;
    }

    if x > state.cpu.r[reg::A as usize] - carry {
        state.cpu.r[reg::F as usize] |= flag::CY;
    }

    state.cpu.r[reg::A as usize] = state.cpu.r[reg::A as usize] - x - carry;

    if state.cpu.r[reg::A as usize] == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }
}

pub fn and(state: &mut GBState, x: u8) {
    // AND a number to A and store the result in A
    state.cpu.r[reg::A as usize] &= x;

    state.cpu.r[reg::F as usize] = flag::H;

    if state.cpu.r[reg::A as usize] == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }
}

pub fn xor(state: &mut GBState, x: u8) {
    // XOR a number to A and store the result in A
    state.cpu.r[reg::A as usize] ^= x;

    state.cpu.r[reg::F as usize] = 0;

    if state.cpu.r[reg::A as usize] == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }
}

pub fn or(state: &mut GBState, x: u8) {
    // OR a number to A and store the result in A
    state.cpu.r[reg::A as usize] |= x;

    state.cpu.r[reg::F as usize] = 0;

    if state.cpu.r[reg::A as usize] == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }
}

pub fn cp(state: &mut GBState, x: u8) {
    // SUB a number to A and update the flags accordingly without updating A
    state.cpu.r[reg::F as usize] |= flag::N;

    if x & 0xf > state.cpu.r[reg::A as usize] & 0xf {
        state.cpu.r[reg::F as usize] |= flag::H;
    }

    if x > state.cpu.r[reg::A as usize] {
        state.cpu.r[reg::F as usize] |= flag::CY;
    }

    let res = state.cpu.r[reg::A as usize] - x;

    if res == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }
}

pub fn rlca(state: &mut GBState) {
    // ROTATE LEFT the A register
    state.cpu.r[reg::F as usize] &= !(flag::H | flag::N | flag::ZF | flag::CY);
    state.cpu.r[reg::F as usize] |= (state.cpu.r[reg::A as usize] >> 7) << 4;
    state.cpu.r[reg::A as usize] <<= 1;
    state.cpu.r[reg::A as usize] |= (state.cpu.r[reg::F as usize] & flag::CY) >> 4;
}

pub fn rrca(state: &mut GBState) {
    // ROTATE RIGHT the A register
    state.cpu.r[reg::F as usize] &= !(flag::H | flag::N | flag::ZF | flag::CY);
    state.cpu.r[reg::F as usize] |= (state.cpu.r[reg::A as usize] & 1) << 4;
    state.cpu.r[reg::A as usize] >>= 1;
    state.cpu.r[reg::A as usize] |= ((state.cpu.r[reg::F as usize] & flag::CY) >> 4) << 7;
}

pub fn rla(state: &mut GBState) {
    // ROTATE LEFT THROUGH CARRY the A register
    let carry = (state.cpu.r[reg::F as usize] & flag::CY) >> 4;

    state.cpu.r[reg::F as usize] &= !(flag::H | flag::N | flag::ZF | flag::CY);
    state.cpu.r[reg::F as usize] |= (state.cpu.r[reg::A as usize] >> 7) << 4;
    state.cpu.r[reg::A as usize] <<= 1;
    state.cpu.r[reg::A as usize] |= carry;
}

pub fn rra(state: &mut GBState) {
    // ROTATE RIGHT THROUGH CARRY the A register
    let carry = (state.cpu.r[reg::F as usize] & flag::CY) >> 4;

    state.cpu.r[reg::F as usize] &= !(flag::H | flag::N | flag::ZF | flag::CY);
    state.cpu.r[reg::F as usize] |= (state.cpu.r[reg::A as usize] & 1) << 4;
    state.cpu.r[reg::A as usize] >>= 1;
    state.cpu.r[reg::A as usize] |= carry << 7;
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
        0b001 => {
            let p = r_16b_from_pc(state)?;
            ldrr16(state, n1 >> 1, p);
            Ok(())
        }
        0b010 => ld00a(state, n1),
        0b011 => todo!(), // 16b INC (?)
        0b100 => inc8(state, n1),
        0b101 => dec8(state, n1),
        0b110 => ldr8(state, n1),
        0b111 => match n1 {
            0b000 => Ok(rlca(state)),
            0b001 => Ok(rrca(state)),
            0b010 => Ok(rla(state)),
            0b011 => Ok(rra(state)),
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
    // Dispatcher for the instructions starting with 0b10 (Arithmetic)
    match n1 {
        0b000 => Ok(add(state, state.r_reg(n2)?)),
        0b001 => Ok(adc(state, state.r_reg(n2)?)),
        0b010 => Ok(sub(state, state.r_reg(n2)?)),
        0b011 => Ok(sbc(state, state.r_reg(n2)?)),
        0b100 => Ok(and(state, state.r_reg(n2)?)),
        0b101 => Ok(xor(state, state.r_reg(n2)?)),
        0b110 => Ok(or(state, state.r_reg(n2)?)),
        0b111 => Ok(cp(state, state.r_reg(n2)?)),
        _ => panic!(),
    }
}

pub fn op11(state: &mut GBState, n1: u8, n2: u8) -> Result<(), MemError> {
    match n2 {
        0b000 => match n1 {
            0b100 => {
                let n = r_8b_from_pc(state)?;
                ldnna(state, n as u16 | 0xff00)
            }
            0b101 => addsp8(state),
            0b110 => {
                let n = r_8b_from_pc(state)?;
                ldann(state, n as u16 | 0xff00)
            }
            0b111 => {
                let n = r_8b_from_pc(state)?;
                ldrr16(state, reg::HL, n as u16 + state.cpu.sp);
                Ok(())
            }
            _ => retcc(state, n1 & 0b11),
        },
        0b001 => match n1 {
            0b001 => ret(state),
            0b011 => todo!(), // RETI
            0b101 => Ok(jphl(state)),
            0b111 => Ok(ldsphl(state)),
            _ => {
                let p = pop(state)?;
                state.cpu.w16(n1 >> 1, p);
                Ok(())
            }
        },
        0b010 => match n1 {
            0b100 => ldnna(state, state.cpu.r[reg::C as usize] as u16 | 0xff00),
            0b101 => {
                let nn = r_16b_from_pc(state)?;
                ldnna(state, nn)
            }
            0b110 => ldann(state, state.cpu.r[reg::C as usize] as u16 | 0xff00),
            0b111 => {
                let nn = r_16b_from_pc(state)?;
                ldann(state, nn)
            }
            _ => jpcc16(state, n1 & 0b11),
        },
        0b011 => match n1 {
            0b000 => jp16(state),
            0b001 => todo!(), // Bitwise operations
            0b010 | 0b011 | 0b100 | 0b101 => unimplemented!(),
            0b110 => todo!(), // DI
            0b111 => todo!(), // EI
            _ => panic!(),
        },
        0b100 => callcc(state, n1 & 0b11),
        0b101 => match n1 {
            0b001 => call(state),
            0b011 | 0b101 | 0b111 => unimplemented!(),
            _ => push(state, state.cpu.r16(n1 >> 1)),
        },
        0b110 => {
            let p = r_8b_from_pc(state)?;

            match n1 {
                0b000 => Ok(add(state, p)),
                0b001 => Ok(adc(state, p)),
                0b010 => Ok(sub(state, p)),
                0b011 => Ok(sbc(state, p)),
                0b100 => Ok(and(state, p)),
                0b101 => Ok(xor(state, p)),
                0b110 => Ok(or(state, p)),
                0b111 => Ok(cp(state, p)),
                _ => panic!(),
            }
        }
        0b111 => todo!(), // RST
        _ => panic!(),
    }
}
