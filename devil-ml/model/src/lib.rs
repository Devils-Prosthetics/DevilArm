// No Standard library is required to run this code, this is so that it can run on the micro controller.
#![no_std]

// The purpose of this file is to be shared between inference and training.

use burn::record::HalfPrecisionSettings;
use burn::record::Recorder;
use burn::record::BinBytesRecorder;
use burn::{
    nn::{
        Dropout, DropoutConfig, Linear, LinearConfig, Relu,
    }, prelude::*
};

use num_enum::{TryFromPrimitive, IntoPrimitive};

// Basic model structure at the time of writing is like this
// Inputs:Outputs
// MODEL_INPUTS:10 -> 10:10 -> 10:Output::COUNT
// All are linear transformations

// The number of inputs the model intakes
pub const MODEL_INPUTS: usize = 192;

// The level of precision that the model should be stored at. Should be half on embedded device
pub type PrecisionSetting = HalfPrecisionSettings;

/// The possible outputs of the model. Update this if more is wanted.
#[derive(Clone, Debug, PartialEq)]
#[derive(TryFromPrimitive, IntoPrimitive)] // This creates helpers to convert the enum to and from numbers 
#[repr(usize)] // That conversion will be to and from usize
pub enum Output {
    Flex,
    Relax,

    Unknown, // Added for logging abilities in devil-embedded, don't add to count
}

// Always update this if changing the number of Output items
impl Output {
    pub const COUNT: usize = 2;

    pub fn from_str(string: &str) -> Option<Self> {
        match string {
            "flex" => Some(Self::Flex),
            "relax" => Some(Self::Relax),
            _ => None
        }
    }
}

// Defines all of the different layers and fields being used, along with the device.
// Read more about this all in burn.dev
#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    linear1: Linear<B>,
    linear2: Linear<B>,
    linear3: Linear<B>,
    linear4: Linear<B>,
    dropout: Dropout,
    activation: Relu,
}

impl<B: Backend> Model<B> {
    // Create a model from embedded states, or a .bin file, as seen in the inference/lib.rs
    pub fn from_embedded(device: &B::Device, embedded_states: &[u8]) -> Self {
        // This reads the file and treats the model as a full precision one.
        // It then loads the stats into a vector on the heap, and initializes the model.
        let record = BinBytesRecorder::<PrecisionSetting>::default()
            .load(embedded_states.to_vec(), device)
            .expect("Should decode state successfully");

        // Returns a new device with the record
        Self::new(device).load_record(record)
    }

    // Creates a new Model with no states, initialized to whatever device backend is provided.
    // This is where the model really is defined, and as such, most optomizations to the model
    // should be placed here.
    pub fn new(device: &B::Device) -> Self {
        Self {
            linear1: LinearConfig::new(MODEL_INPUTS, 10).init(device),
            linear2: LinearConfig::new(10, 10).init(device),
            linear3: LinearConfig::new(10, 10).init(device),
            linear4: LinearConfig::new(10, Output::COUNT).init(device),
            activation: Relu::new(),
            dropout: DropoutConfig::new(0.2).init()
        }
    }

    // Creates the model prediction function. Transforms the model with each opperation,
    // then returns it.
    #[allow(clippy::approx_constant)]
    // Takes in a 1d Tensor, then outputs a 1d Tensor
    pub fn forward(&self, input: Tensor<B, 1>) -> Tensor<B, 1> {
        let x = self.linear1.forward(input); // Run first linear transformation
        let x = self.dropout.forward(x); // Remove a random 50% of the nodes
        let x = self.activation.forward(x); // Run the relu function on the result
        let x = self.linear2.forward(x); // Run second linear transformation
        let x = self.dropout.forward(x); // Remove a random 50% of the nodes
        let x = self.activation.forward(x); // Run the relu function on the result

        let x = self.linear3.forward(x); // Run second linear transformation
        let x = self.dropout.forward(x); // Remove a random 50% of the nodes
        let x = self.activation.forward(x); // Run the relu function on the result

        self.linear4.forward(x) // Return the result of the final linear layer,
        // Could also do a softmax here, but really there is no reason, whichever number is
        // the largest, that one is what the model predicts to be the best.
    }
}
