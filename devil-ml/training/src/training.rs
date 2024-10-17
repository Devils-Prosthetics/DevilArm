use burn::train::RegressionOutput;
use burn::train::ValidStep;
use burn::train::TrainOutput;
use burn::tensor::backend::AutodiffBackend;
use burn::train::TrainStep;
use burn::prelude::*;

use model::Model;
use nn::loss::MseLoss;

use crate::data::DevilBatch;

pub trait ModelForwardStep<B: Backend> {
    fn forward_step(&self, item: DevilBatch<B>) -> RegressionOutput<B>;
}

impl<B: Backend> ModelForwardStep<B> for Model<B> {
    fn forward_step(&self, item: DevilBatch<B>) -> RegressionOutput<B> {
        let targets: Tensor<B, 2> = item.targets.unsqueeze_dim(1);
        let output: Tensor<B, 2> = self.forward(item.inputs);

        let loss = MseLoss::new().forward(output.clone(), targets.clone(), nn::loss::Reduction::Mean);

        RegressionOutput {
            loss,
            output,
            targets,
        }
    }
}

impl<B: AutodiffBackend> TrainStep<DevilBatch<B>, RegressionOutput<B>> for Model<B> {
    fn step(&self, item: DevilBatch<B>) -> TrainOutput<RegressionOutput<B>> {
        let item = self.forward_step(item);

        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> ValidStep<DevilBatch<B>, RegressionOutput<B>> for Model<B> {
    fn step(&self, item: DevilBatch<B>) -> RegressionOutput<B> {
        self.forward_step(item)
    }
}
