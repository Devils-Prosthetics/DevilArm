use std::collections::HashMap;

use burn::{
    data::{dataloader::batcher::Batcher, dataset::{Dataset, InMemDataset}},
    prelude::*,
};

use csv;
use serde::Deserialize;


#[derive(Clone)]
pub struct DevilBatcher<B: Backend> {
    device: B::Device,
}

impl<B: Backend> DevilBatcher<B> {
    pub fn new(device: B::Device) -> Self {
        Self { device }
    }
}

#[derive(Clone, Debug)]
pub struct DevilBatch<B: Backend> {
    pub inputs: Tensor<B, 2>,
    pub targets: Tensor<B, 1>,
}

pub struct DevilDataset {
    dataset: InMemDataset<DevilItem>
}

#[derive(Clone, Debug)]
pub struct DevilItem {
    pub inputs: Vec<f32>,
    pub label: u8,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DevilItemRaw {
    pub inputs: Vec<f32>,
    pub label: String,
}

impl Dataset<DevilItem> for DevilDataset {
    fn get(&self, index: usize) -> Option<DevilItem> {
        self.dataset.get(index)
    }

    fn len(&self) -> usize {
        self.dataset.len()
    }
}

impl DevilDataset {
    pub fn new(csv: &str) -> Self {
        let (items, labels) = Self::from_csv(csv);

        let items = items.into_iter().map(|item| DevilItem {
            inputs: item.inputs,
            label: *labels.get(&item.label).expect("Failed to find id for label"),
        }).collect();

        let dataset = InMemDataset::new(items);

        Self { dataset }
    }

    pub fn from_csv(input: &str) -> (Vec<DevilItemRaw>, HashMap<String, u8>) {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(input.as_bytes());

        let mut output: Vec<DevilItemRaw> = Vec::new();
        let mut labels: HashMap<String, u8> = HashMap::new();

        for result in rdr.records() {
            match result {
                Ok(val) => {
                    let values: Vec<&str> = val.iter().collect();

                    let Some((label, inputs)) = values.split_last() else { continue };
                    let inputs: Vec<f32> = inputs.into_iter()
                        .map(|s| s.parse::<f32>().expect("Found non number in csv"))  // Filter out invalid f32
                        .collect();

                    output.push(DevilItemRaw {
                        inputs, // First 196 items as f32
                        label: label.to_string() // Last item as a string
                    });

                    if !labels.contains_key(*label) {
                        labels.insert(label.to_string(), labels.len() as u8);
                    }
                },
                Err(_) => todo!(),
            };
        }

        return (output, labels);
    }
}

impl<B: Backend> Batcher<DevilItem, DevilBatch<B>> for DevilBatcher<B> {
    fn batch(&self, items: Vec<DevilItem>) -> DevilBatch<B> {
        let inputs: Vec<Tensor<B, 2>> = items
            .iter()
            .map(|item| TensorData::from(item.inputs.as_slice()).convert::<B::FloatElem>())
            .map(|data| Tensor::<B, 1>::from_data(data, &self.device).unsqueeze())
            .collect();

        let targets = items
            .iter()
            .map(|item| {
                Tensor::<B, 1>::from_data(
                    [item.label.elem::<B::IntElem>()],
                    &self.device,
                )
            })
            .collect();

        let inputs = Tensor::cat(inputs, 0).to_device(&self.device);
        let targets = Tensor::cat(targets, 0).to_device(&self.device);

        DevilBatch { inputs, targets }
    }
}