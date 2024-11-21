use burn::{
    backend::{Autodiff, Wgpu},
    optim::AdamConfig,
};
use devil_ml_model::ARTIFACT_DIR;
use std::env;
use training::{data::DevilDataset, train, TrainingConfig};

// Trains the model if `cargo run` is ran and outputs it to /tmp/guide
// Serves no real purpose, except for debugging, might revamp in the future.
// By default does not use custom renderer, which is better for debugging.

fn main() {
    type MyBackend = Wgpu<f32, i32>;
    type MyAutodiffBackend = Autodiff<MyBackend>;

    let device = burn::backend::wgpu::WgpuDevice::default();

    println!("ARTIFACT_DIR: {:?}", ARTIFACT_DIR);

    let trained_model = train::<MyAutodiffBackend>(
        ARTIFACT_DIR,
        TrainingConfig::new(AdamConfig::new()).with_custom_renderer(false),
        device.clone(),
    );

    let validation_input = include_str!(concat!(env!("OUT_DIR"), "/data/train.csv"));

    // Instantiate the dataset from the text
    let validation = DevilDataset::new(validation_input);

    // validation.

    // let input_to_validate: burn::tensor::Tensor<Backend, 1> = Tensor::from_data(inputs, &device);
    // let output = devil_ml_model::infer(input_to_validate, &trained_model);
}
