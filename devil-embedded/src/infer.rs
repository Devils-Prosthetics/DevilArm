use burn::{backend::NdArray, tensor::Tensor};
use devil_ml_model::Model;

// Add the model into the program at compile time, this should be found in the build directory in /model/model.bin
// It is put there by the build.rs script.
static MODEL_BYTES: &[u8] = include_bytes!(concat!(env!("ARTIFACT_DIR"), "/model.bin"));

static mut MODEL: Option<Model<Backend>> = None;

// We are going to use NdArray to run the machine learning backend.
pub type Backend = NdArray<f32>;
pub type BackendDeice = <Backend as burn::tensor::backend::Backend>::Device;

pub struct Inferer<B: burn::prelude::Backend> {
    model: Model<B>,
}

impl<B: burn::prelude::Backend> Inferer<B> {
    pub fn new(device: &B::Device) -> Self {
        let model = Model::from_embedded(device, MODEL_BYTES);
        Inferer { model }
    }

    pub fn infer(&self, item: Tensor<B, 1>) -> Tensor<B, 1> {
        devil_ml_model::infer(item, &self.model)
    }
}
