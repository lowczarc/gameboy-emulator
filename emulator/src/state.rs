use crate::consts::{BOOT_ROM_FILE, PROGRAM_START_ADDRESS, STACK_START_ADDRESS};
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub struct CPU {
    /* B, C, D, E, H, L, A, F registers.
     * A is usually represented by 111 even though it's in index 6.
     * (HL) usually takes the 110 representation.
     * F isn't usually used by the 8bits register operations. */
    pub r: [u8; 8],

    pub pc: u16, // program counter
    pub sp: u16, // stack pointer
}

impl CPU {
    pub fn new() -> Self {
        Self {
            r: [0; 8],

            pc: PROGRAM_START_ADDRESS,
            sp: STACK_START_ADDRESS,
        }
    }
}

#[derive(Debug)]
pub struct Memory {
    // 16 KiB ROM bank 00
    rom_00: [u8; 0x4000],
}

#[derive(Debug)]
pub enum MemError {
    WritingToROM,
    Unimplemented,
    NotUsable,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            rom_00: [0; 0x4000],
        }
    }

    pub fn load_boot_rom(&mut self) -> Result<(), std::io::Error> {
        let mut f = File::open(BOOT_ROM_FILE)?;

        f.read(&mut self.rom_00[0x00..0x100])?;

        Ok(())
    }

    pub fn r(&self, addr: u16) -> Result<u8, MemError> {
        if addr < 0x4000 {
            Ok(self.rom_00[addr as usize])
        } else {
            Err(MemError::Unimplemented)
        }
    }

    pub fn w(&mut self, addr: u16, value: u8) -> Result<(), MemError> {
        if addr < 0x4000 {
            self.rom_00[addr as usize] = value;
            Ok(())
        } else {
            Err(MemError::Unimplemented)
        }
    }
}

pub struct GBState {
    pub cpu: CPU,
    mem: Memory,
}

impl GBState {
    pub fn new() -> Self {
        let mut mem = Memory::new();

        mem.load_boot_rom().unwrap();

        Self {
            cpu: CPU::new(),
            mem,
        }
    }

    pub fn r_mem(&self, addr: u16) -> Result<u8, MemError> {
        self.mem.r(addr)
    }

    pub fn r_reg(&self, r_i: u8) -> Result<u8, MemError> {
        if r_i < 6 {
            Ok(self.cpu.r[r_i as usize])
        } else if r_i == 7 {
            Ok(self.cpu.r[6])
        } else if r_i == 6 {
            let hl: u16 = self.cpu.r[4] as u16 | (self.cpu.r[5] as u16) << 8;

            self.mem.r(hl)
        } else {
            panic!("r_i must be a 3 bits register input number")
        }
    }

    pub fn w_reg(&mut self, r_i: u8, value: u8) -> Result<(), MemError> {
        if r_i < 6 {
            self.cpu.r[r_i as usize] = value;
        } else if r_i == 7 {
            self.cpu.r[6] = value;
        } else if r_i == 6 {
            let hl: u16 = self.cpu.r[4] as u16 | (self.cpu.r[5] as u16) << 8;

            self.mem.w(hl, value)?;
        } else {
            panic!("r_i must be a 3 bits register input number")
        }
        Ok(())
    }
}
