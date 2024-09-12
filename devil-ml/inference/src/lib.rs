#![no_std]
use core::default::Default;
use core::marker::PhantomData;
use burn::{
    backend::{ndarray::NdArrayDevice, NdArray}, module::ModuleDisplayDefault, prelude::*, record::{FullPrecisionSettings, Record}
};

mod model;

type Backend = NdArray<f32>;
type BackendDeice = <Backend as burn::tensor::backend::Backend>::Device;

pub struct Model {
    device: BackendDeice,
    model: model::Model<Backend>
}

impl Model {
    pub fn model_definition(self) {
        let record = self.model.into_record().into_item::<FullPrecisionSettings>();
    }

    pub fn new() -> Self {
        // Get a default device for the backend
        let device = BackendDeice::default();

        // Create a new model and load the state
        let model: model::Model<Backend> = model::Model::default();

        Self {
            device,
            model,
        }
    }

    pub fn run_model<'a>(&self, input: f32) -> Tensor<Backend, 2> {
        // Define the tensor
        let input = Tensor::<Backend, 2>::from_floats([[input]], &self.device);

        // Run the model on the input
        let output = self.model.forward(input);

        output
    }
}


#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use std::println;
    use super::*;

    #[test]
    fn test_inference() {
        let model = Model::new();

        // Define input
        let mut input = 0.0;
        loop {
            if input > 2.0 { break; }
            input += 0.05;

            // Run the model
            let output = model.run_model(input);

            // Output the values
            match output.into_primitive().tensor().array.as_slice() {
                Some(slice) => println!("input: {} - output: {:?}", input, slice),
                None => panic!("Failed to get value")
            };
        }
    }

    #[test]
    fn it_works() {
        let model = Model::new();
        println!("model def: {:?}", model.model_definition());
        panic!();
    }
}
