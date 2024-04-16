#![no_std]
#![no_main]

use embassy_rp::peripherals::USB;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Config as AdcConfig, InterruptHandler as AdcInterruptHandler};
use embassy_rp::gpio;
use embassy_rp::gpio::Pull;
use embassy_rp::pwm::{Config as PwmConfig, Pwm};
use embassy_rp::usb::{Driver, InterruptHandler as UsbInterruptHandler};
use embassy_rp::{adc, bind_interrupts};
use embassy_time::Timer;
use gpio::{Level, Output};
use sensor::{read_adc_value, CHANNEL_AMPLITUDES};
use servo::Servo;
use {defmt_rtt as _, panic_probe as _};
use log::*;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => AdcInterruptHandler;
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
});

mod sensor;
mod servo;

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, LevelFilter::Info, driver);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    // Note that the LED pin on the Pico W is PIN_16
    let mut led = Output::new(p.PIN_25, Level::Low);
    let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let p26 = adc::Channel::new_pin(p.PIN_26, Pull::None);
    let p27 = adc::Channel::new_pin(p.PIN_27, Pull::None);
    let p28 = adc::Channel::new_pin(p.PIN_28, Pull::None);

    let pwm = Pwm::new_output_b(p.PWM_SLICE0, p.PIN_1, PwmConfig::default());
    let mut servo = Servo::default(pwm);

    
    // spawn the task that reads the ADC value
    spawner
        .spawn(read_adc_value(adc, p26, p27, p28, CHANNEL_AMPLITUDES.sender()))
        .unwrap();

    let rx_adv_value = CHANNEL_AMPLITUDES.receiver();
    led.set_high();

    loop {
        servo.move_servo(0);
        Timer::after_secs(1).await;
        servo.move_servo(1050);
        Timer::after_secs(1).await;
        let amplitudes = rx_adv_value.receive().await;
        info!("Amplitudes {:?}", amplitudes);
    }
}
