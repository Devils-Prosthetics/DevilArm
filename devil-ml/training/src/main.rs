use burn::{
    backend::{Autodiff, Wgpu},
    optim::AdamConfig,
};

use training::{train, TrainingConfig};

// Trains the model if `cargo run` is ran and outputs it to /tmp/guide
// Serves no real purpose, except for debugging, might revamp in the future.
// By default does not use custom renderer, which is better for debugging.

fn main() {
    type MyBackend = Wgpu<f32, i32>;
    type MyAutodiffBackend = Autodiff<MyBackend>;

    let device = burn::backend::wgpu::WgpuDevice::default();
    let artifact_dir = "/tmp/guide";
    train::<MyAutodiffBackend>(
        artifact_dir,
        TrainingConfig::new(AdamConfig::new()).with_custom_renderer(false),
        device.clone(),
    );
}
