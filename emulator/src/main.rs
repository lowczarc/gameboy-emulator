pub mod audio;
pub mod consts;
pub mod display;
pub mod io;
pub mod opcodes;
pub mod state;
pub mod tests;

use crate::audio::AudioChannel;
use crate::consts::DISPLAY_UPDATE_SLEEP_TIME_MICROS;
use crate::state::{GBState, MemError};
use std::time::SystemTime;
use std::{thread, time};

pub fn exec_opcode(state: &mut GBState) -> Result<(), MemError> {
    let opcode = state.mem.r(state.cpu.pc)?;
    state.cpu.pc += 1;

    let n1 = (opcode >> 3) & 0b111;
    let n2 = opcode & 0b111;

    match opcode >> 6 {
        0b00 => opcodes::op00(state, n1, n2)?,
        0b01 => opcodes::op01(state, n1, n2)?,
        0b10 => opcodes::op10(state, n1, n2)?,
        0b11 => opcodes::op11(state, n1, n2)?,
        _ => panic!(),
    }

    Ok(())
}

fn main() {
    let mut state = GBState::new();
    state.mem.load_rom("/home/lancelot/tetris.bin").unwrap();

    let mut last_dt = SystemTime::now();
    loop {
        exec_opcode(&mut state).unwrap();
        if SystemTime::now()
            .duration_since(last_dt)
            .unwrap()
            .as_micros()
            > DISPLAY_UPDATE_SLEEP_TIME_MICROS as u128
        {
            state.update_display();
            last_dt = SystemTime::now();
        }
    }
}
