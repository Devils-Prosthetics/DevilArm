// No Standard library is required to run this code, this is so that it can run on the micro controller.
#![no_std]

use model::Model;

pub use model;
pub use burn::tensor::activation::softmax;
use burn::prelude::*;

// Add the model into the program at compile time, this should be found in the build directory in /model/model.bin
// It is put there by the build.rs script.
static MODEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/model/model.bin"));

// Run the model with the given data, which is a rank 2 tensor, basically a 2d array.
// Returns the result as a 2d array
pub fn infer<B: Backend>(device: B::Device, item: Tensor<B, 1>) -> burn::tensor::Tensor<B, 1> {
    let model: Model<B> = Model::from_embedded(&device, MODEL_BYTES);

    let predicted = model.forward(item);

    return predicted;
}

// If running as `cargo test`, include std library, this makes it so that we can println the results for debugging
// the tests.
#[cfg(test)]
extern crate std;

// Only include this if running `cargo test`
#[cfg(test)]
mod tests {
    // Import everything from module above this one, that being lib.rs
    use super::*;

    // NdArray is a backend for the model which runs the inference on the CPU, drastically slower, but still no_std
    use burn::backend::NdArray;
    use model::{Output, MODEL_INPUTS};

    use std::println;

    // Convenience types
    type Backend = NdArray<f32>;
    type BackendDeice = <Backend as burn::tensor::backend::Backend>::Device;

    // Add a test, this gets ran upon running `cargo test`.
    #[test]
    fn test_inference() {
        // Use the default backend, which in this case will always result in NdArray backend
        let device = BackendDeice::default();

        // This is the input array for testing our specific model, should result in the pinkythumb prediction.
        let input: [f32; MODEL_INPUTS] = [1310.0, 22.0, 31.0, 44.0, 21.0, 18.0, 65.0, 23.0, 36.0, 18.0, 21.0, 17.0, 9.0, 15.0, 8.0, 15.0, 11.0, 2.0, 9.0, 4.0, 4.0, 2.0, 4.0, 4.0, 1.0, 2.0, 2.0, 1.0, 2.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1326.0, 27.0, 47.0, 13.0, 54.0, 5.0, 29.0, 35.0, 50.0, 12.0, 13.0, 17.0, 11.0, 19.0, 4.0, 8.0, 0.0, 10.0, 3.0, 4.0, 1.0, 3.0, 3.0, 3.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1313.0, 17.0, 51.0, 21.0, 42.0, 8.0, 29.0, 28.0, 12.0, 38.0, 9.0, 12.0, 5.0, 22.0, 19.0, 4.0, 10.0, 2.0, 6.0, 4.0, 4.0, 2.0, 1.0, 2.0, 2.0, 2.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let tensor: burn::tensor::Tensor<Backend, 1> = Tensor::from_data(input, &device);

        // Run the inference with the 1d tensor, unsqueezed to become a 2d array.
        let inference = infer(device, tensor);

        let inference = softmax(inference, 0);

        // Print out the inference, should add an assert, but this code is rather unstable, and will be changed later
        println!("inference: {:?}", inference);

        let result = inference
            .into_data()
            .as_slice::<f32>() // Convert the inference tensor into a slice of f32's
            .unwrap()
            .into_iter()
            .enumerate() // Add index onto the probability
            .map(|(index, probability)| {
                println!("index: {index}");
                let output = model::Output::try_from(index); // the index is which output it is corresponding with
                let (output, probability) = match output {
                    Ok(output) => (output, *probability), // Returns the output gesture and the probability
                    Err(_) => (model::Output::Unknown, *probability), // This should theoretically never happen, but it's good to test
                };
                println!("{:?}: {:?}", output, probability); // Log the results
                (output, probability) // return the results
            })
            .max_by(|x, y| x.1.partial_cmp(&y.1).unwrap()) // get the gesture with the highest probability 
            .unwrap();

        println!("result: {:?}", result);
        panic!();
    }
}
