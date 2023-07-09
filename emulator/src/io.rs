use crate::state::Memory;

impl Memory {
    pub fn r_io(&self, addr: u8) -> u8 {
        match addr {
            0x00 => println!("READ Joypad"),
            0x11 => println!("READ Sound channel 1 length timer & duty cycle"),
            0x12 => println!("READ Sound channel 1 volume & envelope"),
            0x24 => println!("READ Master volume & VIN panning"),
            0x25 => println!("READ Sound panning"),
            0x26 => println!("READ Sound on/off"),
            0x40 => println!("READ LCD Control"),
            0x42 => (), // println!("READ Viewport Y Position"),
            0x43 => println!("READ Viewport X Position"),
            0x44 => (), // println!("READ LCD Y Coordinate"),
            0x47 => println!("READ BG palette data"),
            _ => println!("Unknowned READ in IO register at address 0xff{:02x}", addr),
        }

        match addr {
            0x40 => self.display.lcdc,
            0x42 => self.display.viewport_y,
            0x43 => self.display.viewport_x,
            0x44 => self.display.ly,
            0x47 => self.display.palette,
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

    pub fn w_io(&mut self, addr: u8, value: u8) {
        println!(
            "Trying to write 0b{:08b} in IO register at address 0xff{:02x}",
            value, addr
        );
        match addr {
            0x00 => println!("WRITE Joypad"),
            0x11 => println!("WRITE Sound channel 1 length timer & duty cycle"),
            0x12 => println!("WRITE Sound channel 1 volume & envelope"),
            0x13 => println!("WRITE Sound channel 1 period low"),
            0x14 => println!("WRITE Sound channel 1 period high & control"),
            0x16 => println!("WRITE Sound channel 2 length timer & duty cycle"),
            0x17 => println!("WRITE Sound channel 2 volume & envelope"),
            0x18 => println!("WRITE Sound channel 2 period low"),
            0x19 => println!("WRITE Sound channel 2 period high & control"),
            0x24 => println!("WRITE Master volume & VIN panning"),
            0x25 => println!("WRITE Sound panning"),
            0x26 => println!("WRITE Sound on/off"),
            0x40 => println!("WRITE LCD Control"),
            0x42 => (), // println!("WRITE Viewport Y Position"),
            0x43 => println!("WRITE Viewport X Position"),
            0x47 => println!("WRITE BG palette data"),
            0x50 => println!("WRITE BOOT LOCK"),
            _ => println!("WRITE IDK Yet"),
        }

        match addr {
            0x11 => {
                self.audio.ch1.duty = value >> 6;
                // TODO: Length timer
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
                // TODO: Length timer
            }
            0x17 => {
                self.audio.ch2.period_value &= 0xff00;
                self.audio.ch2.period_value |= value as u16;
            }
            0x18 => {
                self.audio.ch2.period_value &= 0xff;
                self.audio.ch2.period_value |= ((value & 0b111) as u16) << 8;
                if value >> 7 == 1 {
                    self.audio.ch1.update();
                }
            }
            0x40 => self.display.lcdc = value,
            0x42 => self.display.viewport_y = value,
            0x43 => self.display.viewport_x = value,
            0x47 => self.display.palette = value,
            0x50 => self.boot_rom_on = value & 1 == 0 && self.boot_rom_on,
            _ => self.io[addr as usize] = value,
        }
    }
}
