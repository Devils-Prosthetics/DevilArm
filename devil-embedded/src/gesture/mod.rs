use embassy_rp::pio::Instance;

use crate::servo::Servo;

struct Gestures<'d, T1: Instance, const SM1: usize, T2: Instance, const SM2: usize, T3: Instance, const SM3: usize> {
	thumb_servo: Servo<'d, T1, SM1>,
	index_and_middle_servo: Servo<'d, T2, SM2>,
	ring_and_pinky_servo: Servo<'d, T3, SM3>,
}

impl<'d, T1: Instance, const SM1: usize, T2: Instance, const SM2: usize, T3: Instance, const SM3: usize> Gestures<'d, T1, SM1, T2, SM2, T3, SM3> {
	pub fn new(thumb_servo: Servo<'d, T1, SM1>, index_and_middle_servo: Servo<'d, T2, SM2>, ring_and_pinky_servo: Servo<'d, T3, SM3>) -> Self {
		Self {
			thumb_servo,
			index_and_middle_servo,
			ring_and_pinky_servo
		}
	}

	pub fn start(&mut self) {
		self.thumb_servo.start();
		self.index_and_middle_servo.start();
		self.ring_and_pinky_servo.start();
	}

	pub fn stop(&mut self) {
		self.thumb_servo.stop();
		self.index_and_middle_servo.stop();
		self.ring_and_pinky_servo.stop();
	}

	pub fn thumbs_up(&mut self) {
		self.thumb_servo.rotate(0);
		self.index_and_middle_servo.rotate(self.thumb_servo.max_degree_rotation);
		self.ring_and_pinky_servo.rotate(self.thumb_servo.max_degree_rotation);
	}

	pub fn pinch(&mut self) {
		self.thumb_servo.rotate(90);
		self.index_and_middle_servo.rotate(90);
		self.ring_and_pinky_servo.rotate(self.thumb_servo.max_degree_rotation);
	}
}
