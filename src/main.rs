#![no_std]
#![no_main]

use core::fmt::Debug;
use core::time::Duration;

extern crate alloc;
use crate::pio_pwm::PwmPio;
use alloc::vec;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Config as AdcConfig, InterruptHandler as AdcInterruptHandler};
use embassy_rp::gpio;
use embassy_rp::gpio::Pull;
use embassy_rp::peripherals::{PIO0, USB};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::usb::{Driver, InterruptHandler as UsbInterruptHandler};
use embassy_rp::{adc, bind_interrupts};
use embassy_time::Timer;
use gpio::{Level, Output};

use embedded_alloc::Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

// use microflow::model;
use sensor::{read_adc_value, CHANNEL_AMPLITUDES};

use servo::ServoBuilder;

use log::*;
use usb::usb_task;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => AdcInterruptHandler;
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

mod pio_pwm;
mod sensor;
mod servo;
mod usb;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(usb_task(driver)).unwrap();
    Timer::after_secs(4).await;

    // Note that the LED pin on the Pico W is PIN_16
    let mut led = Output::new(p.PIN_25, Level::Low);
    let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let p26 = adc::Channel::new_pin(p.PIN_26, Pull::None);
    let p27 = adc::Channel::new_pin(p.PIN_27, Pull::None);
    let p28 = adc::Channel::new_pin(p.PIN_28, Pull::None);

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);
    let pwm = PwmPio::attach(&mut common, sm0, p.PIN_1);
    let mut servo = ServoBuilder::new(pwm)
        .set_max_degree_rotation(120)
        .set_min_pulse_width(Duration::from_micros(350))
        .set_max_pulse_width(Duration::from_micros(2600))
        .build();
    servo.start();

    // spawn the task that reads the ADC value
    spawner
        .spawn(read_adc_value(
            adc,
            p26,
            p27,
            p28,
            CHANNEL_AMPLITUDES.sender(),
        ))
        .unwrap();

    let rx_adv_value = CHANNEL_AMPLITUDES.receiver();
    led.set_high();

    info!("Getting started");

    // let mut level = 0;
    loop {
        let amplitudes = rx_adv_value.receive().await;
        info!("NewData");
        for amplitude in amplitudes.iter() {
            info!("{:?}", amplitude);
        }
        info!("EndData");
    }
}
