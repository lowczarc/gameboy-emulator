use crate::consts::CPU_CYCLE_LENGTH_NANOS;
use crate::state::{flag, reg, GBState, MemError};
use std::{thread, time};

pub fn cycles(n: u64) {
    thread::sleep(time::Duration::from_nanos(CPU_CYCLE_LENGTH_NANOS * n))
}

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

    cycles(8);
    state.w_reg(n1, p)
}

pub fn ldrr16(state: &mut GBState, rr: u8, x: u16) {
    // Load a raw 16b value into a register
    state.cpu.w16(rr, x);
}

pub fn ldnnsp(state: &mut GBState) -> Result<(), MemError> {
    // Load SP into an arbitrary position in memory
    let p = r_16b_from_pc(state)?;

    cycles(20);
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

    if state.cpu.sp > 0xfffc || state.cpu.sp < 0xff80 {
        panic!("Stack overflow on PUSH. SP: 0x{:04x}", state.cpu.sp);
    }
    state.mem.w(state.cpu.sp, (x & 0xff) as u8)?;

    state.mem.w(state.cpu.sp + 1, (x >> 8) as u8)?;

    Ok(())
}

pub fn pop(state: &mut GBState) -> Result<u16, MemError> {
    if state.cpu.sp > 0xfffc || state.cpu.sp < 0xff80 {
        panic!("Stack overflow on POP. SP: 0x{:04x}", state.cpu.sp);
    }

    let res = state.mem.r(state.cpu.sp)? as u16 | ((state.mem.r(state.cpu.sp + 1)? as u16) << 8);

    state.cpu.sp += 2;

    Ok(res)
}

pub fn jr8(state: &mut GBState) -> Result<(), MemError> {
    // Unconditional relative jump
    let p = r_8b_from_pc(state)?;

    state.cpu.pc = (state.cpu.pc as i16 + p as i8 as i16) as u16;

    Ok(cycles(12))
}

pub fn jrcc8(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    // Conditional relative jump
    let p = r_8b_from_pc(state)?;

    if state.cpu.check_flag(n1 & 0b11) {
        cycles(4);
        state.cpu.pc = (state.cpu.pc as i16 + p as i8 as i16) as u16;
    }

    Ok(cycles(8))
}

pub fn jp16(state: &mut GBState) -> Result<(), MemError> {
    // Unconditional absolute jump
    let p = r_16b_from_pc(state)?;

    state.cpu.pc = p;

    Ok(cycles(16))
}

pub fn jphl(state: &mut GBState) {
    // Unconditional absolute jump to HL
    state.cpu.pc = state.cpu.r16(reg::HL);
}

pub fn jpcc16(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    // Conditional absolute jump
    let p = r_16b_from_pc(state)?;

    if state.cpu.check_flag(n1 & 0b11) {
        cycles(4);
        state.cpu.pc = p;
    }

    Ok(cycles(12))
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
        cycles(12);
        push(state, state.cpu.pc)?;
        state.cpu.pc = p;
    }

    Ok(cycles(12))
}

pub fn ret(state: &mut GBState) -> Result<(), MemError> {
    state.cpu.pc = pop(state)?;

    if state.cpu.pc == 0 {
        panic!("RET to start");
    }
    Ok(cycles(16))
}

pub fn retcc(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    if state.cpu.check_flag(n1 & 0b11) {
        cycles(12);
        state.cpu.pc = pop(state)?;
    }

    Ok(cycles(8))
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

    Ok(cycles(8))
}

pub fn inc8(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    // Increment 8 bit register
    state.w_reg(n1, state.r_reg(n1)? + 1)?;
    state.cpu.r[reg::F as usize] &= !(flag::N | flag::ZF | flag::H);
    if state.r_reg(n1)? == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }

    if state.r_reg(n1)? & 0xf == 0x0 {
        state.cpu.r[reg::F as usize] |= flag::H;
    }

    Ok(cycles(4))
}

pub fn dec8(state: &mut GBState, n1: u8) -> Result<(), MemError> {
    // Decrement 8 bit register
    state.w_reg(n1, state.r_reg(n1)? - 1)?;
    state.cpu.r[reg::F as usize] |= flag::N;

    state.cpu.r[reg::F as usize] &= !(flag::ZF | flag::H);
    if state.r_reg(n1)? == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }

    if state.r_reg(n1)? & 0xf == 0xf {
        state.cpu.r[reg::F as usize] |= flag::H;
    }

    Ok(cycles(4))
}

