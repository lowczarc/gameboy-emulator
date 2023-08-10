pub const BOOT_ROM_FILE: &str = "./assets/boot.bin";
pub const PROGRAM_START_ADDRESS: u16 = 0x0;
pub const STACK_START_ADDRESS: u16 = 0x0;

pub const SPEEDUP_FACTOR: f64 = 1.0;

pub const DISPLAY_UPDATE_RATE: u64 = 30; // Hertz
pub const DISPLAY_UPDATE_SLEEP_TIME_MICROS: u64 =
    ((1000000 / DISPLAY_UPDATE_RATE) as f64 / SPEEDUP_FACTOR) as u64;

pub const CPU_CLOCK_SPEED: u64 = 4_194_304;
pub const CPU_CYCLE_LENGTH_NANOS: u64 =
    ((1_000_000_000 / CPU_CLOCK_SPEED) as f64 / SPEEDUP_FACTOR) as u64;
