use core::str;

use embassy_futures::join::join;
use embassy_rp::peripherals::USB;
use embassy_rp::rom_data::reset_to_usb_boot;
use embassy_rp::usb::Driver;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};

use embassy_usb::{Builder, Config};
use embassy_usb_logger::{with_class, ReceiverHandler};

// Create a new command handler
struct Handler;

impl ReceiverHandler for Handler {
    async fn handle_data(&self, data: &[u8]) {
        if let Ok(data) = str::from_utf8(data) {
            let data = data.trim();

            // If you are using elf2uf2-term with the '-t' flag, then when closing the serial monitor,
            // this will automatically put the pico into boot mode.
            if data == "q" || data == "elf2uf2-term" {
                reset_to_usb_boot(0, 0); // Restart the chip
            } else if data.eq_ignore_ascii_case("hello") {
                log::info!("World!");
            } else {
                log::info!("Recieved: {:?}", data);
            }
        }
    }

    fn new() -> Self {
        Self
    }
}

#[embassy_executor::task]
pub async fn usb_task(driver: Driver<'static, USB>) {
    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Devils Prosthetics");
    config.product = Some("DevilArm");
    config.serial_number = Some("DEVIL");
    config.max_power = 500;
    config.max_packet_size_0 = 64;

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Create classes on the builder.
    let class = CdcAcmClass::new(&mut builder, &mut state, 64);

    let mut device = builder.build();

    join(
        device.run(),
        with_class!(1024, log::LevelFilter::Info, class, Handler),
    )
    .await;
}
