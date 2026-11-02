use std::process::ExitCode;

use daedalus_runtime_engine::RuntimeEngine;
use daedalus_runtime_protocol::{RuntimeRequest, CURRENT_PROTOCOL_VERSION};
use serde_json::json;

fn main() -> ExitCode {
    let command = std::env::args().nth(1);

    let method = match command.as_deref() {
        Some("hardware") => "hardware.discover",
        Some("info") => "runtime.getInfo",
        Some("ping") => "runtime.ping",
        Some("help") | None => {
            print_help();
            return ExitCode::SUCCESS;
        }
        Some(unknown) => {
            eprintln!("Unknown command: {unknown}");
            print_help();
            return ExitCode::from(2);
        }
    };

    let request = RuntimeRequest {
        protocol_version: CURRENT_PROTOCOL_VERSION,
        request_id: "daedalus-cli".to_owned(),
        method: method.to_owned(),
        params: json!({}),
    };

    let mut engine = RuntimeEngine::new();
    let response = engine.dispatch(request);

    match serde_json::to_string_pretty(&response) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("Could not serialize the runtime response: {error}");
            return ExitCode::FAILURE;
        }
    }

    if response.ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn print_help() {
    println!(
        "Daedalus CLI\n\
         Usage: daedalus <command>\n\n\
         Commands:\n\
           hardware   Discover operating system, CPU, and memory information\n\
           info       Display runtime version and implemented capabilities\n\
           ping       Check runtime health\n\
           help       Show this help"
    );
}
