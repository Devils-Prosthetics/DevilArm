use burn::module::Module;
use burn::record::Recorder;
use burn::record::{BinBytesRecorder, BinFileRecorder, FullPrecisionSettings};
use burn::tensor::activation::softmax;
use devil_ml_model::Model;
use burn::prelude::Backend;
use burn::data::dataloader::Dataset;
use burn::tensor::Tensor;
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

    train::<MyAutodiffBackend>(
        ARTIFACT_DIR,
        TrainingConfig::new(AdamConfig::new()).with_custom_renderer(false),
        device.clone(),
    );

    let data = std::fs::read(format!("{}/model.bin", ARTIFACT_DIR)).unwrap();

    let record = BinBytesRecorder::<FullPrecisionSettings>::default()
        .load(data, &device)
        .expect("failed to find model");

    let model = Model::new(&device).load_record(record);

    let test_input = include_str!(concat!(env!("OUT_DIR"), "/data/testing.csv"));

    // Instantiate the dataset from the text
    let testing_dataset = DevilDataset::new(test_input);

    testing_dataset.iter().for_each(|item| {
        let input_to_validate: burn::tensor::Tensor<MyBackend, 1> = Tensor::from_data(item.inputs.as_slice(), &device);
        let inference = devil_ml_model::infer(input_to_validate, &model);
        let inference = softmax(inference, 0);

        let result = inference
            .clone()
            .into_data()
            .as_slice::<f32>() // Convert the inference tensor into a slice of f32's
            .unwrap()
            .into_iter()
            .enumerate() // Add index onto the probability
            .map(|(index, probability)| {
                let output = devil_ml_model::Output::try_from(index); // the index is which output it is corresponding with
                let (output, probability) = match output {
                    Ok(output) => (output, *probability), // Returns the output gesture and the probability
                    Err(_) => (devil_ml_model::Output::Unknown, *probability), // This should theoretically never happen, but it's good to test
                };
                println!("{:?}: {:?}", output, probability); // Log the results
                (output, probability) // return the results
            })
            .max_by(|x, y| x.1.partial_cmp(&y.1).unwrap()) // get the gesture with the highest probability
            .unwrap();
        println!("Actual gesture: {:?}", item.label);
        println!("Predicted gesture: {:?}\n\n\n", result.0); // Log the gesture
    });
}
