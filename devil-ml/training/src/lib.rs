use burn::train::renderer::TrainingProgress;
use burn::train::renderer::MetricsRenderer;
use burn::train::renderer::MetricState;
use burn::module::Module;
use burn::record::{BinFileRecorder, CompactRecorder, FullPrecisionSettings};
use burn::train::metric::LossMetric;
use burn::train::LearnerBuilder;
use burn::train::metric::AccuracyMetric;
use burn::data::dataloader::DataLoaderBuilder;
use data::DevilBatcher;
use data::DevilDataset;
use burn::tensor::backend::AutodiffBackend;
use burn::prelude::Config;
use burn::optim::AdamConfig;
use model::Model;

pub mod data;
pub mod training;

#[derive(Config)]
pub struct TrainingConfig {
    pub optimizer: AdamConfig,
    #[config(default = 10)]
    pub num_epochs: usize,
    #[config(default = 64)]
    pub batch_size: usize,
    #[config(default = 4)]
    pub num_workers: usize,
    #[config(default = 42)]
    pub seed: u64,
    #[config(default = 1.0e-4)]
    pub learning_rate: f64,
}

fn create_artifact_dir(artifact_dir: &str) {
    // Remove existing artifacts before to get an accurate learner summary
    std::fs::remove_dir_all(artifact_dir).ok();
    std::fs::create_dir_all(artifact_dir).ok();
}

struct CustomRenderer {}

impl MetricsRenderer for CustomRenderer {
    fn update_train(&mut self, _state: MetricState) {}

    fn update_valid(&mut self, _state: MetricState) {}

    fn render_train(&mut self, item: TrainingProgress) {
        dbg!(item);
    }

    fn render_valid(&mut self, item: TrainingProgress) {
        dbg!(item);
    }
}

pub fn train<B: AutodiffBackend>(artifact_dir: &str, config: TrainingConfig, device: B::Device) {
    create_artifact_dir(artifact_dir);
    config
        .save(format!("{artifact_dir}/config.json"))
        .expect("Config should be saved successfully");

    B::seed(config.seed);

    let batcher_train = DevilBatcher::<B>::new(device.clone());
    let batcher_valid = DevilBatcher::<B::InnerBackend>::new(device.clone());

    let train_input = include_str!("../data/train.csv");
    let test_input = include_str!("../data/test.csv");

    let dataset_train = DevilDataset::new(train_input);
    let dataset_test = DevilDataset::new(test_input);

    let dataloader_train = DataLoaderBuilder::new(batcher_train)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(dataset_train);

    let dataloader_test = DataLoaderBuilder::new(batcher_valid)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(dataset_test);

    let learner = LearnerBuilder::new(artifact_dir)
        // .metric_train_numeric(AccuracyMetric::new())
        // .metric_valid_numeric(AccuracyMetric::new())
        .metric_train_numeric(LossMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .with_file_checkpointer(CompactRecorder::new())
        .devices(vec![device.clone()])
        .num_epochs(config.num_epochs)
        .renderer(CustomRenderer {})
        .with_application_logger(None)
        .summary()
        .build(
            Model::new(&device),
            config.optimizer.init(),
            config.learning_rate,
        );

    let model_trained = learner.fit(dataloader_train, dataloader_test);

    let recorder = BinFileRecorder::<FullPrecisionSettings>::new();

    model_trained
        .save_file(format!("{artifact_dir}/model"), &recorder)
        .expect("Trained model should be saved successfully");

    println!("artifact_dir: {artifact_dir}/model");
}
