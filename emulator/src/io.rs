use crate::state::{MemError, Memory};

impl Memory {
    pub fn r_io(&self, addr: u8) -> u8 {
        // match addr {
        //     0x00 => println!("READ Joypad"),
        //     0x0f => println!("Write Interrupt flag"),
        //     0x11 => println!("READ Sound channel 1 length timer & duty cycle"),
        //     0x12 => println!("READ Sound channel 1 volume & envelope"),
        //     0x24 => println!("READ Master volume & VIN panning"),
        //     0x25 => println!("READ Sound panning"),
        //     0x26 => println!("READ Sound on/off"),
        //     0x40 => println!("READ LCD Control"),
        //     0x42 => (), // println!("READ Viewport Y Position"),
        //     0x43 => println!("READ Viewport X Position"),
        //     0x44 => (), // println!("READ LCD Y Coordinate ({})", self.display.ly),
        //     0x47 => println!("READ BG palette data"),
        //     0xff => println!("READ Interrupt enable"),
        //     _ => println!("Unknowned READ in IO register at address 0xff{:02x}", addr),
        // }

        match addr {
            0x00 => {
                if self.joypad_is_action {
                    (self.joypad_reg >> 4) | 0b11010000
                } else {
                    (self.joypad_reg & 0xf) | 0b11100000
                }
            }
            0x04 => self.div,
            0x40 => self.display.lcdc,
            0x42 => self.display.viewport_y,
            0x43 => self.display.viewport_x,
            0x44 => self.display.ly,
            0x47 => self.display.bg_palette,
            0x48 => self.display.obj_palettes[0],
            0x49 => self.display.obj_palettes[1],
            0x50 => {
                if self.boot_rom_on {
                    0xfe
                } else {
                    0xff
                }
            }
            _ => self.io[addr as usize],
        }
    }

    pub fn w_io(&mut self, addr: u8, value: u8) -> Result<(), MemError> {
        // println!(
        //     "Trying to write 0b{:08b} in IO register at address 0xff{:02x}",
        //     value, addr
        // );
        // match addr {
        //     0x00 => println!("WRITE Joypad"),
        //     0x0f => println!("Write Interrupt flag"),
        //     0x11 => println!("WRITE Sound channel 1 length timer & duty cycle"),
        //     0x12 => println!("WRITE Sound channel 1 volume & envelope"),
        //     0x13 => println!("WRITE Sound channel 1 period low"),
        //     0x14 => println!("WRITE Sound channel 1 period high & control"),
        //     0x16 => println!("WRITE Sound channel 2 length timer & duty cycle"),
        //     0x17 => println!("WRITE Sound channel 2 volume & envelope"),
        //     0x18 => println!("WRITE Sound channel 2 period low"),
        //     0x19 => println!("WRITE Sound channel 2 period high & control"),
        //     0x24 => println!("WRITE Master volume & VIN panning"),
        //     0x25 => println!("WRITE Sound panning"),
        //     0x26 => println!("WRITE Sound on/off"),
        //     0x40 => println!("WRITE LCD Control"),
        //     0x42 => (), // println!("WRITE Viewport Y Position"),
        //     0x43 => println!("WRITE Viewport X Position"),
        //     0x47 => println!("WRITE BG palette data"),
        //     0x50 => println!("WRITE Unmount boot ROM "),
        //     0xff => println!("Write Interrupt enable"),
        //     _ => println!("WRITE IDK Yet"),
        // }

        match addr {
            0x00 => {
                self.joypad_is_action = !value & 0b00100000 != 0;
            }
            0x04 => {
                self.div = 0;
            }
            0x11 => {
                self.audio.ch1.duty = value >> 6;
                if value & 0b111111 != 0 {
                    self.audio.ch1.length_timer = Some(value & 0b111111);
                } else {
                    self.audio.ch1.length_timer = None;
                }
            }
            0x12 => {
                self.audio.ch1.initial_volume = value >> 4;
                self.audio.ch1.env_direction = (value & 0xf) >> 3;
                self.audio.ch1.sweep = value & 0b111;
            }
            0x13 => {
                self.audio.ch1.period_value &= 0xff00;
                self.audio.ch1.period_value |= value as u16;
            }
            0x14 => {
                self.audio.ch1.period_value &= 0xff;
                self.audio.ch1.period_value |= ((value & 0b111) as u16) << 8;
                if value >> 7 == 1 {
                    self.audio.ch1.update();
                }
            }
            0x16 => {
                self.audio.ch2.duty = value >> 6;
                if value & 0b111111 != 0 {
                    self.audio.ch2.length_timer = Some(value & 0b111111);
                } else {
                    self.audio.ch2.length_timer = None;
                }
            }
            0x17 => {
                self.audio.ch2.initial_volume = value >> 4;
                self.audio.ch2.env_direction = (value & 0xf) >> 3;
                self.audio.ch2.sweep = value & 0b111;
            }
            0x18 => {
                self.audio.ch2.period_value &= 0xff00;
                self.audio.ch2.period_value |= value as u16;
            }
            0x19 => {
                self.audio.ch2.period_value &= 0xff;
                self.audio.ch2.period_value |= ((value & 0b111) as u16) << 8;
                if value >> 7 == 1 {
                    self.audio.ch2.update();
                }
            }
            0x1b => {
                if value < 64 {
                    self.audio.ch3.length_timer = Some(value);
                } else {
                    self.audio.ch3.length_timer = None;
                }
            }
            0x1c => {
                let s = (value >> 5) & 0b11;
                if s == 0 {
                    self.audio.ch3.initial_volume = 0;
                } else {
                    self.audio.ch3.initial_volume = 0xf >> (s - 1);
                }
            }
            0x1d => {
                self.audio.ch3.period_value &= 0xff00;
                self.audio.ch3.period_value |= value as u16;
            }
            0x1e => {
                self.audio.ch3.period_value &= 0xff;
                self.audio.ch3.period_value |= ((value & 0b111) as u16) << 8;
                self.audio.ch3.period_value /= 2;
                if value >> 7 == 1 {
                    self.audio.ch3.update();
                }
            }
            0x40 => self.display.lcdc = value,
            0x42 => self.display.viewport_y = value,
            0x43 => self.display.viewport_x = value,
            0x46 => {
                if value < 0xe0 {
                    let addr = (value as u16) << 8;

                    for i in 0..0xa0 {
                        self.w(0xfe00 | i, self.r(addr | i)?)?;
                    }
                }
            }
            0x47 => self.display.bg_palette = value,
            0x48 => self.display.obj_palettes[0] = value,
            0x49 => self.display.obj_palettes[1] = value,
            0x50 => self.boot_rom_on = value & 1 == 0 && self.boot_rom_on,
            _ => self.io[addr as usize] = value,
        }

        if (addr >= 0x30 && addr <= 0x3f) {
            let i = (addr - 0x30) as usize;
            self.audio.ch3.wave_pattern[i * 2] = value >> 4;
            self.audio.ch3.wave_pattern[i * 2 + 1] = value & 0xf;
        }

        Ok(())
    }
}
