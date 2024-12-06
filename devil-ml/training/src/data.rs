use burn::{
    data::{
        dataloader::batcher::Batcher,
        dataset::{Dataset, InMemDataset},
    },
    prelude::*,
};

use csv;
use devil_ml_model::Output;

// This file just creates the batching logic, just a bunch of boiler plate, based upon
// https://burn.dev/burn-book/basic-workflow/data.html
// and https://burn.dev/burn-book/building-blocks/dataset.html

//  Batcher's structure, just includes the device.
#[derive(Clone)]
pub struct DevilBatcher<B: Backend> {
    device: B::Device,
}

impl<B: Backend> DevilBatcher<B> {
    // Creates new batcher with device
    pub fn new(device: B::Device) -> Self {
        Self { device }
    }
}

// This is a single batch, it takes in inputs tensor, and an outputs tensor.
// Inputs is a 2d array of the MODEL_INPUTS sensor inputs
// [[f32; MODEL_INPUTS]; n] where n is the number of items in the batch
// targets is the corresponding label
#[derive(Clone, Debug)]
pub struct DevilBatch<B: Backend> {
    pub inputs: Tensor<B, 2>,
    pub targets: Tensor<B, 1, Int>,
}

// Defines the dataset, which just stores in memory the DevilItem struct
pub struct DevilDataset {
    dataset: InMemDataset<DevilItem>,
}

// Just stores a [f32; MODEL_INPUTS] vector of sensor inputs, and a label as a number, will change to
// an enum in the future
#[derive(Clone, Debug)]
pub struct DevilItem {
    pub inputs: Vec<f32>,
    pub label: Output,
}

// Makes DevilDataset implement the Dataset trait, allows it to be used as a dataset
impl Dataset<DevilItem> for DevilDataset {
    fn get(&self, index: usize) -> Option<DevilItem> {
        self.dataset.get(index) // just get the item from the InMemDataset
    }

    fn len(&self) -> usize {
        self.dataset.len() // just get the len from the InMemDataset
    }
}

impl DevilDataset {
    /// Converts the csv as a string into a DevilDataset
    pub fn new(csv: &str) -> Self {
        let items = Self::from_csv(csv);

        // Create an InMemDataset of DevilItems
        let dataset = InMemDataset::new(items);

        Self { dataset }
    }

    /// Return an vector of DevilItems from a csv
    pub fn from_csv(input: &str) -> Vec<DevilItem> {
        // Initialize csv reader
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false) // Our files have no headers
            .from_reader(input.as_bytes()); // the string as bytes is the input

        // The vector which will store all the DevilItems
        let mut output: Vec<DevilItem> = Vec::new();

        // For each row in the csv
        for row in rdr.records() {
            // We have to match, because if records is empty, technically this will be an error
            match row {
                Ok(row) => {
                    // Just converts each item in the row to a &str
                    let values: Vec<&str> = row.iter().collect();

                    // Split the last item of the vector into label, and the rest into inputs, if there is a None
                    // returned from split_last, just inform that the row has an issue, then skip it.
                    let Some((label, inputs)) = values.split_last() else {
                        eprintln!("row is improperly formatted");
                        continue;
                    };

                    // Parse all of the inputs into f32's
                    let inputs: Vec<f32> = inputs
                        .into_iter()
                        .map(|s| s.parse::<f32>().expect("Found non number in csv")) // Filter out invalid f32
                        .collect();

                    // parsing label to Output
                    let label = label.to_string();
                    let label = match Output::from_str(&label) {
                        Some(val) => val,
                        None => {
                            println!("Failed to parse label '{}'", label);
                            continue;
                        },
                    };

                    // Push the item to the vector
                    output.push(DevilItem {
                        inputs, // First 196 items as f32
                        label,  // Output
                    });
                }
                Err(_) => continue,
            };
        }

        return output;
    }
}

// Batcher function, this gets called when burn wants to batch everything.
impl<B: Backend> Batcher<DevilItem, DevilBatch<B>> for DevilBatcher<B> {
    fn batch(&self, items: Vec<DevilItem>) -> DevilBatch<B> {
        // Return a vector of 2d tensors with shape [1, MODEL_INPUTS]
        let inputs: Vec<Tensor<B, 2>> = items
            .iter()
            .map(|item| TensorData::from(item.inputs.as_slice()).convert::<B::FloatElem>())
            .map(|data| Tensor::<B, 1>::from_data(data, &self.device).unsqueeze())
            .collect();

        // You typically do normalization in the function above, but with our setup, we expect things to be
        // already normalized, b/c we do that on the chip.

        // Return a vector of classification with a 1d int tensor.
        let targets = items
            .iter()
            .map(|item| {
                Tensor::<B, 1, Int>::from_data(
                    [(item.label.clone() as u8).elem::<B::IntElem>()],
                    &self.device,
                )
            })
            .collect();

        // Concatenate that vector into 2d tensor with shape [n, MODEL_INPUTS]
        let inputs = Tensor::cat(inputs, 0).to_device(&self.device);

        // Combines all the classifications into a 1d tensor with shape [n]
        let targets = Tensor::cat(targets, 0).to_device(&self.device);

        DevilBatch { inputs, targets }
    }
}
