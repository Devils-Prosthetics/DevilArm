use core::time::Duration;

use embassy_rp::clocks::{self};
use embassy_rp::pio::{self};
use embassy_rp::gpio::Level;
use ::pio::InstructionOperands;
use pio::{
    Common, Config, Instance, PioPin, StateMachine,
};

pub fn to_pio_cycles(duration: Duration) -> u32 {
    (clocks::clk_sys_freq() / 1_000_000) / 3 * duration.as_micros() as u32 // parentheses are required to prevent overflow
}

pub struct PwmPio<'d, T: Instance, const SM: usize> {
    sm: StateMachine<'d, T, SM>,
}

impl<'d, T: Instance, const SM: usize> PwmPio<'d, T, SM> {
    pub fn attach(
        pio: &mut Common<'d, T>,
        mut sm: StateMachine<'d, T, SM>,
        pin: impl PioPin,
    ) -> Self {
        let prg = pio_proc::pio_asm!(
            ".side_set 1 opt"
                "pull noblock    side 0"
                "mov x, osr"
                "mov y, isr"
            "countloop:"
                "jmp x!=y noset"
                "jmp skip        side 1"
            "noset:"
                "nop"
            "skip:"
                "jmp y-- countloop"
        );

        pio.load_program(&prg.program);
        let pin = pio.make_pio_pin(pin);
        sm.set_pins(Level::High, &[&pin]);
        sm.set_pin_dirs(pio::Direction::Out, &[&pin]);

        let mut cfg = Config::default();
        cfg.use_program(&pio.load_program(&prg.program), &[&pin]);

        sm.set_config(&cfg);

        Self { sm }
    }

    pub fn start(&mut self) {
        self.sm.set_enable(true);
    }

    pub fn stop(&mut self) {
        self.sm.set_enable(false);
    }

    pub fn set_period(&mut self, duration: Duration) {
        let is_enabled = self.sm.is_enabled();
        while !self.sm.tx().empty() {}
        self.sm.set_enable(false);
        self.sm.tx().push(to_pio_cycles(duration));
        unsafe {
            self.sm.exec_instr(InstructionOperands::PULL { if_empty: false, block: false }.encode());
            self.sm.exec_instr(InstructionOperands::OUT { destination: ::pio::OutDestination::ISR, bit_count: 32 }.encode());
        };
        if is_enabled {
            self.sm.set_enable(true)
        }
    }

    pub fn set_level(&mut self, level: u32) {
        self.sm.tx().push(level);
    }    

    pub fn write(&mut self, duration: Duration) {
        self.set_level(to_pio_cycles(duration));
    }


}
