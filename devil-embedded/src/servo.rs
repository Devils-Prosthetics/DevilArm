// This code has now been included in the raspberry pi pico embassy examples.
use core::time::Duration;
use embassy_rp::pio::Instance;
use embassy_time::Timer;
use crate::pio_pwm::PwmPio;

// This was merged by me into the embassy main branch in the examples.

// The following will be interpreted as micro seconds.
const DEFAULT_MIN_PULSE_WIDTH: u64 = 1000; // uncalibrated default, the shortest duty cycle sent to a servo
const DEFAULT_MAX_PULSE_WIDTH: u64 = 2000; // uncalibrated default, the longest duty cycle sent to a servo 
const REFRESH_INTERVAL: u64 = 20000; // classic default period to refresh servos in microseconds
const DEFAULT_MAX_DEGREE_ROTATION: u64 = 180; // 180 degrees is typical

// This struct stores the Servo's information, most notably the period, min_pulse_width, and max_pulse_width
// To understand what each of these do, please read up on pwm, here https://blog.wokwi.com/learn-servo-motor-using-wokwi-logic-analyzer/
pub struct ServoBuilder<'d, T: Instance, const SM: usize> {
    pwm: PwmPio<'d, T, SM>,
    period: Duration,
    min_pulse_width: Duration,
    max_pulse_width: Duration,
    max_degree_rotation: u64, // The max number of degrees a servo is able to rotate.
}

impl<'d, T: Instance, const SM: usize> ServoBuilder<'d, T, SM> {
    // Create a new ServoBuilder, which is then used to instantiate a Servo struct. This is for configuration. Read up
    // On Rust builder pattern here https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
    pub fn new(pwm: PwmPio<'d, T, SM>) -> Self {
        Self {
            pwm,
            period: Duration::from_micros(REFRESH_INTERVAL),
            min_pulse_width: Duration::from_micros(DEFAULT_MIN_PULSE_WIDTH),
            max_pulse_width: Duration::from_micros(DEFAULT_MAX_PULSE_WIDTH),
            max_degree_rotation: DEFAULT_MAX_DEGREE_ROTATION,
        }
    }

    // Set the duration of the period that the servo uses, typically given in the datasheet.
    // Default is 20 ms
    pub fn set_period(mut self, duration: Duration) -> Self {
        self.period = duration;
        self
    }

    // Set the duration of the minimum pulse width that the servo uses, typically given in the datasheet.
    // Default is 1 ms
    pub fn set_min_pulse_width(mut self, duration: Duration) -> Self {
        self.min_pulse_width = duration;
        self
    }

    // Set the duration of the maximum pulse width that the servo uses, typically given in the datasheet.
    // Default is 2 ms
    pub fn set_max_pulse_width(mut self, duration: Duration) -> Self {
        self.max_pulse_width = duration;
        self
    }

    // Set the duration of the maximum number of degrees that the servo can rotate, typically given in the datasheet.
    // Default is 180 degrees
    pub fn set_max_degree_rotation(mut self, degree: u64) -> Self {
        self.max_degree_rotation = degree;
        self
    }

    // Get the Servo struct
    pub fn build(mut self) -> Servo<'d, T, SM> {
        self.pwm.set_period(self.period);
        Servo {
            pwm: self.pwm,
            min_pulse_width: self.min_pulse_width,
            max_pulse_width: self.max_pulse_width,
            max_degree_rotation: self.max_degree_rotation,
            last_time_written: self.min_pulse_width.clone()
        }
    }
}

// The Servo struct, which is constructed from ServoBuilder
pub struct Servo<'d, T: Instance, const SM: usize> {
    pwm: PwmPio<'d, T, SM>,
    min_pulse_width: Duration,
    max_pulse_width: Duration,
    pub max_degree_rotation: u64,
    last_time_written: Duration
}

pub enum EasingFunction {
    Cubic
}

impl<'d, T: Instance, const SM: usize> Servo<'d, T, SM> {
    // Start running the Servo
    pub fn start(&mut self) {
        self.pwm.start();
    }

    // Stop the Servo from running
    pub fn stop(&mut self) {
        self.pwm.stop();
    }

    // Write the duration that the Servo PWM cycle should run. More can be found from pio_pwm.rs
    pub fn write_time(&mut self, duration: Duration) {
        self.last_time_written = duration.clone();
        self.pwm.write(duration);
    }

    // Rotate the servo to the specified degree.
    pub fn rotate(&mut self, degree: u64) {
        let duration = self.degrees_to_time(degree);

        self.pwm.write(duration);
    }

    // WIP
    fn degrees_to_time(&self, degree: u64) -> Duration {
        let degree_per_nano_second = (self.max_pulse_width.as_nanos() as u64 - self.min_pulse_width.as_nanos() as u64) / self.max_degree_rotation;
        let mut duration = Duration::from_nanos(degree * degree_per_nano_second + self.min_pulse_width.as_nanos() as u64);
        if self.max_pulse_width < duration {
            duration = self.max_pulse_width;
        }

        duration
    }

    // WIP
    fn time_to_degree(&self, time: Duration) -> u64 {
        let degree_per_nano_second = (self.max_pulse_width.as_nanos() as u64 - self.min_pulse_width.as_nanos() as u64) / self.max_degree_rotation;
        let degree = (time - self.min_pulse_width).as_nanos() / degree_per_nano_second as u128;

        degree as u64
    }

    // Ease moving into the specified degree
    // tick is the amount of time between each update, and duration is the length of time to run easing funciton
    // over with the specified easing function.
    pub async fn ease_to(&mut self, degree: u64, refresh_rate: embassy_time::Duration, duration: embassy_time::Duration, easing_function: EasingFunction) {
        let mut steps: f32 = 0.0;
        let time_to_wite = self.degrees_to_time(degree);
        let steps_end = (duration.as_ticks() / refresh_rate.as_ticks()) as f32;
        loop {
            steps += 1.0;
            Timer::after(refresh_rate).await;

            // match easing_function {
                // EasingFunction::Cubic => easer::functions::Cubic::ease_in(steps, self.last_time_written.as_nanos() as f32, time_to_wite.as_nanos() as f32, steps_end),
            // }
        }
    }

    // Release the pwm struct from the servo
    pub fn release(self) -> PwmPio<'d, T, SM> {
        self.pwm
    }
}
