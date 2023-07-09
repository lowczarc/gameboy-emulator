pub const BOOT_ROM_FILE: &str = "./assets/boot.bin";
pub const PROGRAM_START_ADDRESS: u16 = 0x0;
pub const STACK_START_ADDRESS: u16 = 0x0;

pub const DISPLAY_UPDATE_RATE: u64 = 60; // Hertz
pub const DISPLAY_UPDATE_SLEEP_TIME_MICROS: u64 = 1000000 / DISPLAY_UPDATE_RATE;
