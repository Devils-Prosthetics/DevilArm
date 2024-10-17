use burn::optim::AdamConfig;
use training::TrainingConfig;
use training::train;
use burn::backend::{Autodiff, Wgpu};
use std::path::PathBuf;
use std::env;


fn main() {
    let out = &PathBuf::from(env::var("OUT_DIR").unwrap());

    type MyBackend = Wgpu<f32, i32>;
    type MyAutodiffBackend = Autodiff<MyBackend>;

    let device = burn::backend::wgpu::WgpuDevice::default();
    let artifact_dir = out.join("model");

    if artifact_dir.exists() { return; }

    train::<MyAutodiffBackend>(
        artifact_dir.to_str().expect("Failed to convert artifact_dir to string"),
        TrainingConfig::new(AdamConfig::new()),
        device.clone(),
    );
}
