#![no_std]

use burn::record::Recorder;
use burn::record::FullPrecisionSettings;
use burn::record::BinBytesRecorder;
use burn::{
    nn::{
        Dropout, DropoutConfig, Linear, LinearConfig, Relu,
    }, prelude::*
};

// #[cfg(test)]
// extern crate std;

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    linear1: Linear<B>,
    linear2: Linear<B>,
    linear3: Linear<B>,
    dropout: Dropout,
    activation: Relu,
    phantom: core::marker::PhantomData<B>,
    device: burn::module::Ignored<B::Device>,
}

impl<B: Backend> Model<B> {
    pub fn from_embedded(device: &B::Device, embedded_states: &[u8]) -> Self {
        let record = BinBytesRecorder::<FullPrecisionSettings>::default()
            .load(embedded_states.to_vec(), device)
            .expect("Should decode state successfully");
        Self::new(device).load_record(record)
    }
}

impl<B: Backend> Model<B> {
    #[allow(unused_variables)]
    pub fn new(device: &B::Device) -> Self {
        let linear1 = LinearConfig::new(192, 10).init(device);
        let linear2 = LinearConfig::new(10, 10).init(device);
        let linear3 = LinearConfig::new(10, 3).init(device);
        let activation = Relu::new();
        let dropout = DropoutConfig::new(0.5).init();

        Self {
            activation,
            linear1,
            linear2,
            linear3,
            dropout,
            phantom: core::marker::PhantomData,
            device: burn::module::Ignored(device.clone()),
        }
    }

    #[allow(clippy::let_and_return, clippy::approx_constant)]
    pub fn forward(&self, input: Tensor<B, 2>) -> Tensor<B, 2> {
        let x = self.linear1.forward(input);
        let x = self.dropout.forward(x);
        let x = self.activation.forward(x);
        let x = self.linear2.forward(x);
        let x = self.dropout.forward(x);
        let x = self.activation.forward(x);

        self.linear3.forward(x)
    }
}
