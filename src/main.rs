// Setup
#![no_std]
#![no_main]

use embedded_hal::digital::v2::OutputPin;
use normalize::print_normalize;
use rp_pico::hal;
use rp_pico::pac::{CorePeripherals, Peripherals};
use sensor::{read_sensor_input, setup_adc_pins};
use setup::{setup_adc, setup_clocks, setup_delay, setup_pins, setup_serial, setup_sio, Serial};

mod normalize;
mod sensor;
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

    let clocks = setup_clocks(
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    );
    let mut delay = setup_delay(core.SYST, &clocks);
    let sio = setup_sio(pac.SIO);
    let pins = setup_pins(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut adc = setup_adc(pac.ADC, &mut pac.RESETS);

    setup_serial(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        &mut pac.RESETS,
    );

    Serial::read(read);

    // Set the LED to be an output
    let mut led_pin = pins.led.into_push_pull_output();
    let mut adc_pins = setup_adc_pins(pins.gpio26, pins.gpio27, pins.gpio28);

    // Blink the LED at 1 Hz
    loop {
        led_pin.set_high().unwrap();
        delay.delay_ms(500);
        led_pin.set_low().unwrap();
        delay.delay_ms(500);

        let values = read_sensor_input(&mut adc, &mut adc_pins.0, &mut adc_pins.1, &mut adc_pins.2);
        println!(
            "Raw Sensor Values: {}, {}, {}",
            values.0, values.1, values.2
        );

        print_normalize(3.0, 1.0, &mut [1.0, 3.0, 5.0, 10.0])
    }
}
