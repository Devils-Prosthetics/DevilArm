use burn::data::dataloader::DataLoaderBuilder;
use burn::module::Module;
use burn::optim::AdamConfig;
use burn::prelude::Config;
use burn::record::{BinFileRecorder, CompactRecorder};
use burn::tensor::backend::AutodiffBackend;
use burn::train::metric::AccuracyMetric;
use burn::train::metric::LossMetric;
use burn::train::renderer::MetricState;
use burn::train::renderer::MetricsRenderer;
use burn::train::renderer::TrainingProgress;
use burn::train::LearnerBuilder;
use data::DevilBatcher;
use data::DevilDataset;
use devil_ml_model::Model;
use devil_ml_model::Output;
use devil_ml_model::PrecisionSetting;

pub mod data;
pub mod training;

// Uses a macro to add lots of functionality to this config, as seen in
// https://burn.dev/burn-book/building-blocks/config.html
#[derive(Config)]
pub struct TrainingConfig {
    pub optimizer: AdamConfig,
    #[config(default = 2)]
    pub num_epochs: usize,
    #[config(default = 7)]
    // On going bug here https://github.com/tracel-ai/burn/issues/1970 seems like this is the max size
    pub batch_size: usize,
    #[config(default = 4)]
    pub num_workers: usize,
    #[config(default = 42)]
    pub seed: u64,
    #[config(default = 1.0e-4)]
    pub learning_rate: f64,
    #[config(default = true)]
    pub custom_renderer: bool,
}

// Removes, then creates the directory for the output of the model
fn create_artifact_dir(artifact_dir: &str) {
    // Remove existing artifacts before to get an accurate learner summary
    std::fs::remove_dir_all(artifact_dir).ok();
    std::fs::create_dir_all(artifact_dir).ok();
}

// This renderer exists, because the default, nicer looking one breaks the build.rs script
struct CustomRenderer {}

// Implements the necessary trait for the renderer to be accepted
impl MetricsRenderer for CustomRenderer {
    fn update_train(&mut self, _state: MetricState) {}

    fn update_valid(&mut self, _state: MetricState) {}

    fn render_train(&mut self, item: TrainingProgress) {
        dbg!(item); // Basically just logs the item
    }

    fn render_valid(&mut self, item: TrainingProgress) {
        dbg!(item); // Basically just logs the item
    }
}

/// Trains the model and outputs all of the byproducts to artifact_dir, using the specified backend device.
pub fn train<B: AutodiffBackend>(
    artifact_dir: &str,
    config: TrainingConfig,
    device: B::Device,
) -> Model<B> {
    create_artifact_dir(artifact_dir);
    config
        .save(format!("{artifact_dir}/config.json"))
        .expect("Config should be saved successfully");

    // Set the seed for the random number generator used by the backend.
    B::seed(config.seed);

    // Create the batcher for the training data
    let batcher_train = DevilBatcher::<B>::new(device.clone());
    // Create the batcher for the validation data
    let batcher_valid = DevilBatcher::<B::InnerBackend>::new(device.clone());

    // Get the train and test csv's as &str's
    let train_input = include_str!(concat!(env!("OUT_DIR"), "/data/train.csv"));
    let validation_input = include_str!(concat!(env!("OUT_DIR"), "/data/validation.csv"));

    // Instantiate the dataset from the text
    let dataset_train = DevilDataset::new(train_input);
    let dataset_validation = DevilDataset::new(validation_input);

    // Creates a DataLoader for the train dataset, it batches it into items to send to the worker,
    // after shuffling. Think of a worker as a seperate process/thread to run the model training on
    let dataloader_train = DataLoaderBuilder::new(batcher_train)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(dataset_train);

    // Same thing as above, but for the validation data
    let dataloader_validation = DataLoaderBuilder::new(batcher_valid)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(dataset_validation);

    // Creates a learner, which is then used to
    let learner = LearnerBuilder::new(artifact_dir)
        // Not implementing these yet, might in the future if needed
        .metric_train_numeric(AccuracyMetric::new())
        .metric_valid_numeric(AccuracyMetric::new())
        .metric_train_numeric(LossMetric::new()) // Get the loss for the model
        .metric_valid_numeric(LossMetric::new())
        .with_file_checkpointer(CompactRecorder::new()) // Creates checkpoints for the model, useful for DevilArmTrainer
        .devices(vec![device.clone()]) // Lists off the devices able to be used, only one here
        .num_epochs(config.num_epochs) // Set the number of epochs to train
        .with_application_logger(None)
        .summary(); // Print out the final results of the model training

    // If custom renderer is defined, then use the custom renderer
    let learner = if config.custom_renderer {
        learner.renderer(CustomRenderer {})
    } else {
        learner
    };

    let learner = learner.build(
        Model::new(&device),
        config.optimizer.init(),
        config.learning_rate,
    );

    // Train the model, and output the fully trained model
    let model_trained = learner.fit(dataloader_train, dataloader_validation);

    // Setup full precision bin file recorder, to write the trained model to.
    let recorder = BinFileRecorder::<PrecisionSetting>::new();

    // Write the model using that recorder, and save results in the artifact_dir/model
    model_trained
        .clone()
        .save_file(format!("{artifact_dir}/model"), &recorder)
        .expect("Trained model should be saved successfully");

    model_trained
}
