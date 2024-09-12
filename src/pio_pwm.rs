// This code has now been included in the raspberry pi pico embassy examples.
use core::time::Duration;

use embassy_rp::clocks::{self};
use embassy_rp::pio::{self, Pin};
use embassy_rp::gpio::Level;
use ::pio::InstructionOperands;
use pio::{
    Common, Config, Instance, PioPin, StateMachine,
};

// This converts the duration provided into the number of cycles the PIO needs to run to make it take the same time
pub fn to_pio_cycles(duration: Duration) -> u32 {
    (clocks::clk_sys_freq() / 1_000_000) / 3 * duration.as_micros() as u32 // parentheses are required to prevent overflow
}

// Just a struct that stores the PIO state machine. Remember there are two PIO blocks, each with four state machines
// We are just running one program, so we just need one state machine
pub struct PwmPio<'d, T: Instance, const SM: usize> {
    sm: StateMachine<'d, T, SM>,
    pin: Pin<'d, T>
}

impl<'d, T: Instance, const SM: usize> PwmPio<'d, T, SM> {
    // Attach a servo to the specified pio, state machine, and pin.
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

        Self { sm, pin }
    }

    // Runs the servo input command, basically creating the PIO wave.
    pub fn start(&mut self) {
        self.sm.set_enable(true);
    }

    // Stops the servo input command, ceasing all signals from the PIN that were generated via PIO.
    pub fn stop(&mut self) {
        self.sm.set_enable(false);
    }

    // Set the duration of the period for the PWM wave, read more about pwm and servos here https://blog.wokwi.com/learn-servo-motor-using-wokwi-logic-analyzer/ 
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

    // Set the number of pio cycles to set the wave on high to.
    pub fn set_level(&mut self, level: u32) {
        self.sm.tx().push(level);
    }    

    // Set the duration for the wave to be high on.
    pub fn write(&mut self, duration: Duration) {
        self.set_level(to_pio_cycles(duration));
    }

    // Return the state machine and pin.
    pub fn release(self) -> (StateMachine<'d, T, SM>, Pin<'d, T>) {
        (self.sm, self.pin)
    }
}
