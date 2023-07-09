use crate::state::MemError;
use minifb::{Window, WindowOptions};

const colors: [u32; 4] = [0x00081820, 0x346856, 0x0088c070, 0x00e0f8d0];

#[derive(Debug)]
pub struct Display {
    window: Window,
    framebuffer: [u32; 160 * 144],

    tiledata: [u8; 0x1800],
    tilemaps: [u8; 0x800],
}

impl Display {
    pub fn new() -> Self {
        Self {
            window: Window::new("Gameboy Emulator", 512, 461, WindowOptions::default()).unwrap(),
            framebuffer: [0; 160 * 144],
            tiledata: [0; 0x1800],
            tilemaps: [0; 0x800],
        }
    }

    pub fn cls(&mut self) {
        self.framebuffer = [colors[0]; 160 * 144];
    }

    pub fn update(&mut self) {
        self.window
            .update_with_buffer(&self.framebuffer, 160, 144)
            .unwrap();
    }

    pub fn print_tile(&mut self, tile: u8, x: u8, y: u8) {
        let tile_pointer = ((tile as u16) << 4) as usize;
        for i in 0..8 {
            for b in (0..8).rev() {
                let data = (((self.tiledata[tile_pointer + i * 2] as u16) >> b) & 1)
                    | ((((self.tiledata[tile_pointer + i * 2 + 1] as u16) >> b) & 1) << 1);
                self.framebuffer[((y * 8) as usize + i) * 160 + (x * 8) as usize + (7 - b)] =
                    colors[data as usize];
            }
        }
    }
    pub fn print_all_tiles(&mut self) {
        for i in 0..=255 {
            self.print_tile(i, i % 20, i / 20);
        }
    }

    pub fn w(&mut self, addr: u16, value: u8) -> Result<(), MemError> {
        println!("{:04x} {:02x}", addr, value);
        if addr < 0x1800 {
            self.tiledata[addr as usize] = value;
        } else {
            self.tilemaps[addr as usize - 0x1800] = value;
        }
        Ok(())
    }

    pub fn r(&self, addr: u16) -> Result<u8, MemError> {
        if addr < 0x1800 {
            Ok(self.tiledata[addr as usize])
        } else {
            Ok(self.tilemaps[addr as usize - 0x1800])
        }
    }
}
