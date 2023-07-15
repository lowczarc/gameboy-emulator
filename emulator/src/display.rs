use crate::consts::DISPLAY_UPDATE_SLEEP_TIME_MICROS;
use crate::state::MemError;
use minifb::{Window, WindowOptions};
use std::time::SystemTime;

const COLORS: [u32; 4] = [0x00e0f8d0, 0x0088c070, 0x346856, 0x00081820];

#[derive(Debug)]
pub struct Display {
    window: Window,
    framebuffer: [u32; 160 * 144],

    tiledata: [u8; 0x1800],
    tilemaps: [u8; 0x800],
    pub palette: u8,
    pub viewport_y: u8,
    pub viewport_x: u8,
    pub lcdc: u8,
    pub ly: u8,
    last_dt: SystemTime,
    last_udt: SystemTime,
}

impl Display {
    pub fn new() -> Self {
        Self {
            window: Window::new("Gameboy Emulator", 512, 461, WindowOptions::default()).unwrap(),
            framebuffer: [0; 160 * 144],
            tiledata: [0; 0x1800],
            tilemaps: [0; 0x800],
            palette: 0,
            viewport_y: 0,
            viewport_x: 0,
            lcdc: 0,
            ly: 0,
            last_dt: SystemTime::now(),
            last_udt: SystemTime::now(),
        }
    }

    pub fn cls(&mut self) {
        self.framebuffer = [COLORS[0]; 160 * 144];
    }

    pub fn update(&mut self) {
        self.window
            .update_with_buffer(&self.framebuffer, 160, 144)
            .unwrap();
    }

    pub fn color_palette(&self, color_byte: u8) -> u32 {
        COLORS[((self.palette >> (color_byte << 1)) & 0b11) as usize]
    }

    pub fn print_tile(&mut self, tile: u8, x: u8, y: u8, i: usize) {
        let tile_pointer = if (self.lcdc >> 4) & 1 == 1 {
            ((tile as u16) << 4) as usize
        } else {
            ((tile as u16) << 4) as usize + 0x800
        };
        for b in (0..8).rev() {
            let data = (((self.tiledata[tile_pointer + i * 2] as u8) >> b) & 1)
                | ((((self.tiledata[tile_pointer + i * 2 + 1] as u8) >> b) & 1) << 1);

            let pxx = x as i32 * 8 + 7 - b as i32 - self.viewport_x as i32;
            let pxy = ((y as i32 * 8) + i as i32) - self.viewport_y as i32;

            if pxy < 144 && pxx < 160 && pxy >= 0 && pxx >= 0 {
                self.framebuffer[pxy as usize * 160 + pxx as usize] = self.color_palette(data);
            }
        }
    }
    pub fn print_all_tiles(&mut self) {
        for i in 0..=255 {
            for b in 0..8 {
                self.print_tile(i, i % 20, i / 20, b);
            }
        }
    }

    pub fn w(&mut self, addr: u16, value: u8) -> Result<(), MemError> {
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

    pub fn print_tile_map1(&mut self) {
        let tilemap_pointer = if (self.lcdc >> 3) & 1 == 1 { 0x400 } else { 0 };
        let y = (self.ly / 8) as usize;
        for x in 0..32 {
            let tile = self.tilemaps[tilemap_pointer + y * 32 + x];
            self.print_tile(tile, x as u8, y as u8, (self.ly % 8) as usize);
        }
        self.ly = (self.ly + 1) % 154;
    }

    pub fn update_display(&mut self) {
        if self.lcdc & 0b10000000 != 0 {
            self.print_tile_map1();
            self.last_udt = SystemTime::now();
            if self.ly == 153 {
                // self.cls();
                if SystemTime::now()
                    .duration_since(self.last_dt)
                    .unwrap()
                    .as_micros()
                    > DISPLAY_UPDATE_SLEEP_TIME_MICROS as u128
                {
                    self.update();
                    self.last_dt = SystemTime::now();
                }
            }
        }
    }
}
