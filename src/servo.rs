use crate::println;
use crate::servo::pac::interrupt;
use defmt_rtt as _;
use embedded_hal::timer::CountDown;
use fugit::ExtU64;
use hal::gpio::PullNone;
use panic_probe as _;
use rp_pico::hal::clocks::SystemClock;
use rp_pico::hal::dma::{Channel, ChannelIndex, DMAExt, CH0, CH1};
use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1, Gpio2, Gpio3};
use rp_pico::hal::gpio::{FunctionNull, FunctionPio0, FunctionSio, Pin, PullDown, SioInput};
use rp_pico::hal::pio::{PIOExt, StateMachineIndex, UninitStateMachine, PIO, SM0};
use rp_pico::hal::{self, fugit, Timer};
use rp_pico::pac::{self, DMA, PIO0, PIO1, RESETS};
use servo_pio::calibration::{AngularCalibration, Calibration};
use servo_pio::pwm_cluster::{dma_interrupt, GlobalState, GlobalStates, Handler};
use servo_pio::servo_cluster::{
    ServoCluster, ServoClusterBuilder, ServoClusterBuilderError, ServoData,
};

//use pimoroni_servo2040::hal;
//use pimoroni_servo2040::pac;

//global variables
const NUM_SERVOS: usize = 4;
const NUM_CHANNELS: usize = 12;
static mut STATE1: Option<GlobalState<CH0, CH1, PIO0, SM0>> = {
    const NONE_HACK: Option<GlobalState<CH0, CH1, PIO0, SM0>> = None;
    NONE_HACK
};
static mut GLOBALS: GlobalStates<NUM_CHANNELS> = {
    const NONE_HACK: Option<&'static mut dyn Handler> = None;
    GlobalStates {
        states: [NONE_HACK; NUM_CHANNELS],
    }
};

#[allow(non_snake_case)]
pub fn servo(
    RESETS: &mut RESETS,
    PIO0: PIO0,
    PIO1: PIO1,
    system_clock: SystemClock,
    timer: Timer,
    DMA: DMA,
    gpio0: Pin<Gpio0, FunctionNull, PullDown>,
    gpio1: Pin<Gpio1, FunctionNull, PullDown>,
    gpio2: Pin<Gpio2, FunctionNull, PullDown>,
    gpio3: Pin<Gpio3, FunctionNull, PullDown>,
) -> ! {
    let servo_pins: [_; NUM_SERVOS] = [
        ServoData {
            pin: gpio0.reconfigure::<FunctionPio0, PullNone>().into_dyn_pin(),
            calibration: Calibration::builder(AngularCalibration::default())
                .limit_lower()
                .limit_upper()
                .build(),
        },
        ServoData {
            pin: gpio1.reconfigure::<FunctionPio0, PullNone>().into_dyn_pin(),
            calibration: Calibration::builder(AngularCalibration::default())
                .limit_lower()
                .limit_upper()
                .build(),
        },
        ServoData {
            pin: gpio2.reconfigure::<FunctionPio0, PullNone>().into_dyn_pin(),
            calibration: Calibration::builder(AngularCalibration::default())
                .limit_lower()
                .limit_upper()
                .build(),
        },
        ServoData {
            pin: gpio3.reconfigure::<FunctionPio0, PullNone>().into_dyn_pin(),
            calibration: Calibration::builder(AngularCalibration::default())
                .limit_lower()
                .limit_upper()
                .build(),
        },
    ];

    println!("Made servo pins");

    let (mut pio0, sm0, _, _, _) = PIO0.split(RESETS);
    // Use a different pio for the leds because they run at a different
    // clock speed.
    let (mut pio1, sm10, _, _, _) = PIO1.split(RESETS);
    let dma = DMA.split(RESETS);

    println!("setup pio and dma");

    // Build the servo cluster
    let mut servo_cluster = match build_servo_cluster(
        &mut pio0,
        sm0,
        (dma.ch0, dma.ch1),
        servo_pins,
        #[cfg(feature = "debug_pio")]
        pins.scl
            .reconfigure::<FunctionPio0, PullNone>()
            .into_dyn_pin(),
        system_clock,
        unsafe { &mut STATE1 },
    ) {
        Ok(cluster) => cluster,
        Err(e) => {
            match e {
                BuildError::Build(build) => match build {
                    ServoClusterBuilderError::MismatchingGlobalState => {
                        println!("Failed to build servo cluster: MismatchingGlobalState")
                    }
                    ServoClusterBuilderError::MissingPins => {
                        println!("Failed to build servo cluster: MissingPins")
                    }
                    ServoClusterBuilderError::MissingCalibrations => {
                        println!("Failed to build servo cluster: MissingCalibrations")
                    }
                },
            };
            #[allow(clippy::empty_loop)]
            loop {}
        }
    };

    println!("setup servo cluster");

    // Unmask the DMA interrupt so the handler can start running. This can only
    // be done after the servo cluster has been built.
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::DMA_IRQ_0);
    }

    const MIN_PULSE: f32 = 1500.0;
    const MID_PULSE: f32 = 2000.0;
    const MAX_PULSE: f32 = 2500.0;
    let movement_delay = 20.millis();

    // We need to use the indices provided by the cluster because the servo pin
    // numbers do not line up with the indices in the clusters and PIO.
    let [servo1, servo2, servo3, servo4] = servo_cluster.servos();

    let mut count_down = timer.count_down();

    println!("Setup countdown");

    servo_cluster.set_pulse(servo1, MAX_PULSE, false);
    servo_cluster.set_pulse(servo2, MID_PULSE, false);
    servo_cluster.set_pulse(servo3, MIN_PULSE, false);
    servo_cluster.set_value(servo4, 35.0, true);
    count_down.start(movement_delay * 5);

    let mut velocity1: f32 = 10.0;
    let mut velocity2: f32 = 15.0;
    let mut velocity3: f32 = 25.0;
    let mut velocity4: f32 = 50.0;

    println!("ssetup pluses");
    #[allow(clippy::empty_loop)]
    loop {
        println!("Looping");
        for (servo, velocity) in [
            (servo1, &mut velocity1),
            (servo2, &mut velocity2),
            (servo3, &mut velocity3),
            (servo4, &mut velocity4),
        ] {
            println!("In for loop");
            let mut pulse = match servo_cluster.pulse(servo) {
                Some(value) => value,
                None => {
                    println!("Failed to get pulse");
                    continue;
                }
            };
            pulse += *velocity;
            if !(MIN_PULSE..=MAX_PULSE).contains(&pulse) {
                *velocity *= -1.0;
                pulse = pulse.clamp(MIN_PULSE, MAX_PULSE);
            }
            // Assign pulses to each servo, but passing `false` to prevent
            // immediate usage of the pulse.
            servo_cluster.set_pulse(servo, pulse, false);
            println!("Setting Pulse");
        }
        // Load to trigger a simultaneous of the values to the servos. Phasing
        // of the PWM signals will be used to prevent voltage spikes.
        servo_cluster.load();
        count_down.start(movement_delay);
    }
}

