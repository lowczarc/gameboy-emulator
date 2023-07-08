use crate::consts::{BOOT_ROM_FILE, PROGRAM_START_ADDRESS, STACK_START_ADDRESS};
use crate::io::IORegisters;
use std::fs::File;
use std::io::Read;

pub mod reg {
    pub const B: u8 = 0;
    pub const C: u8 = 1;
    pub const D: u8 = 2;
    pub const E: u8 = 3;
    pub const H: u8 = 4;
    pub const L: u8 = 5;
    pub const A: u8 = 6;
    pub const F: u8 = 7;

    pub const BC: u8 = 0;
    pub const DE: u8 = 1;
    pub const HL: u8 = 2;
    pub const SP: u8 = 3;
}

pub mod flag {
    pub const NZ: u8 = 0;
    pub const Z: u8 = 1;
    pub const NC: u8 = 2;
    pub const C: u8 = 3;

    pub const CY: u8 = 1 << 4;
    pub const H: u8 = 1 << 5;
    pub const N: u8 = 1 << 6;
    pub const ZF: u8 = 1 << 7;
}

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

    pub fn r16(&self, r: u8) -> u16 {
        if r == reg::SP {
            return self.sp;
        } else {
            return self.r[r as usize * 2 + 1] as u16 | ((self.r[r as usize * 2] as u16) << 8);
        }
    }

    pub fn w16(&mut self, r: u8, value: u16) {
        if r == reg::SP {
            self.sp = value;
        } else {
            self.r[r as usize * 2 + 1] = (value & 0xff) as u8;
            self.r[r as usize * 2] = (value >> 8) as u8;
        }
    }

    pub fn check_flag(&self, flag: u8) -> bool {
        let f = self.r[reg::F as usize];

        match flag {
            flag::NZ => f >> 7 == 0,
            flag::Z => f >> 7 == 1,
            flag::NC => f >> 4 == 0,
            flag::C => f >> 4 == 1,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub struct Memory {
    // 16 KiB ROM bank 00
    rom_00: [u8; 0x4000],

    // 8 KiB Video RAM
    vram: [u8; 0x2000],

    io: IORegisters,

    // High RAM
    hram: [u8; 0x7f],
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
            vram: [0; 0x2000],
            io: IORegisters::new(),
            hram: [0; 0x7f],
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
        } else if addr >= 0x8000 && addr < 0xa000 {
            Ok(self.vram[addr as usize - 0x8000])
        } else if addr >= 0xff00 && addr < 0xff80 {
            Ok(self.io.r(addr as u8))
        } else if addr >= 0xff80 && addr < 0xffff {
            Ok(self.hram[addr as usize - 0xff80])
        } else {
            // println!(
            //     "Trying to read at address 0x{:04x} which is unimplemented",
            //     addr
            // );
            Ok(0)
            // Err(MemError::Unimplemented)
        }
    }

    pub fn w(&mut self, addr: u16, value: u8) -> Result<(), MemError> {
        if addr < 0x4000 {
            self.rom_00[addr as usize] = value;
            Ok(())
        } else if addr >= 0x8000 && addr < 0xa000 {
            self.vram[addr as usize - 0x8000] = value;
            Ok(())
        } else if addr >= 0xff00 && addr < 0xff80 {
            Ok(self.io.w((addr & 0xff) as u8, value))
        } else if addr >= 0xff80 && addr < 0xffff {
            self.hram[addr as usize - 0xff80] = value;
            Ok(())
        } else {
            // println!(
            //     "Trying to write at address 0x{:04x} which is unimplemented",
            //     addr
            // );
            Ok(())
            // Err(MemError::Unimplemented)
        }
    }
}

pub struct GBState {
    pub cpu: CPU,
    pub mem: Memory,
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
