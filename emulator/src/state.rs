use crate::audio::Audio;
use crate::consts::{BOOT_ROM_FILE, PROGRAM_START_ADDRESS, STACK_START_ADDRESS};
use crate::display::Display;
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
            flag::NC => (f >> 4) & 1 == 0,
            flag::C => (f >> 4) & 1 == 1,
            _ => unimplemented!(),
        }
    }
}

pub struct Memory {
    boot_rom: [u8; 0x100],
    pub boot_rom_on: bool,

    // 32 KiB ROM bank 00
    rom: [u8; 0x8000],

    // 4 KiB Work RAM 00
    wram_00: [u8; 0x1000],

    // 4 KiB Work RAM 00
    wram_01: [u8; 0x1000],

    // 8 KiB Video RAM
    pub display: Display,

    pub io: [u8; 0x80],

    // High RAM
    hram: [u8; 0x7f],

    pub audio: Audio,

    pub ime: bool,

    pub div: u8,

    pub joypad_reg: u8,

    pub joypad_is_action: bool,

    pub interrupts_register: u8,

    pub halt: bool,
}

#[derive(Debug)]
pub enum MemError {
    WritingToROM,
    Unimplemented,
    NotUsable,
}

impl Memory {
    pub fn new() -> Self {
        let mut display = Display::new();

        display.cls();

        Self {
            boot_rom: [0; 0x100],
            boot_rom_on: true,
            rom: [0; 0x8000],
            wram_00: [0; 0x1000],
            wram_01: [0; 0x1000],
            display,
            io: [0; 0x80],
            hram: [0; 0x7f],
            audio: Audio::new(),
            ime: false,
            interrupts_register: 0,
            joypad_is_action: false,
            joypad_reg: 0,
            div: 0,
            halt: false,
        }
    }

    pub fn load_boot_rom(&mut self) -> Result<(), std::io::Error> {
        let mut f = File::open(BOOT_ROM_FILE)?;

        f.read(&mut self.boot_rom)?;

        Ok(())
    }

    pub fn load_rom(&mut self, file: &str) -> Result<(), std::io::Error> {
        let mut f = File::open(file)?;

        f.read(&mut self.rom)?;

        Ok(())
    }

    pub fn r(&self, addr: u16) -> Result<u8, MemError> {
        if addr < 0x100 && self.boot_rom_on {
            Ok(self.boot_rom[addr as usize])
        } else if addr < 0x8000 {
            Ok(self.rom[addr as usize])
        } else if addr >= 0xc000 && addr < 0xd000 {
            Ok(self.wram_00[addr as usize - 0xc000])
        } else if addr >= 0xe000 && addr < 0xf000 {
            Ok(self.wram_00[addr as usize - 0xe000])
        } else if addr >= 0xd000 && addr < 0xe000 {
            Ok(self.wram_01[addr as usize - 0xd000])
        } else if (addr >= 0x8000 && addr < 0xa000) || (addr >= 0xfe00 && addr < 0xfea0) {
            self.display.r(addr & !0x8000)
        } else if addr >= 0xff00 && addr < 0xff80 {
            Ok(self.r_io((addr & 0xff) as u8))
        } else if addr >= 0xff80 && addr < 0xffff {
            Ok(self.hram[addr as usize - 0xff80])
        } else if addr == 0xffff {
            Ok(self.interrupts_register)
        } else {
            // println!(
            //     "Trying to read at address 0x{:04x} which is unimplemented",
            //     addr
            // );
            Ok(0) //Err(MemError::Unimplemented)
        }
    }

    pub fn w(&mut self, addr: u16, value: u8) -> Result<(), MemError> {
        if addr < 0x100 && self.boot_rom_on {
            Ok(())
        } else if addr < 0x8000 {
            Ok(())
        } else if addr >= 0xc000 && addr < 0xd000 {
            self.wram_00[addr as usize - 0xc000] = value;
            Ok(())
        } else if addr >= 0xe000 && addr < 0xf000 {
            self.wram_00[addr as usize - 0xe000] = value;
            Ok(())
        } else if addr >= 0xd000 && addr < 0xe000 {
            self.wram_01[addr as usize - 0xd000] = value;
            Ok(())
        } else if (addr >= 0x8000 && addr < 0xa000) || (addr >= 0xfe00 && addr < 0xfea0) {
            self.display.w(addr & !0x8000, value)
        } else if addr >= 0xff00 && addr < 0xff80 {
            Ok(self.w_io((addr & 0xff) as u8, value)?)
        } else if addr >= 0xff80 && addr < 0xffff {
            self.hram[addr as usize - 0xff80] = value;
            Ok(())
        } else if addr == 0xffff {
            self.interrupts_register = value;
            Ok(())
        } else {
            // println!(
            //     "Trying to write at address 0x{:04x} which is unimplemented",
            //     addr
            // );
            Ok(()) //Err(MemError::Unimplemented)
        }
    }
}

pub struct GBState {
    pub cpu: CPU,
    pub mem: Memory,
    pub is_debug: bool,
}

impl GBState {
    pub fn new() -> Self {
        let mut mem = Memory::new();

        mem.load_boot_rom().unwrap();

        Self {
            cpu: CPU::new(),
            mem,
            is_debug: false,
        }
    }

    pub fn r_reg(&self, r_i: u8) -> Result<u8, MemError> {
        if r_i < 6 {
            Ok(self.cpu.r[r_i as usize])
        } else if r_i == 7 {
            Ok(self.cpu.r[6])
        } else if r_i == 6 {
            self.mem.r(self.cpu.r16(reg::HL))
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
            self.mem.w(self.cpu.r16(reg::HL), value)?;
        } else {
            panic!("r_i must be a 3 bits register input number")
        }
        Ok(())
    }

    pub fn debug(&self, s: &str) -> () {
        if self.is_debug {
            println!("{}", s);
        }
    }
}
