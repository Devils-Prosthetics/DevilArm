use crate::println;

use embedded_hal::adc::OneShot;
use rp_pico::hal::{
    adc::AdcPin,
    gpio::{
        bank0::{Gpio26, Gpio27, Gpio28},
        DefaultTypeState, FunctionSio, Pin, PullNone, SioInput,
    },
    Adc,
};

pub fn setup_adc_pins(
    gpio26: Pin<
        Gpio26,
        <Gpio26 as DefaultTypeState>::Function,
        <Gpio26 as DefaultTypeState>::PullType,
    >,
    gpio27: Pin<
        Gpio27,
        <Gpio27 as DefaultTypeState>::Function,
        <Gpio27 as DefaultTypeState>::PullType,
    >,
    gpio28: Pin<
        Gpio28,
        <Gpio28 as DefaultTypeState>::Function,
        <Gpio28 as DefaultTypeState>::PullType,
    >,
) -> (
    AdcPin<Pin<Gpio26, FunctionSio<SioInput>, PullNone>>,
    AdcPin<Pin<Gpio27, FunctionSio<SioInput>, PullNone>>,
    AdcPin<Pin<Gpio28, FunctionSio<SioInput>, PullNone>>,
) {
    let mut adc_pin_0 = AdcPin::new(gpio26.into_floating_input());
    let mut adc_pin_1 = AdcPin::new(gpio27.into_floating_input());
    let mut adc_pin_2 = AdcPin::new(gpio28.into_floating_input());

    return (adc_pin_0, adc_pin_1, adc_pin_2);
}

pub fn read_sensor_input(
    adc: &mut Adc,
    adc_pin_0: &mut AdcPin<Pin<Gpio26, FunctionSio<SioInput>, PullNone>>,
    adc_pin_1: &mut AdcPin<Pin<Gpio27, FunctionSio<SioInput>, PullNone>>,
    adc_pin_2: &mut AdcPin<Pin<Gpio28, FunctionSio<SioInput>, PullNone>>,
) -> (u16, u16, u16) {
    let pin_adc_counts_0: u16 = adc.read(adc_pin_0).unwrap();
    let pin_adc_counts_1: u16 = adc.read(adc_pin_1).unwrap();
    let pin_adc_counts_2: u16 = adc.read(adc_pin_2).unwrap();

    return (pin_adc_counts_0, pin_adc_counts_1, pin_adc_counts_2);
}

pub fn print_sensor_output(values: (u16, u16, u16)) {
    println!("NewData");
    println!("{}", values.0);
    println!("{}", values.1);
    println!("{}", values.2);
    println!("EndData");
}
