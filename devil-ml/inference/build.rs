use burn_import::burn::graph::RecordType;
use burn_import::onnx::ModelGen;

fn main() {
    println!("cargo:rerun-if-changed=model/sine.onnx");

    generate_model();
}

fn generate_model() {
    // Generate the model code from the ONNX file.
    ModelGen::new()
        .input("model/sine.onnx")
        .out_dir("model/")
        .record_type(RecordType::Bincode)
        .embed_states(true)
        .run_from_script();
}
