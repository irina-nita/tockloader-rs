// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

mod cli;
mod errors;

use cli::make_cli;
use errors::TockloaderError;
use tockloader_lib::{info_probe, list_probe};

#[tokio::main]
async fn main() -> Result<(), TockloaderError> {
    let result = run().await;
    if let Err(e) = &result {
        eprintln!("\n{}", e);
    }
    result
}

async fn run() -> Result<(), TockloaderError> {
    let matches = make_cli().get_matches();

    if matches.get_flag("debug") {
        println!("Debug mode enabled");
    }

    match matches.subcommand() {
        Some(("listen", _sub_matches)) => {
            match tock_process_console::run().await {
                Ok(()) => {}
                Err(_) => {
                    print!("cli bricked!")
                }
            };
        }
        Some(("list", sub_matches)) => {
            // TODO(NegrilaRares) Result handle
            let _apps_details = match list_probe(sub_matches).await {
                Ok(apps_details) => apps_details,
                Err(e) => panic!("While listing apps encountered: {}", e),
            };
        }
        Some(("install", _sub_matches)) => {}
        Some(("info", sub_matches)) => {
            let _attributes = match info_probe(sub_matches).await {
                Ok(attributes) => attributes,
                Err(e) => panic!("While listing board info encountered: {}", e),
            };
        }

        // If only the "--debug" flag is set, then this branch is executed
        // Or, more likely at this stage, a subcommand hasn't been implemented yet.
        _ => {
            println!("Could not run the provided subcommand.");
            _ = make_cli().print_help();
        }
    }

    Ok(())
}
