use embassy_rp::pio::Instance;

use crate::servo::Servo;

// Gestures struct, which contains all the servos used to construct the arm.
struct Gestures<'d, T1: Instance, const SM1: usize, T2: Instance, const SM2: usize, T3: Instance, const SM3: usize> {
	thumb_servo: Servo<'d, T1, SM1>,
	index_and_middle_servo: Servo<'d, T2, SM2>,
	ring_and_pinky_servo: Servo<'d, T3, SM3>,
}

impl<'d, T1: Instance, const SM1: usize, T2: Instance, const SM2: usize, T3: Instance, const SM3: usize> Gestures<'d, T1, SM1, T2, SM2, T3, SM3> {
	/// Create a new Geastures struct
	pub fn new(thumb_servo: Servo<'d, T1, SM1>, index_and_middle_servo: Servo<'d, T2, SM2>, ring_and_pinky_servo: Servo<'d, T3, SM3>) -> Self {
		Self {
			thumb_servo,
			index_and_middle_servo,
			ring_and_pinky_servo
		}
	}

	/// Make the servos move to position
	pub fn start(&mut self) {
		self.thumb_servo.start();
		self.index_and_middle_servo.start();
		self.ring_and_pinky_servo.start();
	}

	/// Stop the servos from trying to move
	pub fn stop(&mut self) {
		self.thumb_servo.stop();
		self.index_and_middle_servo.stop();
		self.ring_and_pinky_servo.stop();
	}

	/// Make the arm create a thumbs up
	pub fn thumbs_up(&mut self) {
		self.thumb_servo.rotate(0);
		self.index_and_middle_servo.rotate(self.thumb_servo.max_degree_rotation);
		self.ring_and_pinky_servo.rotate(self.thumb_servo.max_degree_rotation);
	}

	/// Make the arm create a pinch
	pub fn pinch(&mut self) {
		self.thumb_servo.rotate(90);
		self.index_and_middle_servo.rotate(90);
		self.ring_and_pinky_servo.rotate(self.thumb_servo.max_degree_rotation);
	}
}
