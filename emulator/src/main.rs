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
use std::{thread, time};

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

    loop {
        let c = exec_opcode(&mut state).unwrap();

        // The OS scheduler is not precise enough to sleep at every iteration.
        // The workaround is to sleep every 10000 cycles and keep track of the
        // remaining cycles.
        if cycles >= 10000 {
            thread::sleep(time::Duration::from_nanos(
                cycles * consts::CPU_CYCLE_LENGTH_NANOS,
            ));
            gamepad.update_events();

            let action_button_reg = gamepad.get_action_gamepad_reg();
            let direction_button_reg = gamepad.get_direction_gamepad_reg();

            state.mem.joypad_reg = direction_button_reg | (action_button_reg << 4);

            cycles = 0;
        }
        cycles += c;

        let vblank_interrupt = state.mem.display.update_display(c);

        if vblank_interrupt {
            state.mem.io[0x0f] |= 1;
        }

        state.check_interrupts().unwrap();
    }
}
