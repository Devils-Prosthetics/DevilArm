use burn::optim::AdamConfig;
use training::TrainingConfig;
use training::train;
use burn::backend::{Autodiff, Wgpu};
use std::path::PathBuf;
use std::env;

// Macro which allows the user to print out to console despite being in a build.rs file
// Comes from https://github.com/rust-lang/cargo/issues/985#issuecomment-1071667472
macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    // Get the path to the out directory, which is where we'll store the model.
    let out = env::temp_dir();

    // Create a wgpu backend, this makes training significantly faster, and has great compatability between OS's.
    type MyBackend = Wgpu<f32, i32>;
    type MyAutodiffBackend = Autodiff<MyBackend>;

    // Instantiate the backend
    let device = burn::backend::wgpu::WgpuDevice::default();

    // The artifact directory is where the model gets outputted, which is where ever OUT_DIR is + "/model".
    let artifact_dir = out.join("devil-model");

    // Output the directory, so it is easier to debug in the future, should be removed eventually
    p!("artifact_dir: {:?}", artifact_dir.to_str().unwrap());

    println!("cargo:rustc-env=OUT_DIR={}", artifact_dir.to_str().unwrap());

    // If the artifact directory exists, then we shouldn't recompile the model.
    // The artifact directory shouldn't exist if one of the build dependencies for inference changes.
    if artifact_dir.exists() { return; }

    // Train the model, with a default training config, and the specified wgpu backend.
    train::<MyAutodiffBackend>(
        artifact_dir.to_str().expect("Failed to convert artifact_dir to string"),
        TrainingConfig::new(AdamConfig::new()),
        device.clone(),
    );
}
