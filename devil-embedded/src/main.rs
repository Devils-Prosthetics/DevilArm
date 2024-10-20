#![no_std]
#![no_main]

use core::time::Duration;

extern crate alloc;
use crate::pio_pwm::PwmPio;
use burn::backend::NdArray;
use burn::tensor::Tensor;
use devil_ml::model::MODEL_INPUTS;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Config as AdcConfig, InterruptHandler as AdcInterruptHandler};
use embassy_rp::gpio;
use embassy_rp::gpio::Pull;
use embassy_rp::peripherals::{PIO0, USB};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::usb::{Driver, InterruptHandler as UsbInterruptHandler};
use embassy_rp::{adc, bind_interrupts};
use gpio::{Level, Output};

use sensor::{read_adc_value, CHANNEL_AMPLITUDES};
use servo::ServoBuilder;

use log::*;
use usb::usb_task;
use {defmt_rtt as _, panic_probe as _};

use embedded_alloc::Heap;

use devil_ml::{self, softmax};

mod gesture;
mod pio_pwm;
mod sensor;
mod servo;
mod usb;

// Sets up an allocator to be used, without this, you cannot put things on the heap, no vectors!
#[global_allocator]
static HEAP: Heap = Heap::empty();

// Bind the interupts to the corresponding handlers
bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => AdcInterruptHandler;
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

// We are going to use NdArray to run the machine learning backend.
type Backend = NdArray<f32>;
type BackendDeice = <Backend as burn::tensor::backend::Backend>::Device;

// This is the main function for the program. Where execution starts.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initializes the allocator, must be done before use.
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 100 * 1024; // Watch out for this, if it is too big or small, program may crash
        // this is in u8 bytes, as such this is a total of 100kb
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) } // Initialize the heap
    }

    // This returns the peripherals struct
    let p = embassy_rp::init(Default::default());

    // This handles the usb
    let driver = Driver::new(p.USB, Irqs);

    // Spawn the usb_task, and pass the driver for it.
    spawner.spawn(usb_task(driver)).unwrap();

    // Defining the pins that are to be used with the program
    // Note that the LED pin on the Pico W is PIN_16
    let mut led = Output::new(p.PIN_25, Level::Low);
    // These are the pins for the sensors
    let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let p26 = adc::Channel::new_pin(p.PIN_26, Pull::None);
    let p27 = adc::Channel::new_pin(p.PIN_27, Pull::None);
    let p28 = adc::Channel::new_pin(p.PIN_28, Pull::None);

    // This defines a Servo, not really in use rn, but it will be more integrated in the final code,
    // Mostly detached for easy testing
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
            CHANNEL_AMPLITUDES.sender(), // This is the channel which sends the data between "processes"
        ))
        .unwrap();

    let rx_adv_value = CHANNEL_AMPLITUDES.receiver(); // This is where the channel recieves the data
    led.set_high(); // turn on the led

    info!("Getting started");

    // Initialize the NdArray backend
    let device = BackendDeice::default();

    loop {
        // Convert the u32 into f32, these really should be normalized between 0 and 1.
        let inputs: [f32; MODEL_INPUTS] = rx_adv_value.receive().await.map(|x| x as f32);
        let inputs_min = inputs.into_iter().reduce(f32::min).unwrap(); // get the min of inputs
        let inputs_max = inputs.into_iter().reduce(f32::max).unwrap(); // get the max of inputs

        let inputs = inputs.map(|x| (x - inputs_min) / (inputs_max - inputs_min)); // normalize the input

        info!("NewData"); // Everything between NewData and EndData gets saved to a csv to be trained
        for input in inputs {
            info!("{}", input);
        }
        info!("EndData");

        // Create a tensor from the input
        let tensor: burn::tensor::Tensor<Backend, 1> = Tensor::from_data(inputs, &device);

        // run inference on the tensor with the NdArray
        let inference = devil_ml::infer(device, tensor);

        // normalize each output from the tensor to be between 0 and 1
        let inference = softmax(inference, 0);

        info!("inference done!");
        let result = inference
            .into_data()
            .as_slice::<f32>() // Convert the inference tensor into a slice of f32's
            .unwrap()
            .into_iter()
            .enumerate() // Add index onto the probability
            .map(|(index, probability)| {
                let output = devil_ml::model::Output::try_from(index); // the index is which output it is corresponding with
                let (output, probability) = match output {
                    Ok(output) => (output, *probability), // Returns the output gesture and the probability
                    Err(_) => (devil_ml::model::Output::Unknown, *probability), // This should theoretically never happen, but it's good to test
                };
                info!("{:?}: {:?}", output, probability); // Log the results
                (output, probability) // return the results
            })
            .max_by(|x, y| x.1.partial_cmp(&y.1).unwrap()) // get the gesture with the highest probability 
            .unwrap();

        info!("Predicted gesture: {:?}\n", result.0); // Log the gesture

        // Add in here the displaying of the gesture at a later date
    }
}
