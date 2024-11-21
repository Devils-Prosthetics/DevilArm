fn main() {
    let out = directories::ProjectDirs::from("com", "Devils Prosthetics", "devil-ml").unwrap();

    let out = out.data_dir();
    let artifact_dir = out.to_str().unwrap();

    println!("cargo:rustc-env=ARTIFACT_DIR={}", artifact_dir);
}
