use crate::opcodes;
use crate::state::{GBState, MemError};

impl GBState {
    pub fn check_interrupts(&mut self) -> Result<(), MemError> {
        if self.mem.ime {
            let interrupts = self.mem.io[0x0f] & self.mem.interrupts_register;
            if interrupts & 1 == 1 {
                opcodes::push(self, self.cpu.pc)?;

                self.mem.ime = false;
                self.cpu.pc = 0x40;

                self.mem.io[0x0f] &= !1;
            }
        }
        Ok(())
    }
}
