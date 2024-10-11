use model::ModelConfig;
use burn::optim::AdamConfig;
use burn::backend::{Autodiff, Wgpu};
use training::TrainingConfig;
use std::path::PathBuf;
use std::env;

use training::train;

fn main() {
    let out = &PathBuf::from(env::var("OUT_DIR").unwrap());

    type MyBackend = Wgpu<f32, i32>;
    type MyAutodiffBackend = Autodiff<MyBackend>;

    let device = burn::backend::wgpu::WgpuDevice::default();
    let artifact_dir = out.join("model");
    let artifact_dir = artifact_dir.to_str().expect("Failed to create directory for training model");

    if !artifact_dir.is_empty() { return; }

    train::<MyAutodiffBackend>(
        artifact_dir,
        TrainingConfig::new(ModelConfig::new(10, 512), AdamConfig::new()),
        device.clone(),
    );
}
