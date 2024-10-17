use burn::{
    backend::{Autodiff, Wgpu},
    optim::AdamConfig,
};

use training::{train, TrainingConfig};

fn main() {
    type MyBackend = Wgpu<f32, i32>;
    type MyAutodiffBackend = Autodiff<MyBackend>;

    let device = burn::backend::wgpu::WgpuDevice::default();
    let artifact_dir = "/tmp/guide";
    train::<MyAutodiffBackend>(
        artifact_dir,
        TrainingConfig::new(AdamConfig::new()),
        device.clone(),
    );
}
