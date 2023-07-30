pub mod audio;
pub mod consts;
pub mod display;
pub mod gamepad;
pub mod interrupts;
pub mod io;
pub mod opcodes;
pub mod state;
pub mod tests;

use crate::gamepad::Gamepad;
use crate::state::{GBState, MemError};
use std::env;
use std::time::SystemTime;
use std::{time, thread};

pub fn exec_opcode(state: &mut GBState) -> Result<u64, MemError> {
    let opcode = state.mem.r(state.cpu.pc)?;

    state.cpu.pc += 1;

    let n1 = (opcode >> 3) & 0b111;
    let n2 = opcode & 0b111;

    match opcode >> 6 {
        0b00 => opcodes::op00(state, n1, n2),
        0b01 => opcodes::op01(state, n1, n2),
        0b10 => opcodes::op10(state, n1, n2),
        0b11 => opcodes::op11(state, n1, n2),
        _ => panic!(),
    }
}

fn main() {
    if env::args().len() != 2 {
        println!("Usage: gameboy-emulator <rom.gb>");
        return;
    }

    let rom = env::args().nth(1);

    println!("Initializing Gamepad...");

    let mut gamepad = Gamepad::new();

    println!("Starting {:?}...", rom.clone().unwrap());

    let mut state = GBState::new();
    let mut cycles = 0;

    state.mem.load_rom(&rom.unwrap()).unwrap();
    let mut nanos_sleep: i128 = 0;

    loop {
        let now = SystemTime::now();
        let c = exec_opcode(&mut state).unwrap();

        if cycles >= 256 {

            gamepad.update_events();

            let action_button_reg = gamepad.get_action_gamepad_reg();
            let direction_button_reg = gamepad.get_direction_gamepad_reg();

            state.mem.joypad_reg = direction_button_reg | (action_button_reg << 4);

            state.mem.div += 1;

            cycles = 0;
        }
        cycles += c;

        let vblank_interrupt = state.mem.display.update_display(c);

        if vblank_interrupt {
            state.mem.io[0x0f] |= 1;
        }

        state.check_interrupts().unwrap();

        nanos_sleep += c as i128 * consts::CPU_CYCLE_LENGTH_NANOS as i128;

        if nanos_sleep > 0 {
            thread::sleep(time::Duration::from_nanos(
                nanos_sleep as u64,
            ));

            nanos_sleep = nanos_sleep - SystemTime::now()
             .duration_since(now)
             .unwrap()
             .as_nanos() as i128;
        }
    }
}
