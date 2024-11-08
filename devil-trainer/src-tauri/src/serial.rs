use std::time::Duration;

use anyhow::Result;
use serialport::SerialPort;
use tauri::{http::response, Emitter};

use crate::GLOBAL_APP_HANDLE;

struct Serial {
    port: Box<dyn SerialPort>,
}

impl Serial {
    pub fn connect(port: &str) -> Result<Self> {
        let port = serialport::new("/dev/ttyACM0", 115_200)
            .timeout(Duration::from_millis(10))
            .open()?;

        Ok(Self { port })
    }
}

// Starting serial port connection
// Should be part of struct
pub fn start(port: &str) -> bool {
    let port = serialport::new("/dev/ttyACM0", 115_200)
        .timeout(Duration::from_millis(10))
        .open();

    let mut port = match port {
        Ok(port) => port,
        Err(err) => {
            println!("err: {:?}", err);
            return false;
        }
    };

    loop {
        let mut serial_buf: Vec<u8> = vec![0; 32];

        match port.read(serial_buf.as_mut_slice()) {
            Ok(_) => {
                // Process the data here
                if let Ok(data) = String::from_utf8(serial_buf) {
                    let response = GLOBAL_APP_HANDLE.get().unwrap().emit("serial-data", data);
                    println!("{:?}", response);
                }
            }
            Err(err) => {
                println!("Error reading from port: {:?}", err);
            }
        }

        // Optionally sleep to prevent constant polling if needed
        std::thread::sleep(Duration::from_millis(10));
    }
}

// This should actually be part of a struct and drop the current connection
// will be implemented in the future
pub fn stop() -> bool {
    true
}
