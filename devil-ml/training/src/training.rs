use burn::prelude::*;
use burn::tensor::backend::AutodiffBackend;
use burn::train::ClassificationOutput;
use burn::train::TrainOutput;
use burn::train::TrainStep;
use burn::train::ValidStep;

use model::Model;
use nn::loss::CrossEntropyLossConfig;

use crate::data::DevilBatch;

// Allows for us to implement forward_step on the Model, despite Model not being defined within this crate.
pub trait ModelForwardStep<B: Backend> {
    fn forward_step(&self, item: DevilBatch<B>) -> ClassificationOutput<B>;
}

// Implement forward_step, this will be used for training and validation steps.
impl<B: Backend> ModelForwardStep<B> for Model<B> {
    // This function will runn for each train step and validation step
    fn forward_step(&self, item: DevilBatch<B>) -> ClassificationOutput<B> {
        // The point of the conversion from inputs to outputs with chunks is the following
        // item.inputs will be an tensor like this [[f32; MODEL_INPUTS]; 10], we need to convert this into
        // Vector of tensors like [f32; MODEL_INPUTS] so that we can call self.forward on all of them
        // From there we need to convert them back into [[f32; MODEL_INPUTS]; 1] (unsqueeze does this)
        // and then into [[f32; MODEL_INPUTS]; 10] with cat
        // So in summery [[f32; MODEL_INPUTS]; 10] -> [f32; MODEL_INPUTS] -> [[f32; MODEL_INPUTS]; 1] -> [[f32; MODEL_INPUTS]; 10]

        let num_chunks = item.inputs.dims()[0]; // Gets the number of elements in the tensor
        let chunks = item.inputs.chunk(num_chunks, 0); // chunks them into that number of elements

        // Shrinks the tensors into a 1d tensor
        let inputs: Vec<_> = chunks
            .into_iter()
            .map(|item| item.squeeze::<1>(0))
            .collect();

        // Runs the tensors through self.forward, then converts them into 2d tensors
        let outputs: Vec<_> = inputs
            .into_iter()
            .map(|input| self.forward(input).unsqueeze())
            .collect();

        // Adds all the tensors back together, so that it is the same as what came in, except passed through forward
        let outputs: Tensor<B, 2> = Tensor::cat(outputs, 0);

        let targets: Tensor<B, 1, Int> = item.targets;

        // get the targets and outputs, then get the result from running the forward function, use that to calculate the loss

        // Loss calculation is using cross entropy loss function, read more here
        // https://pytorch.org/docs/stable/generated/torch.nn.CrossEntropyLoss.html
        let loss = CrossEntropyLossConfig::new()
            .init(&outputs.device())
            .forward(outputs.clone(), targets.clone());

        // Return the results of that calc to let burn know if the gradient descent used was successful.
        ClassificationOutput {
            loss,
            output: outputs,
            targets,
        }
    }
}

// Implements TrainStep, which is used by burn to determine if the descent was successful
impl<B: AutodiffBackend> TrainStep<DevilBatch<B>, ClassificationOutput<B>> for Model<B> {
    fn step(&self, item: DevilBatch<B>) -> TrainOutput<ClassificationOutput<B>> {
        let item = self.forward_step(item);

        // To be honest, not quite sure why the loss needs to be backwards
        TrainOutput::new(self, item.loss.backward(), item)
    }
}

// Same thing but for validation. Remember, validation will run every epoch.
impl<B: Backend> ValidStep<DevilBatch<B>, ClassificationOutput<B>> for Model<B> {
    fn step(&self, item: DevilBatch<B>) -> ClassificationOutput<B> {
        self.forward_step(item)
    }
}
