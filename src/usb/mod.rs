use embassy_futures::join::join;
use embassy_rp::peripherals::USB;
use embassy_rp::rom_data::reset_to_usb_boot;
use embassy_rp::usb::Driver;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};

use embassy_usb::{Builder, Config};

use crate::with_class;

use self::serial::Handler;

mod serial;

pub fn trim_ascii_whitespace(x: &[u8]) -> &[u8] {
    let from = match x.iter().position(|x| !x.is_ascii_whitespace()) {
        Some(i) => i,
        None => return &x[0..0],
    };
    let to = x.iter().rposition(|x| !x.is_ascii_whitespace()).unwrap();
    &x[from..=to]
}

struct CommandHandler {}

impl Handler for CommandHandler {
    async fn handle_data(&self, data: &[u8]) {
        let command = b"q";
        let second_command = b"elf2uf2-term";
        if trim_ascii_whitespace(data).eq_ignore_ascii_case(command) {
            reset_to_usb_boot(0, 0);
        } else if trim_ascii_whitespace(data).eq_ignore_ascii_case(second_command) {
            reset_to_usb_boot(0, 0);
        }
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

    join(device.run(), with_class!(1024, log::LevelFilter::Info, class, CommandHandler)).await;
}

