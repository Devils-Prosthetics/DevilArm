#![no_std]
use core::default::Default;

use model::{Model, ModelConfig, TrainingConfig};

use burn::{
    backend::NdArray, data::{dataloader::batcher::Batcher, dataset::vision::MnistItem}, optim::AdamConfig, prelude::*, record::{BinBytesRecorder, CompactRecorder, FullPrecisionSettings, Recorder}
};

type Backend = NdArray<f32>;
type BackendDeice = <Backend as burn::tensor::backend::Backend>::Device;

static MODEL_BYTES: &[u8] = include_bytes!("/tmp/guide/model.bin");

pub fn infer<B: burn::prelude::Backend>(artifact_dir: &str, device: B::Device, item: MnistItem) {
    // Load model binary record in full precision
    let record = BinBytesRecorder::<FullPrecisionSettings>::default()
        .load(MODEL_BYTES.to_vec(), &device)
        .expect("Should be able to load model the model weights from bytes");

    let config = TrainingConfig::new(ModelConfig::new(10, 512), AdamConfig::new());

    let model: Model<B> = config.model.init(&device).load_record(record);

    let predicted = model.forward(item.image.into());
}


#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use std::println;
    use super::*;

    #[test]
    fn test_inference() {
    }

    // #[test]
    // fn it_works() {
    //     let model = Model::new();
    //     println!("model def: {:?}", model.model_definition());
    //     panic!();
    // }
}
