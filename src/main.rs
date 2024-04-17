#![no_std]
#![no_main]

use core::time::Duration;

use crate::pio_pwm::PwmPio;
// use crate::servo::{Servo, us_to_pio_cycles};
use embassy_rp::peripherals::{PIO0, USB};
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Config as AdcConfig, InterruptHandler as AdcInterruptHandler};
use embassy_rp::gpio;
use embassy_rp::gpio::Pull;
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::pwm::{Config as PwmConfig, Pwm};
use embassy_rp::usb::{Driver, InterruptHandler as UsbInterruptHandler};
use embassy_rp::{adc, bind_interrupts};
use embassy_time::Timer;
use gpio::{Level, Output};
use sensor::{read_adc_value, CHANNEL_AMPLITUDES};

use servo::ServoBuilder;
use usb::usb_task;
use {defmt_rtt as _, panic_probe as _};
use log::*;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => AdcInterruptHandler;
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

mod sensor;
mod servo;
mod usb;
mod pio_pwm;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(usb_task(driver)).unwrap();
    Timer::after_secs(2).await;

    // Note that the LED pin on the Pico W is PIN_16
    let mut led = Output::new(p.PIN_25, Level::Low);
    let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let p26 = adc::Channel::new_pin(p.PIN_26, Pull::None);
    let p27 = adc::Channel::new_pin(p.PIN_27, Pull::None);
    let p28 = adc::Channel::new_pin(p.PIN_28, Pull::None);

    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);
    let pwm = PwmPio::attach(&mut common, sm0, p.PIN_1);
    let mut servo = ServoBuilder::new(pwm)
        .set_max_degree_rotation(120)
        .set_min_pulse_width(Duration::from_micros(350))
        .set_max_pulse_width(Duration::from_micros(2600))
        .build();
    servo.start();

    // spawn the task that reads the ADC value
    spawner
        .spawn(read_adc_value(adc, p26, p27, p28, CHANNEL_AMPLITUDES.sender()))
        .unwrap();

    let rx_adv_value = CHANNEL_AMPLITUDES.receiver();
    led.set_high();

    let mut level = 0;
    loop {
        level = (level + 1) % 120;
        info!("level: {}", level);
        servo.rotate(level);
        Timer::after_millis(100).await;
        // let amplitudes = rx_adv_value.receive().await;
        // info!("NewData");
        // for amplitude in amplitudes.iter() {
        //     info!("{:?}", amplitude);
        // }
        // info!("EndData");
    }
}
