use embassy_rp::adc::{self, Adc, Async};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Sender};
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

// This is the array we will send over the channel to the main process
type Amplitudes = [u32; NUM_OF_MEASUREMENTS * 3];

// This is the number of measurements that we get per sensor
const NUM_OF_MEASUREMENTS: usize = 64;

// This is the amount of delay between each measurement. This is required, otherwise it will freeze the main
// process
const TICKS_PER_MEASUREMENT: u64 = 100;

// The channel which we send and recieve data through
pub static CHANNEL_AMPLITUDES: Channel<ThreadModeRawMutex, Amplitudes, 64> = Channel::new();

// This is technically a process, which allows us to use Async
#[embassy_executor::task]
pub async fn read_adc_value(
    mut adc: Adc<'static, Async>,
    mut p26: adc::Channel<'static>,
    mut p27: adc::Channel<'static>,
    mut p28: adc::Channel<'static>,
    tx_value: Sender<'static, ThreadModeRawMutex, Amplitudes, 64>,
) {
    // Filters for the signal from power line noise with Savannah's EMGFilters
    let mut filter_1 = emg_filter_rs::EMGFilters::new(emg_filter_rs::SampleFrequency::Hz1000, emg_filter_rs::NotchFrequency::Hz60, true, true, true);
    let mut filter_2 = emg_filter_rs::EMGFilters::new(emg_filter_rs::SampleFrequency::Hz1000, emg_filter_rs::NotchFrequency::Hz60, true, true, true);
    let mut filter_3 = emg_filter_rs::EMGFilters::new(emg_filter_rs::SampleFrequency::Hz1000, emg_filter_rs::NotchFrequency::Hz60, true, true, true);

    // Define the measurements arrays that will later be combined
    let mut measurements_1 = [0f32; NUM_OF_MEASUREMENTS];
    let mut measurements_2 = [0f32; NUM_OF_MEASUREMENTS];
    let mut measurements_3 = [0f32; NUM_OF_MEASUREMENTS];
    
    // The amplitudes array which will be sent, initialize as zeros
    let mut amplitudes: Amplitudes = [0u32; NUM_OF_MEASUREMENTS * 3];
    let mut pos = 0;
    loop {
        // keep adding filtered value from the read value.
        measurements_1[pos] = filter_1.update(adc.read(&mut p26).await.unwrap().into());
        measurements_2[pos] = filter_2.update(adc.read(&mut p27).await.unwrap().into());
        measurements_3[pos] = filter_3.update(adc.read(&mut p28).await.unwrap().into());

        // Keep adding one, until 64 measurements from each sensor is taken, then run if statment
        pos = (pos + 1) % NUM_OF_MEASUREMENTS;
        if pos == 0 {
            // compute amplitudes of measurements
            let spectrum = microfft::real::rfft_64(&mut measurements_1);
            spectrum[0].im = 0.0;
            for (i, a) in spectrum.iter().map(|c| c.l1_norm() as u32).enumerate() {
                amplitudes[i] = a;
            }

            let spectrum = microfft::real::rfft_64(&mut measurements_2);
            spectrum[0].im = 0.0;
            for (i, a) in spectrum.iter().map(|c| c.l1_norm() as u32).enumerate() {
                amplitudes[i + NUM_OF_MEASUREMENTS] = a;
            }

            let spectrum = microfft::real::rfft_64(&mut measurements_3);
            spectrum[0].im = 0.0;
            for (i, a) in spectrum.iter().map(|c| c.l1_norm() as u32).enumerate() {
                amplitudes[i + NUM_OF_MEASUREMENTS * 2] = a;
            }
            // send amplitudes to main thread
            tx_value.send(amplitudes).await;
        }

        // Wait the needed amount of ticks per measurement.
        Timer::after_ticks(TICKS_PER_MEASUREMENT).await;
    }
}
