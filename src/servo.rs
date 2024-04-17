
use core::time::Duration;

use embassy_rp::clocks::{self};

use embassy_rp::pio::{self, ShiftConfig, ShiftDirection};
use embassy_rp::pio::Irq;
use embassy_rp::gpio::Level;
use fixed::*;
use log::*;
use ::pio::InstructionOperands;
use pio::{
    Common, Config, FifoJoin, Instance, PioPin, StateMachine,
};
use crate::pio_pwm::PwmPio;

use {defmt_rtt as _, panic_probe as _};

const DEFAULT_MIN_PULSE_WIDTH: u64 = 1000; // uncalibrated default, the shortest duty cycle sent to a servo
const DEFAULT_MAX_PULSE_WIDTH: u64 = 2000; // uncalibrated default, the longest duty cycle sent to a servo 
const DEFAULT_NEUTRAL_PULSE_WIDTH: u64 = 1500; // default duty cycle when servo is attached
const REFRESH_INTERVAL: u64 = 20000; // classic default period to refresh servos in microseconds 
const DEFAULT_MAX_DEGREE_ROTATION: u64 = 180; // 180 degrees is typical


// pub struct Pwm

pub struct ServoBuilder<'d, T: Instance, const SM: usize> {
    pwm: PwmPio<'d, T, SM>,
    period: Duration,
    min_pulse_width: Duration,
    max_pulse_width: Duration,
    neutral_pulse_width: Duration,
    max_degree_rotation: u64,
}

impl<'d, T: Instance, const SM: usize> ServoBuilder<'d, T, SM> {
    pub fn new(pwm: PwmPio<'d, T, SM>) -> Self {
        Self {
            pwm,
            period: Duration::from_micros(REFRESH_INTERVAL),
            min_pulse_width: Duration::from_micros(DEFAULT_MIN_PULSE_WIDTH),
            max_pulse_width: Duration::from_micros(DEFAULT_MAX_PULSE_WIDTH),
            neutral_pulse_width: Duration::from_micros((DEFAULT_MAX_PULSE_WIDTH + DEFAULT_MIN_PULSE_WIDTH) / 2),
            max_degree_rotation: DEFAULT_MAX_DEGREE_ROTATION
        }
    }

    pub fn set_period(mut self, duration: Duration) -> Self {
        self.period = duration;
        self
    }

    pub fn set_min_pulse_width(mut self, duration: Duration) -> Self {
        self.min_pulse_width = duration;
        self
    }

    pub fn set_max_pulse_width(mut self, duration: Duration) -> Self {
        self.max_pulse_width = duration;
        self
    }

    pub fn set_neutral_pulse_width(mut self, duration: Duration) -> Self {
        self.neutral_pulse_width = duration;
        self
    }

    pub fn set_max_degree_rotation(mut self, degree: u64) -> Self {
        self.max_degree_rotation = degree;
        self
    }

    pub fn build(mut self) -> Servo<'d, T, SM> {
        self.pwm.set_period(self.period);
        Servo {
            pwm: self.pwm,
            min_pulse_width: self.min_pulse_width,
            max_pulse_width: self.max_pulse_width,
            neutral_pulse_width: self.neutral_pulse_width,
            max_degree_rotation: self.max_degree_rotation,
        }
    }
}

pub struct Servo<'d, T: Instance, const SM: usize> {
    pwm: PwmPio<'d, T, SM>,
    min_pulse_width: Duration,
    max_pulse_width: Duration,
    neutral_pulse_width: Duration,
    max_degree_rotation: u64,
}

impl<'d, T: Instance, const SM: usize> Servo<'d, T, SM> {
    pub fn start(&mut self) {
        self.pwm.start();
    }

    pub fn stop(&mut self) {
        self.pwm.stop();
    }

    pub fn write_time(&mut self, duration: Duration) {
        self.pwm.write(duration);
    }

    pub fn rotate(&mut self, degree: u64) {
        let degree_per_nano_second = (self.max_pulse_width.as_nanos() as u64 - self.min_pulse_width.as_nanos() as u64) / self.max_degree_rotation;
        let mut duration = Duration::from_nanos(degree * degree_per_nano_second + self.min_pulse_width.as_nanos() as u64);

        info!("duration first move: {}", duration.as_micros());
        if self.max_pulse_width < duration {
            duration = self.max_pulse_width;
        }

        info!("duration move: {}", duration.as_micros());

        self.pwm.write(duration);
    }
}