pub fn inc16(state: &mut GBState, rr: u8) {
    // Increment 16 bit register
    state.cpu.w16(rr, state.cpu.r16(rr) + 1);
    state.cpu.r[reg::F as usize] &= !(flag::N | flag::ZF | flag::H);

    if state.cpu.r16(rr) == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }

    if state.cpu.r16(rr) & 0xff == 0x0 {
        state.cpu.r[reg::F as usize] |= flag::H;
    }
    cycles(8)
}

pub fn dec16(state: &mut GBState, rr: u8) {
    // Decrement 16 bit register
    state.cpu.w16(rr, state.cpu.r16(rr) - 1);
    state.cpu.r[reg::F as usize] |= flag::N;

    state.cpu.r[reg::F as usize] &= !(flag::ZF | flag::H);
    if state.cpu.r16(rr) == 0 {
        state.cpu.r[reg::F as usize] |= flag::ZF;
    }

    if state.cpu.r16(rr) & 0xff == 0xff {
        state.cpu.r[reg::F as usize] |= flag::H;
    }
    cycles(8)
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

pub fn rlc(state: &mut GBState, r_i: u8) -> Result<(), MemError> {
    // ROTATE LEFT the input register
    let mut n = state.r_reg(r_i)?;
    state.cpu.r[reg::F as usize] &= !(flag::H | flag::N | flag::ZF | flag::CY);
    state.cpu.r[reg::F as usize] |= (n >> 7) << 4;
    n <<= 1;
    n |= (state.cpu.r[reg::F as usize] & flag::CY) >> 4;
    state.w_reg(r_i, n)
}

pub fn rrc(state: &mut GBState, r_i: u8) -> Result<(), MemError> {
    // ROTATE RIGHT the input register
    let mut n = state.r_reg(r_i)?;
    state.cpu.r[reg::F as usize] &= !(flag::H | flag::N | flag::ZF | flag::CY);
    state.cpu.r[reg::F as usize] |= (n & 1) << 4;
    n >>= 1;
    n |= ((state.cpu.r[reg::F as usize] & flag::CY) >> 4) << 7;
    state.w_reg(r_i, n)
}

pub fn rl(state: &mut GBState, r_i: u8) -> Result<(), MemError> {
    // ROTATE LEFT THROUGH CARRY the input register
    let mut n = state.r_reg(r_i)?;
    let carry = (state.cpu.r[reg::F as usize] & flag::CY) >> 4;

    state.cpu.r[reg::F as usize] &= !(flag::H | flag::N | flag::ZF | flag::CY);
    state.cpu.r[reg::F as usize] |= (n >> 7) << 4;
    n <<= 1;
    n |= carry;
    state.w_reg(r_i, n)
}

pub fn rr(state: &mut GBState, r_i: u8) -> Result<(), MemError> {
    // ROTATE RIGHT THROUGH CARRY the input register
    let mut n = state.r_reg(r_i)?;
    let carry = (state.cpu.r[reg::F as usize] & flag::CY) >> 4;

    state.cpu.r[reg::F as usize] &= !(flag::H | flag::N | flag::ZF | flag::CY);
    state.cpu.r[reg::F as usize] |= (n & 1) << 4;
    n >>= 1;
    n |= carry << 7;
    state.w_reg(r_i, n)
}

pub fn bit(state: &mut GBState, n1: u8, n2: u8) -> Result<(), MemError> {
    let z = (((state.r_reg(n2)? >> n1) & 1) ^ 1) << 7;

    state.cpu.r[reg::F as usize] &= !(flag::N | flag::ZF);
    state.cpu.r[reg::F as usize] |= flag::H | z;
    Ok(())
}

pub fn op00(state: &mut GBState, n1: u8, n2: u8) -> Result<(), MemError> {
    // Dispatcher for the instructions starting with 0b00 based on their 3 LSB
    match n2 {
        0b000 => match n1 {
            0b000 => Ok(cycles(4)),
            0b001 => ldnnsp(state),
            0b010 => todo!("STOP"), // STOP
            0b011 => jr8(state),
            _ => jrcc8(state, n1),
        },
        0b001 => match n1 {
            0b001 | 0b011 | 0b101 | 0b111 => todo!("ADD HL rr"),
            0b000 | 0b010 | 0b100 | 0b110 => {
                let p = r_16b_from_pc(state)?;
                ldrr16(state, n1 >> 1, p);
                Ok(cycles(12))
            }
            _ => panic!(),
        },
        0b010 => ld00a(state, n1),
        0b011 => match n1 {
            0b001 | 0b011 | 0b101 | 0b111 => Ok(dec16(state, n1 >> 1)),
            0b000 | 0b010 | 0b100 | 0b110 => Ok(inc16(state, n1 >> 1)),
            _ => panic!(),
        },
        0b100 => inc8(state, n1),
        0b101 => dec8(state, n1),
        0b110 => ldr8(state, n1),
        0b111 => {
            cycles(4);
            match n1 {
                0b000 => rlc(state, 7),
                0b001 => rrc(state, 7),
                0b010 => rl(state, 7),
                0b011 => rr(state, 7),
                0b100 => Ok(daa(state)),
                0b101 => Ok(cpl(state)),
                0b110 => Ok(scf(state)),
                0b111 => Ok(ccf(state)),
                _ => panic!(),
            }
        }
        _ => panic!(),
    }
}

pub fn op01(state: &mut GBState, n1: u8, n2: u8) -> Result<(), MemError> {
    // Dispatcher for the instructions starting with 0b01 (LD r,r and HALT)
    if n1 == 0b110 && n2 == 0b110 {
        cycles(4);
        todo!("HALT") // HALT
    } else {
        cycles(4);
        if n1 == 0b110 || n2 == 0b110 {
            cycles(4);
        }
        ldrr(state, n1, n2)
    }
}

pub fn op10(state: &mut GBState, n1: u8, n2: u8) -> Result<(), MemError> {
    // Dispatcher for the instructions starting with 0b10 (Arithmetic)
    cycles(4);
    if n2 == 0b110 {
        cycles(4);
    }
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
                cycles(12);
                ldnna(state, n as u16 | 0xff00)
            }
            0b101 => addsp8(state),
            0b110 => {
                let n = r_8b_from_pc(state)?;
                cycles(12);
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
            0b011 => todo!("RETI"), // RETI
            0b101 => Ok(jphl(state)),
            0b111 => Ok(ldsphl(state)),
            _ => {
                let p = pop(state)?;
                cycles(12);
                state.cpu.w16(n1 >> 1, p);
                Ok(())
            }
        },
        0b010 => match n1 {
            0b100 => {
                cycles(8);
                ldnna(state, state.cpu.r[reg::C as usize] as u16 | 0xff00)
            }
            0b101 => {
                let nn = r_16b_from_pc(state)?;
                cycles(16);
                ldnna(state, nn)
            }
            0b110 => {
                cycles(8);
                ldann(state, state.cpu.r[reg::C as usize] as u16 | 0xff00)
            }
            0b111 => {
                let nn = r_16b_from_pc(state)?;
                cycles(16);
                ldann(state, nn)
            }
            _ => jpcc16(state, n1 & 0b11),
        },
        0b011 => match n1 {
            0b000 => jp16(state),
            0b001 => op_bitwise(state), // Bitwise operations
            0b010 | 0b011 | 0b100 | 0b101 => unimplemented!(),
            0b110 => todo!("DI"), // DI
            0b111 => todo!("EI"), // EI
            _ => panic!(),
        },
        0b100 => callcc(state, n1 & 0b11),
        0b101 => match n1 {
            0b001 => call(state),
            0b011 | 0b101 | 0b111 => unimplemented!(),
            _ => {
                cycles(16);
                push(state, state.cpu.r16(n1 >> 1))
            }
        },
        0b110 => {
            let p = r_8b_from_pc(state)?;

            cycles(8);
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
        0b111 => todo!("RST"), // RST
        _ => panic!(),
    }
}

pub fn op_bitwise(state: &mut GBState) -> Result<(), MemError> {
    let p = r_8b_from_pc(state)?;
    let opcode = p >> 6;
    let n1 = p >> 3 & 0b111;
    let n2 = p & 0b111;

    cycles(8);
    if n2 == 110 {
        cycles(8);
    }
    match opcode {
        0b00 => match n1 {
            0b000 => rlc(state, n2),
            0b001 => rrc(state, n2),
            0b010 => rl(state, n2),
            0b011 => rr(state, n2),
            0b100 => todo!("SLA"),
            0b101 => todo!("SRA"),
            0b110 => todo!("SWAP"),
            0b111 => todo!("SRL"),
            _ => panic!(),
        },
        0b01 => bit(state, n1, n2),
        0b10 => todo!("RES"),
        0b11 => todo!("SET"),
        _ => panic!(),
    }
}
