use tauri::{AppHandle, Emitter};
use once_cell::sync::OnceCell;
use std::process::Command;
mod serial;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

// Starts the serial connection in a new thread.
#[tauri::command]
fn serial_start(port: String) -> bool {
    let port_clone = port.clone();
    
    std::thread::spawn(move || {
        if let wow = serial::start(&port_clone) {
            eprintln!("Error starting serial port: {:?}", wow);
        }
    });

    true // Return true to signal thread was spawned successfully
}

// Stops the serial connection
#[tauri::command]
fn serial_stop() -> bool {
    serial::stop()
}

// Here for reference
#[tauri::command]
fn download(app: AppHandle, serial_data: String) {
    app.emit("serial-output", &serial_data).unwrap();
}

// Runs the DevilArm project, will eventually just take a path as input
#[tauri::command]
fn upload_file_to_pi() -> Result<String, String> {
    // Specify the directory where you want to run the cargo command
    let directory = "../../DevilArm/devil-embedded";

    // Run `cargo run --manifest-path /path/to/your/cargo/project/Cargo.toml`
    let output = Command::new("cargo")
        .arg("run")
        .current_dir(directory)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                Ok(format!("Cargo run success: {}", stdout))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Cargo run failed: {}", stderr))
            }
        }
        Err(e) => Err(format!("Failed to execute cargo run: {}", e.to_string())),
    }
}

// Enables other functions to use app handler, notably serial
pub static GLOBAL_APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(move |app| {
            // Set the global app handler once in setup.
            GLOBAL_APP_HANDLE.set(app.handle().clone())
                .expect("Failed to set global app handle");
            
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![serial_start, serial_stop, upload_file_to_pi])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
