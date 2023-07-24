use crate::consts::DISPLAY_UPDATE_SLEEP_TIME_MICROS;
use crate::state::MemError;
use minifb::{Window, WindowOptions};
use std::time::SystemTime;

const COLORS: [u32; 4] = [0x00e0f8d0, 0x0088c070, 0x346856, 0x00081820];

const LINE_DOTS: u64 = 456;

mod lcdc_flags {
    pub const BG_PRIORITY: u8 = 0b1;
    pub const OBJ_ENABLE: u8 = 0b10;
    pub const OBJ_SIZE: u8 = 0b100;
    pub const BG_TILEMAP_AREA: u8 = 0b1000;
    pub const BG_TILEDATA_AREA: u8 = 0b10000;
    pub const WIN_ENABLE: u8 = 0b100000;
    pub const WIN_TILEMAP_AREA: u8 = 0b1000000;
    pub const LCD_ENABLE: u8 = 0b10000000;
}

#[derive(Debug)]
pub struct Display {
    window: Window,
    framebuffer: [u32; 160 * 144],

    tiledata: [u8; 0x1800],
    tilemaps: [u8; 0x800],
    oam: [u8; 0xa0],

    pub bg_palette: u8,
    pub obj_palettes: [u8; 2],
    pub viewport_y: u8,
    pub viewport_x: u8,
    pub lcdc: u8,
    pub ly: u8,

    last_dt: SystemTime,

    stat: u64,
}

impl Display {
    pub fn new() -> Self {
        Self {
            window: Window::new("Gameboy Emulator", 512, 461, WindowOptions::default()).unwrap(),
            framebuffer: [0; 160 * 144],
            tiledata: [0; 0x1800],
            tilemaps: [0; 0x800],
            oam: [0; 0xa0],
            bg_palette: 0,
            obj_palettes: [0; 2],
            viewport_y: 0,
            viewport_x: 0,
            lcdc: 0,
            ly: 0,
            last_dt: SystemTime::now(),
            stat: 0,
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

    pub fn color_palette(&self, color_byte: u8, palette: u8) -> u32 {
        COLORS[((palette >> (color_byte << 1)) & 0b11) as usize]
    }

    pub fn print_tile(&mut self, tile: u8, x: u8, y: u8, l: usize) {
        let tile_pointer = if self.lcdc & lcdc_flags::BG_TILEDATA_AREA != 0 {
            ((tile as u16) << 4) as usize
        } else {
            ((tile as u16) << 4) as usize + 0x800
        };
        for b in (0..8).rev() {
            let data = (((self.tiledata[tile_pointer + l * 2] as u8) >> b) & 1)
                | ((((self.tiledata[tile_pointer + l * 2 + 1] as u8) >> b) & 1) << 1);

            let pxx = x as i32 * 8 + 7 - b as i32 - self.viewport_x as i32;
            let pxy = ((y as i32 * 8) + l as i32) - self.viewport_y as i32;

            if pxy < 144 && pxx < 160 && pxy >= 0 && pxx >= 0 {
                self.framebuffer[pxy as usize * 160 + pxx as usize] =
                    self.color_palette(data, self.bg_palette);
            }
        }
    }
    pub fn print_all_tiles(&mut self) {
        for i in 0..=255 {
            for l in 0..8 {
                self.print_tile(i, i % 20, i / 20, l);
            }
        }
    }

    pub fn w(&mut self, addr: u16, value: u8) -> Result<(), MemError> {
        if addr < 0x1800 {
            self.tiledata[addr as usize] = value;
        } else if addr >= 0x7e00 {
            self.oam[addr as usize - 0x7e00] = value;
        } else {
            self.tilemaps[addr as usize - 0x1800] = value;
        }
        Ok(())
    }

    pub fn r(&self, addr: u16) -> Result<u8, MemError> {
        if addr < 0x1800 {
            Ok(self.tiledata[addr as usize])
        } else if addr >= 0x7e00 {
            Ok(self.oam[addr as usize - 0x7e00])
        } else {
            Ok(self.tilemaps[addr as usize - 0x1800])
        }
    }

    pub fn print_bg(&mut self) {
        let tilemap_pointer = if self.lcdc & lcdc_flags::BG_TILEMAP_AREA != 0 {
            0x400
        } else {
            0
        };

        let y = (self.ly / 8) as usize;

        for x in 0..32 {
            let tile = self.tilemaps[tilemap_pointer + y * 32 + x];
            self.print_tile(tile, x as u8, y as u8, (self.ly % 8) as usize);
        }
    }

    pub fn print_win(&mut self) {
        if self.lcdc & lcdc_flags::WIN_ENABLE == 0 {
            return;
        }
        println!("PRINT WIN");

        let tilemap_pointer = if self.lcdc & lcdc_flags::WIN_TILEMAP_AREA != 0 {
            0x400
        } else {
            0
        };

        let y = (self.ly / 8) as usize;

        for x in 0..32 {
            let tile = self.tilemaps[tilemap_pointer + y * 32 + x];
            self.print_tile(tile, x as u8, y as u8, (self.ly % 8) as usize);
        }
    }

    pub fn print_obj(&mut self) {
        if self.lcdc & lcdc_flags::OBJ_ENABLE == 0 {
            return;
        }

        for o in 0..40 {
            let y = self.oam[o * 4];
            let x = self.oam[o * 4 + 1];
            let tile = self.oam[o * 4 + 2];
            let opts = self.oam[o * 4 + 3];
            let tile_pointer = ((tile as u16) << 4) as usize;

            if y < self.ly || y >= self.ly + 8 {
                continue;
            }

            let l = y - self.ly;

            for b in (0..8).rev() {
                let pxx = x as i32 + 7 - b as i32 - 8;
                let pxy = self.ly as i32 - 8;

                let data = (((self.tiledata[tile_pointer + l as usize * 2] as u8) >> b) & 1)
                    | ((((self.tiledata[tile_pointer + l as usize * 2 + 1] as u8) >> b) & 1) << 1);

                if pxy < 144 && pxx < 160 && pxy >= 0 && pxx >= 0 {
                    self.framebuffer[pxy as usize * 160 + pxx as usize] =
                        self.color_palette(data, self.bg_palette);
                }
            }
        }
    }

    pub fn update_display(&mut self, cycles: u64) -> bool {
        let mut vblank_interrupt = false;
        self.stat += cycles;
        if self.lcdc & lcdc_flags::LCD_ENABLE != 0 && self.stat >= LINE_DOTS {
            self.print_bg();
            self.print_win();
            self.print_obj();
            self.ly = (self.ly + 1) % 154;
            self.stat %= LINE_DOTS;
            if self.ly == 0x90
                && SystemTime::now()
                    .duration_since(self.last_dt)
                    .unwrap()
                    .as_micros()
                    > DISPLAY_UPDATE_SLEEP_TIME_MICROS as u128
            {
                self.update();
                vblank_interrupt = true;
                self.last_dt = SystemTime::now();
            }
        }
        if self.lcdc & lcdc_flags::LCD_ENABLE == 0 {
            self.ly = 0;
        }

        return vblank_interrupt;
    }
}
