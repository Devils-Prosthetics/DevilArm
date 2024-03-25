// Setup
#![no_std]
#![no_main]

// A shorter alias for the Hardware Abstraction Layer, which provides
// higher-level drivers.
use embedded_hal::digital::v2::OutputPin;
use rp_pico::hal;

// USB Device support
use usb_device::class_prelude::*;

use rp_pico::pac::{Peripherals, CorePeripherals};
use setup::{setup_serial, Serial};
use rp_pico::hal::prelude::*;

mod setup;

pub fn main(mut pac: Peripherals, core: CorePeripherals) -> ! {
    let read = |buf: &mut [u8], count: usize| {
        // Convert to upper case
        buf.iter_mut().take(count).for_each(|b| {
            b.make_ascii_uppercase();
        });

        // Send back to the host
        let mut wr_ptr = &buf[..count];
        while !wr_ptr.is_empty() {
            let _ = Serial::write(wr_ptr).map(|len| {
                wr_ptr = &wr_ptr[len..];
            });
        }
    };

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();


    let mut delay = cortex_m::delay::Delay::new(core.SYST, (&clocks.system_clock.freq().to_Hz()).clone());

    let sio = hal::Sio::new(pac.SIO);

    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    

    setup_serial(usb_bus);

    Serial::read(read);

    // Set the LED to be an output
    let mut led_pin = pins.led.into_push_pull_output();

    // Blink the LED at 1 Hz
    loop {
        let _ = Serial::write(b"Hello World!\r\n");
        let _ = Serial::flush();
        led_pin.set_high().unwrap();
        delay.delay_ms(500);
        led_pin.set_low().unwrap();
        delay.delay_ms(500);
    }
}
