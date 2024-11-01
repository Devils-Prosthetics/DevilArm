#![allow(unused)]
use embassy_rp::pio::Instance;

use crate::servo::Servo;

// Gestures struct, which contains all the servos used to construct the arm.
struct Gestures<
    'd,
    T1: Instance,
    const SM1: usize,
    T2: Instance,
    const SM2: usize,
    T3: Instance,
    const SM3: usize,
> {
    thumb_servo: Servo<'d, T1, SM1>,
    fingers_servo: Servo<'d, T2, SM2>,
    arm_servo: Servo<'d, T3, SM3>,
}

impl<
        'd,
        T1: Instance,
        const SM1: usize,
        T2: Instance,
        const SM2: usize,
        T3: Instance,
        const SM3: usize,
    > Gestures<'d, T1, SM1, T2, SM2, T3, SM3>
{
    /// Create a new Geastures struct
    pub fn new(
        thumb_servo: Servo<'d, T1, SM1>,
        fingers_servo: Servo<'d, T2, SM2>,
        arm_servo: Servo<'d, T3, SM3>,
    ) -> Self {
        Self {
            thumb_servo,
            fingers_servo,
            arm_servo,
        }
    }

    /// Make the servos move to position
    pub fn start(&mut self) {
        self.thumb_servo.start();
        self.fingers_servo.start();
        self.arm_servo.start();
    }

    /// Stop the servos from trying to move
    pub fn stop(&mut self) {
        self.thumb_servo.stop();
        self.fingers_servo.stop();
        self.arm_servo.stop();
    }

    /// Make the arm create a thumbs up
    pub fn thumbs_up(&mut self) {
        self.thumb_servo.rotate(0);
        self.fingers_servo
            .rotate(self.thumb_servo.max_degree_rotation);
        self.arm_servo.rotate(self.thumb_servo.max_degree_rotation);
    }

    /// Make the arm create a pinch
    pub fn pinch(&mut self) {
        self.thumb_servo.rotate(90);
        self.fingers_servo.rotate(90);
        self.arm_servo.rotate(self.thumb_servo.max_degree_rotation);
    }
}
