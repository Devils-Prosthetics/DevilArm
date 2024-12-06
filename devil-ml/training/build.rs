use csv::Writer;
use devil_ml_model::Output;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::env;
use std::error::Error;
use std::fs::create_dir_all;
use std::path::PathBuf;

// Macro which allows the user to print out to console despite being in a build.rs file
// Comes from https://github.com/rust-lang/cargo/issues/985#issuecomment-1071667472
macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let arm_data = include_str!("./data/savannah's arm v4.csv");
    let mut parse_arm_data = parse_csv(arm_data);

    parse_arm_data.iter().take(10).for_each(|item| {
        println!(
            "inputs: {:?}, label: {:?}",
            item.inputs.iter().take(10).collect::<Vec<_>>(),
            item.label
        );
    });

    let mut rng = thread_rng();
    parse_arm_data.shuffle(&mut rng);
    parse_arm_data.iter().take(10).for_each(|item| {
        println!(
            "inputs: {:?}, label: {:?}",
            item.inputs.iter().take(10).collect::<Vec<_>>(),
            item.label
        );
    });

    println!("parse_arm_data_length: {:?}", parse_arm_data.len());

    let train_data = parse_arm_data
        .drain(0..(parse_arm_data.len() as f32 * 0.60) as usize)
        .collect::<Vec<_>>();
    let validation_data = parse_arm_data
        .drain(0..(parse_arm_data.len() / 2))
        .collect::<Vec<_>>();
    let testing_data = parse_arm_data;

    println!("train_data_length: {:?}", train_data.len());
    println!("validation_data_length: {:?}", validation_data.len());
    println!("testing_data_length: {:?}", testing_data.len());

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let data_folder = out_path.join("data");
    create_dir_all(&data_folder)?;
    write_csv(train_data, data_folder.join("train.csv"))?;
    write_csv(validation_data, data_folder.join("validation.csv"))?;
    write_csv(testing_data, data_folder.join("testing.csv"))?;

    println!(
        "out_dir: {:?}",
        out_path
            .to_str()
            .expect("Failed to convert to string for whatever reason")
    );

    Ok(())
}

fn write_csv(data: Vec<DevilItem>, path: PathBuf) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;

    for item in data {
        let inputs = item
            .inputs
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>();
        let label = item.label.to_str().to_string();

        wtr.write_record(inputs.iter().chain(std::iter::once(&label)))?;
    }

    wtr.flush()?;

    Ok(())
}

#[derive(Clone, Debug)]
pub struct DevilItem {
    pub inputs: Vec<f32>,
    pub label: Output,
}

pub fn parse_csv(input: &str) -> Vec<DevilItem> {
    // Initialize csv reader
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false) // Our files have no headers
        .from_reader(input.as_bytes()); // the string as bytes is the input

    // The vector which will store all the DevilItems
    let mut output: Vec<DevilItem> = Vec::new();

    // For each row in the csv
    for row in rdr.records() {
        // We have to match, because if records is empty, technically this will be an error
        match row {
            Ok(row) => {
                // Just converts each item in the row to a &str
                let values: Vec<&str> = row.iter().collect();

                // Split the last item of the vector into label, and the rest into inputs, if there is a None
                // returned from split_last, just inform that the row has an issue, then skip it.
                let Some((label, inputs)) = values.split_last() else {
                    eprintln!("row is improperly formatted");
                    continue;
                };

                // Parse all of the inputs into f32's
                let inputs: Vec<f32> = inputs
                    .into_iter()
                    .map(|s| s.parse::<f32>().expect("Found non number in csv")) // Filter out invalid f32
                    .collect();

                // parsing label to Output
                let label = label.to_string();
                let label = match Output::from_str(&label) {
                    Some(val) => val,
                    None => {
                        println!("Failed to parse label '{}'", label);
                        continue;
                    },
                };

                // Push the item to the vector
                output.push(DevilItem {
                    inputs, // First 196 items as f32
                    label,  // Output
                });
            }
            Err(_) => continue,
        };
    }

    return output;
}