enum BuildError {
    Build(ServoClusterBuilderError),
}

fn build_servo_cluster<C1, C2, P, SM>(
    pio: &mut PIO<P>,
    sm: UninitStateMachine<(P, SM)>,
    dma_channels: (Channel<C1>, Channel<C2>),
    servo_data: [ServoData<AngularCalibration, FunctionPio0>; NUM_SERVOS],
    #[cfg(feature = "debug_pio")] side_set_pin: Pin<DynPinId, FunctionPio0, PullNone>,
    system_clock: SystemClock,
    state: &'static mut Option<GlobalState<C1, C2, P, SM>>,
) -> Result<ServoCluster<NUM_SERVOS, P, SM, AngularCalibration>, BuildError>
where
    C1: ChannelIndex,
    C2: ChannelIndex,
    P: PIOExt<PinFunction = FunctionPio0>,
    SM: StateMachineIndex,
{
    #[allow(unused_mut)]
    let mut builder: ServoClusterBuilder<
        '_,
        AngularCalibration,
        C1,
        C2,
        P,
        SM,
        FunctionPio0,
        NUM_SERVOS,
        NUM_CHANNELS,
    > = ServoCluster::<NUM_SERVOS, P, SM, AngularCalibration>::builder(
        pio,
        sm,
        dma_channels,
        unsafe { &mut GLOBALS },
    )
    .pins_and_calibration(servo_data);
    #[cfg(feature = "debug_pio")]
    {
        builder = builder.side_set_pin(side_set_pin);
    }
    builder
        .pwm_frequency(50.0)
        .build(&system_clock, state)
        .map_err(BuildError::Build)
}

#[interrupt]
fn DMA_IRQ_0() {
    critical_section::with(|_| {
        // Safety: we're within a critical section, so nothing else will modify global_state.
        dma_interrupt(unsafe { &mut GLOBALS });
    });
}
