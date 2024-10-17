#![no_std]
use core::default::Default;

use model::Model;

use burn::{
    backend::NdArray, prelude::*, record::{BinBytesRecorder, FullPrecisionSettings, Recorder}
};

type Backend = NdArray<f32>;
type BackendDeice = <Backend as burn::tensor::backend::Backend>::Device;

static MODEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/model/model.bin"));

pub fn infer<B: burn::prelude::Backend>(device: B::Device, item: Tensor<B, 2>) -> burn::tensor::Tensor<B, 2> {
    let model: Model<B> = Model::from_embedded(&device, MODEL_BYTES);

    let predicted = model.forward(item);

    return predicted;
}


#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use std::println;
    use super::*;

    type Backend = NdArray<f32>;
    type BackendDeice = <Backend as burn::tensor::backend::Backend>::Device;

    #[test]
    fn test_inference() {
        let device = BackendDeice::default();

        let input: [f32; 192] = [1310.0, 22.0, 31.0, 44.0, 21.0, 18.0, 65.0, 23.0, 36.0, 18.0, 21.0, 17.0, 9.0, 15.0, 8.0, 15.0, 11.0, 2.0, 9.0, 4.0, 4.0, 2.0, 4.0, 4.0, 1.0, 2.0, 2.0, 1.0, 2.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1326.0, 27.0, 47.0, 13.0, 54.0, 5.0, 29.0, 35.0, 50.0, 12.0, 13.0, 17.0, 11.0, 19.0, 4.0, 8.0, 0.0, 10.0, 3.0, 4.0, 1.0, 3.0, 3.0, 3.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1313.0, 17.0, 51.0, 21.0, 42.0, 8.0, 29.0, 28.0, 12.0, 38.0, 9.0, 12.0, 5.0, 22.0, 19.0, 4.0, 10.0, 2.0, 6.0, 4.0, 4.0, 2.0, 1.0, 2.0, 2.0, 2.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let tensor: burn::tensor::Tensor<Backend, 1> = Tensor::from_data(input, &device);

        let inference = infer(device, tensor.unsqueeze());
    }

    // #[test]
    // fn it_works() {
    //     let model = Model::new();
    //     println!("model def: {:?}", model.model_definition());
    //     panic!();
    // }
}
