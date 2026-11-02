//! Command-line inspector for the local Daedalus runtime.
//!
//! This CLI invokes RuntimeEngine directly in-process. The separate
//! daedalus-runtime executable exposes the same engine over JSON stdio.

use std::process::ExitCode;

use daedalus_runtime_engine::RuntimeEngine;
use daedalus_runtime_protocol::{RuntimeRequest, CURRENT_PROTOCOL_VERSION};
use serde_json::json;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        print_help();
        return ExitCode::SUCCESS;
    };

    match command.as_str() {
        "help" | "-h" | "--help" => {
            print_help();
            return ExitCode::SUCCESS;
        }
        "-V" | "--version" => {
            println!("daedalus {VERSION}");
            return ExitCode::SUCCESS;
        }
        "hardware" | "info" | "ping" => {}
        _ => {
            eprintln!("Unknown command: {command}");
            eprintln!("Run daedalus help for usage.");
            return ExitCode::from(2);
        }
    }

    if let Some(unexpected) = args.next() {
        eprintln!("Unexpected argument: {unexpected}");
        eprintln!("Run daedalus help for usage.");
        return ExitCode::from(2);
    }

    let method = match command.as_str() {
        "hardware" => "hardware.discover",
        "info" => "runtime.getInfo",
        "ping" => "runtime.ping",
        _ => unreachable!("command was validated"),
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

    if response.ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

fn print_help() {
    println!(
        "Daedalus CLI {VERSION}\n\
         \n\
         Usage: daedalus <COMMAND>\n\
         \n\
         Commands:\n\
           hardware   Discover platform, CPU, memory, storage, GPU and sensors\n\
           info       Show runtime version, protocol version and capabilities\n\
           ping       Check the in-process runtime engine\n\
           help       Show this help\n\
         \n\
         Options:\n\
           -h, --help       Show help\n\
           -V, --version    Show version\n\
         \n\
         Results from runtime commands are pretty-printed JSON."
    );
}
