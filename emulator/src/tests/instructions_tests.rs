use crate::exec_opcode;
use crate::state::{flag, reg, GBState, MemError};

#[test]
fn test_ldrr() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b01000001)?; // Opcode for LD B, C
    state.cpu.r[reg::C as usize] = 42;
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r[reg::B as usize], 42);
    Ok(())
}

#[test]
fn test_ldr8() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b00000110)?; // Opcode for LD B, n
    state.mem.w(1, 0x42)?; // n = 0x42
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r[reg::B as usize], 0x42);
    Ok(())
}

#[test]
fn test_ldrr16() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b00000001)?; // Opcode for LD BC, nn
    state.mem.w(1, 0x34)?; // nn lower byte = 0x34
    state.mem.w(2, 0x12)?; // nn higher byte = 0x12
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r16(reg::B), 0x1234);
    Ok(())
}

#[test]
fn test_ldnnsp() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b00001000)?; // Opcode for LD (nn), SP
    state.mem.w(1, 0x34)?; // nn lower byte = 0x34
    state.mem.w(2, 0x12)?; // nn higher byte = 0x12
    state.cpu.sp = 0x5678; // SP = 0x5678
    exec_opcode(&mut state)?;
    assert_eq!(state.mem.r(0x1234)?, 0x78); // SP low
    assert_eq!(state.mem.r(0x1235)?, 0x56); // SP high
    Ok(())
}

#[test]
fn test_jr8() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b00011000)?; // Opcode for JR n
    state.mem.w(1, 0x05)?; // n = 5
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.pc, 0x07); // PC is incremented by n (5) and the size of the instruction itself (2)
    Ok(())
}

#[test]
fn test_jrcc8() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b00100000)?; // Opcode for JR NZ, n (NZ is used for simplicity)
    state.mem.w(1, 0x05)?; // n = 5
    state.cpu.r[reg::F as usize] = 0; // Zero flag = 0, meaning NZ (not zero) is true
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.pc, 0x07); // PC is incremented by n (5) and the size of the instruction itself (2)
    Ok(())
}

#[test]
fn test_ld00a_normal() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.cpu.r[reg::A as usize] = 0x42;
    state.cpu.w16(reg::B, 0x1000);
    state.mem.w(0, 0b00000010)?; // Opcode for LD (BC), A
    exec_opcode(&mut state)?;
    assert_eq!(state.mem.r(0x1000)?, 0x42);
    Ok(())
}

#[test]
fn test_ld00a_hl_plus() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.cpu.r[reg::A as usize] = 0x42;
    state.cpu.w16(reg::HL, 0x1000);
    state.mem.w(0, 0b00100010)?; // Opcode for LD (HL+), A
    exec_opcode(&mut state)?;
    assert_eq!(state.mem.r(0x1000)?, 0x42);
    assert_eq!(state.cpu.r16(reg::HL), 0x1001); // Check if HL was incremented
    Ok(())
}

#[test]
fn test_ld00a_hl_minus() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.cpu.r[reg::A as usize] = 0x42;
    state.cpu.w16(reg::HL, 0x1000);
    state.mem.w(0, 0b00110010)?; // Opcode for LD (HL-), A
    exec_opcode(&mut state)?;
    assert_eq!(state.mem.r(0x1000)?, 0x42);
    assert_eq!(state.cpu.r16(reg::HL), 0xfff);
    Ok(())
}

#[test]
fn test_inc8() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b00000100)?; // Opcode for INC B
    state.cpu.r[reg::B as usize] = 42;
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r[reg::B as usize], 43);
    Ok(())
}

#[test]
fn test_dec8() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b00000101)?; // Opcode for DEC B
    state.cpu.r[reg::B as usize] = 42;
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r[reg::B as usize], 41);
    Ok(())
}

#[test]
fn test_ccf() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b00111111)?; // Opcode for CCF
    state.cpu.r[reg::F as usize] = 0b00010000; // Set the carry flag
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r[reg::F as usize] & 0b00010000, 0); // Check if carry flag is reset
    Ok(())
}

#[test]
fn test_scf() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b00110111)?; // Opcode for SCF
    state.cpu.r[reg::F as usize] = 0; // Reset the carry flag
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r[reg::F as usize] & 0b00010000, 0b00010000); // Check if carry flag is set
    Ok(())
}

#[test]
fn test_cpl() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b00101111)?; // Opcode for CPL
    state.cpu.r[reg::A as usize] = 0b10101010;
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r[reg::A as usize], 0b01010101); // Check if all bits in A have been flipped
    Ok(())
}

#[test]
fn test_daa_add() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b00100111)?; // DAA opcode
    state.cpu.r[reg::A as usize] = 0x1B;

    exec_opcode(&mut state)?;

    assert_eq!(state.cpu.r[reg::A as usize], 0x21);

    Ok(())
}

