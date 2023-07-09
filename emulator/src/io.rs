#[derive(Debug)]
pub struct IORegisters([u8; 0x80]);

impl IORegisters {
    pub fn new() -> Self {
        Self([0; 0x80])
    }

    pub fn r(&self, addr: u8) -> u8 {
        match addr {
            0x00 => println!("READ Joypad"),
            0x11 => println!("READ Sound channel 1 length timer & duty cycle"),
            0x12 => println!("READ Sound channel 1 volume & envelope"),
            0x24 => println!("READ Master volume & VIN panning"),
            0x25 => println!("READ Sound panning"),
            0x26 => println!("READ Sound on/off"),
            0x42 => println!("READ Viewport Y Position"),
            0x44 => (), // println!("READ LCD Y Coordinate"),
            0x47 => println!("READ BG palette data"),
            _ => println!("Unknowned READ in IO register at address 0xff{:02x}", addr),
        }
        self.0[addr as usize]
    }

    pub fn w(&mut self, addr: u8, value: u8) {
        println!(
            "Trying to write 0b{:08b} in IO register at address 0xff{:02x}",
            value, addr
        );
        match addr {
            0x00 => println!("WRITE Joypad"),
            0x11 => println!("WRITE Sound channel 1 length timer & duty cycle"),
            0x12 => println!("WRITE Sound channel 1 volume & envelope"),
            0x24 => println!("WRITE Master volume & VIN panning"),
            0x25 => println!("WRITE Sound panning"),
            0x26 => println!("WRITE Sound on/off"),
            0x40 => println!("WRITE LCD Control"),
            0x42 => println!("WRITE Viewport Y Position"),
            0x47 => println!("WRITE BG palette data"),
            _ => println!("WRITE IDK Yet"),
        }
        self.0[addr as usize] = value
    }
}
