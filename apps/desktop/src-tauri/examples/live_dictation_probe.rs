use std::{thread, time::Duration};

use verboscribe2_desktop_lib::AppService;

fn main() {
    let hold_ms = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse::<u64>().ok())
        .unwrap_or(5_000);

    let service = AppService::default();

    println!("settings: {:?}", service.settings());
    println!("app_status_before: {:?}", service.app_status());
    println!("runtime_before: {:?}", service.runtime_status());

    match service.start_dictation() {
        Ok(status) => println!("start_dictation: {:?}", status),
        Err(error) => {
            eprintln!("start_dictation_error: {error}");
            eprintln!("runtime_after_start_error: {:?}", service.runtime_status());
            std::process::exit(1);
        }
    }

    println!("recording_for_ms: {hold_ms}");
    thread::sleep(Duration::from_millis(hold_ms));

    match service.stop_dictation() {
        Ok(status) => println!("stop_dictation: {:?}", status),
        Err(error) => {
            eprintln!("stop_dictation_error: {error}");
            eprintln!("runtime_after_stop_error: {:?}", service.runtime_status());
            eprintln!("app_status_after_stop_error: {:?}", service.app_status());
            std::process::exit(1);
        }
    }

    println!("runtime_after: {:?}", service.runtime_status());
    println!("app_status_after: {:?}", service.app_status());
}