#[test]
fn test_daa_sub() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b00100111)?; // DAA opcode
    state.cpu.r[reg::A as usize] = 0x1F;
    state.cpu.r[reg::F as usize] = flag::N; // Substraction flag

    exec_opcode(&mut state)?;

    assert_eq!(state.cpu.r[reg::A as usize], 0x19);

    Ok(())
}

#[test]
fn test_add() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b10000000)?; // Opcode for ADD A, B
    state.cpu.r[reg::A as usize] = 10;
    state.cpu.r[reg::B as usize] = 15;
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r[reg::A as usize], 25);
    assert_eq!(state.cpu.r[reg::F as usize] & flag::ZF, 0); // Check Zero Flag
    assert_eq!(state.cpu.r[reg::F as usize] & flag::N, 0); // Check Subtract Flag
    assert_eq!(state.cpu.r[reg::F as usize] & flag::H, flag::H); // Check Half Carry Flag
    assert_eq!(state.cpu.r[reg::F as usize] & flag::CY, 0); // Check Carry Flag
    Ok(())
}

#[test]
fn test_adc() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b10001000)?; // Opcode for ADC A, B
    state.cpu.r[reg::A as usize] = 10;
    state.cpu.r[reg::B as usize] = 15;
    state.cpu.r[reg::F as usize] |= flag::CY; // Set Carry Flag
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r[reg::A as usize], 26);
    assert_eq!(state.cpu.r[reg::F as usize] & flag::ZF, 0); // Check Zero Flag
    assert_eq!(state.cpu.r[reg::F as usize] & flag::N, 0); // Check Subtract Flag
    assert_eq!(state.cpu.r[reg::F as usize] & flag::H, flag::H); // Check Half Carry Flag
    assert_eq!(state.cpu.r[reg::F as usize] & flag::CY, 0); // Check Carry Flag
    Ok(())
}

#[test]
fn test_sub() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b10010000)?; // Opcode for SUB A, B
    state.cpu.r[reg::A as usize] = 20;
    state.cpu.r[reg::B as usize] = 15;
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r[reg::A as usize], 5);
    assert_eq!(state.cpu.r[reg::F as usize] & flag::ZF, 0); // Check Zero Flag
    assert_eq!(state.cpu.r[reg::F as usize] & flag::N, flag::N); // Check Subtract Flag
    assert_eq!(state.cpu.r[reg::F as usize] & flag::H, flag::H); // Check Half Carry Flag
    assert_eq!(state.cpu.r[reg::F as usize] & flag::CY, 0); // Check Carry Flag
    Ok(())
}

#[test]
fn test_sbc() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b10011000)?; // Opcode for SBC A, B
    state.cpu.r[reg::A as usize] = 10;
    state.cpu.r[reg::B as usize] = 3;
    state.cpu.r[reg::F as usize] |= flag::CY; // Set Carry Flag
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r[reg::A as usize], 6);
    assert_eq!(state.cpu.r[reg::F as usize] & flag::ZF, 0); // Check Zero Flag
    assert_eq!(state.cpu.r[reg::F as usize] & flag::N, flag::N); // Check Subtract Flag
    assert_eq!(state.cpu.r[reg::F as usize] & flag::H, 0); // Check Half Carry Flag
    assert_eq!(state.cpu.r[reg::F as usize] & flag::CY, 0); // Check Carry Flag
    Ok(())
}

#[test]
fn test_and() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b10100000)?; // Opcode for AND A, B
    state.cpu.r[reg::A as usize] = 0b10101010;
    state.cpu.r[reg::B as usize] = 0b11001100;
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r[reg::A as usize], 0b10001000);
    Ok(())
}

#[test]
fn test_xor() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b10101000)?; // Opcode for XOR A, B
    state.cpu.r[reg::A as usize] = 0b10101010;
    state.cpu.r[reg::B as usize] = 0b11001100;
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r[reg::A as usize], 0b01100110);
    Ok(())
}

#[test]
fn test_or() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b10110000)?; // Opcode for OR A, B
    state.cpu.r[reg::A as usize] = 0b10101010;
    state.cpu.r[reg::B as usize] = 0b11001100;
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r[reg::A as usize], 0b11101110);
    Ok(())
}

#[test]
fn test_cp() -> Result<(), MemError> {
    let mut state = GBState::new();
    state.mem.w(0, 0b10111000)?; // Opcode for CP A, B
    state.cpu.r[reg::A as usize] = 20;
    state.cpu.r[reg::B as usize] = 15;
    exec_opcode(&mut state)?;
    assert_eq!(state.cpu.r[reg::A as usize], 20);
    assert_eq!(state.cpu.r[reg::F as usize] & flag::ZF, 0); // Check Zero Flag
    assert_eq!(state.cpu.r[reg::F as usize] & flag::N, flag::N); // Check Subtract Flag
    assert_eq!(state.cpu.r[reg::F as usize] & flag::H, flag::H); // Check Half Carry Flag
    assert_eq!(state.cpu.r[reg::F as usize] & flag::CY, 0); // Check Carry Flag
    Ok(())
}
