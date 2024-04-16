use embassy_rp::pwm::{Config as PwmConfig, Pwm, Slice};
use fixed::traits::ToFixed;

const SERVO_DIV_INT: u8 = 250;
const SERVO_TOP: u16 = 10000;

const SERVO_CENTER_DUTY: u16 = 700;
const SERVO_MAX_DELTA_DUTY: u16 = 350;
const SERVO_MAX_DUTY: u16 = SERVO_CENTER_DUTY + SERVO_MAX_DELTA_DUTY;
const SERVO_MIN_DUTY: u16 = SERVO_CENTER_DUTY - SERVO_MAX_DELTA_DUTY;

pub struct Servo<'a, T: Slice> {
	div: u8,
	top: u16,
	center_duty: i16,
	max_delta_duty: u16,
	max_duty: u16,
	min_duty: u16,
	pwm: Pwm<'a, T>
}

impl<'a, T: Slice> Servo<'a, T> {
	pub fn default(pwm: Pwm<'a, T>) -> Servo<T> {
		Servo { 
			div: SERVO_DIV_INT,
			top: SERVO_TOP,
			center_duty: SERVO_CENTER_DUTY as i16,
			max_delta_duty: SERVO_MAX_DELTA_DUTY,
			max_duty: SERVO_MAX_DUTY,
			min_duty: SERVO_MIN_DUTY,
			pwm
		}
	}

	fn create_config(&self, steer: i16) -> PwmConfig {
		let duty_b = (((-steer * 10) + self.center_duty) as u16)
			.min(self.max_duty)
			.max(self.min_duty);

		let mut c = PwmConfig::default();
		c.invert_a = false;
		c.invert_b = false;
		c.phase_correct = false;
		c.enable = true;
		c.divider = self.div.to_fixed();
		c.compare_a = 0;
		c.compare_b = duty_b;
		c.top = self.top;

		c
	}

	pub fn move_servo(&mut self, steer: i16) {
		let config = self.create_config(steer);
		self.pwm.set_config(&config);
	}
}
