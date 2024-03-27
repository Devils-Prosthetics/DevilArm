//! # Pico USB Serial (with Interrupts) Example
//!
//! Creates a USB Serial device on a Pico board, with the USB driver running in
//! the USB interrupt.
//!
//! This will create a USB Serial device echoing anything it receives. Incoming
//! ASCII characters are converted to upercase, so you can tell it is working
//! and not just local-echo!
//!
//! See the `Cargo.toml` file for Copyright and license details.

use core::cell::RefCell;


use cortex_m::delay::Delay;
use cortex_m::interrupt::free;
use cortex_m::interrupt::Mutex;
use embedded_hal::watchdog;
// The macro for our start-up function
use rp_pico::entry;


use rp_pico::hal::clocks::ClocksManager;
use rp_pico::hal::clocks::SystemClock;
use rp_pico::hal::clocks::UsbClock;
// The macro for marking our interrupt functions
use rp_pico::hal::pac::interrupt;

use panic_probe as _;

use defmt::*;
use defmt_rtt as _;

// A shorter alias for the Peripheral Access Crate, which provides low-level
// register access
use rp_pico::hal::pac;

// A shorter alias for the Hardware Abstraction Layer, which provides
// higher-level drivers.
use rp_pico::hal;

use rp_pico::hal::sio::SioGpioBank0;
use rp_pico::hal::Sio;
use rp_pico::hal::Watchdog;
use rp_pico::pac::IO_BANK0;
use rp_pico::pac::PADS_BANK0;
use rp_pico::pac::SIO;
use rp_pico::pac::SYST;
use rp_pico::pac::USBCTRL_DPRAM;
use rp_pico::pac::USBCTRL_REGS;
use rp_pico::Pins;
// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::SerialPort;

use rp_pico::hal::prelude::*;

use crate::main;


/// The USB Device Driver (shared with the interrupt).
static mut USB_DEVICE: Option<UsbDevice<hal::usb::UsbBus>> = None;

/// The USB Bus Driver (shared with the interrupt).
static mut USB_BUS: Option<UsbBusAllocator<hal::usb::UsbBus>> = None;

/// The USB Serial Device Driver (shared with the interrupt).
static mut USB_SERIAL: Option<SerialPort<hal::usb::UsbBus>> = None;

static mut ENABLE_SERIAL: bool = false;

type ReadFunction = fn(&mut [u8], usize);
static READFUNC: Mutex<RefCell<Option<ReadFunction>>> = Mutex::new(RefCell::new(None));

pub fn setup_clocks(XOSC: pac::XOSC, CLOCKS: pac::CLOCKS, PLL_SYS: pac::PLL_SYS, PLL_USB: pac::PLL_USB, RESETS: &mut pac::RESETS, watchdog: &mut Watchdog) -> ClocksManager {
    hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        XOSC,
        CLOCKS,
        PLL_SYS,
        PLL_USB,
        RESETS,
        watchdog,
    )
    .ok()
    .unwrap()
}

pub fn setup_delay(SYST: SYST, clocks: &ClocksManager) -> Delay {
    cortex_m::delay::Delay::new(SYST, (clocks.system_clock.freq().to_Hz()).clone())
}

pub fn setup_sio(SIO: SIO) -> Sio {
    hal::Sio::new(SIO)
}

pub fn setup_pins(IO_BANK0: IO_BANK0, PADS_BANK0: PADS_BANK0, gpio_bank0: SioGpioBank0, RESETS: &mut pac::RESETS) -> Pins {
    rp_pico::Pins::new(
        IO_BANK0,
        PADS_BANK0,
        gpio_bank0,
        RESETS,
    )
}


pub fn setup_serial(USBCTRL_REGS: USBCTRL_REGS, USBCTRL_DPRAM: USBCTRL_DPRAM, usb_clock: UsbClock, RESETS: &mut pac::RESETS) {
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        USBCTRL_REGS,
        USBCTRL_DPRAM,
        usb_clock,
        true,
        RESETS,
    ));

    unsafe {
        ENABLE_SERIAL = true;

        // Note (safety): This is safe as interrupts haven't been started yet
        USB_BUS = Some(usb_bus);

        // Grab a reference to the USB Bus allocator. We are promising to the
        // compiler not to take mutable access to this global variable whilst this
        // reference exists!
        let bus_ref = USB_BUS.as_ref().unwrap();

        // Set up the USB Communications Class Device driver
        let serial = SerialPort::new(bus_ref);
        
        USB_SERIAL = Some(serial);

        // Create a USB device with a fake VID and PID
        let usb_dev = UsbDeviceBuilder::new(bus_ref, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Devils Prosthetics")
            .product("Serial port")
            .serial_number("DEVILARM")
            .device_class(2) // from: https://www.usb.org/defined-class-codes
            .build();
        
        // Note (safety): This is safe as interrupts haven't been started yet
        USB_DEVICE = Some(usb_dev);

        pac::NVIC::unmask(hal::pac::Interrupt::USBCTRL_IRQ);
    }
}

/// Entry point to our bare-metal application.
///
/// The `#[entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables are initialised.
///
/// The function configures the RP2040 peripherals, then blinks the LED in an
/// infinite loop.
#[entry]
fn setup() -> ! {
    // Grab our singleton objects
    let pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    main(pac, core);
}

pub struct Serial {}

impl Serial {
    pub fn write(data: &[u8]) -> Result<usize, UsbError> {
        let serial = unsafe { USB_SERIAL.as_mut().unwrap() };
        serial.write(data)
    }

    pub fn flush() -> Result<(), UsbError> {
        let serial = unsafe { USB_SERIAL.as_mut().unwrap() };
        serial.flush()
    }

    pub fn read(read: ReadFunction) {
        free(move |cs| {
            *READFUNC.borrow(cs).borrow_mut() = Some(read);
        });
    }
}

/// This function is called whenever the USB Hardware generates an Interrupt
/// Request.
///
/// We do all our USB work under interrupt, so the main thread can continue on
/// knowing nothing about USB.
#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
    // Grab the global objects. This is OK as we only access them under interrupt.
    let usb_dev = USB_DEVICE.as_mut().unwrap();
    let serial = USB_SERIAL.as_mut().unwrap();

    if ENABLE_SERIAL {
        // Poll the USB driver with all of our supported USB Classes
        if usb_dev.poll(&mut [serial]) {
            let mut buf = [0u8; 64];
            match serial.read(&mut buf) {
                Err(_e) => {
                    // Do nothing
                }
                Ok(0) => {
                    // Do nothing
                }
                Ok(count) => {
                    free(|cs| {
                        if let Some(read) = READFUNC.borrow(cs).borrow_mut().as_mut() {
                            read(&mut buf, count);
                        }
                    });
                }
            }
        }
    }
}


