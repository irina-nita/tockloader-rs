// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

mod cli;
mod errors;
mod interfaces;

use cli::make_cli;
use errors::TockloaderError;
<<<<<<< HEAD
=======
use interfaces::{build_interface, traits::*};
use tokio::time::sleep;
use probe_rs::probe::list::Lister;
use probe_rs::{Permissions, MemoryInterface};
use tbf_parser::parse::*;

>>>>>>> 717bc32 (tried list implementation)
use tock_process_console;

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
            let _ = match tock_process_console::run().await {
                Ok(()) => {}
                Err(_) => {
                    print!("cli bricked!")
                }
            };
        }
        Some(("list", _sub_matches)) => {
            list_probes().await?;
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

async fn list_probes() -> Result<(), TockloaderError> {
    
    let lister = Lister::new();
    let probes = lister.list_all();
    println!("Probes: {:?}\n", probes);

    let probe = probes[0].open().unwrap();
    let mut session = probe.attach("nRF52805_xxAA", Permissions::default()).unwrap();

    println!("Session target: {:?}\n", session.target());
    println!("Session interfaces: {:?}\n", session.architecture());
    println!("Session core: {:?}\n", session.list_cores());

    let mut core = session.core(0).unwrap();

    // Read a block of 200 8-bit words
    let mut buff = vec![0u8; 200];
    core.read(0x0004_0000, &mut buff).unwrap();
    println!("Data: {:?}\n", buff);

    let (ver, header_len, whole_len) = parse_tbf_header_lengths(&buff[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    println!("Version: {:?}\n", ver);
    println!("Header length: {:?}\n", header_len);
    println!("Whole length: {:?}\n", whole_len);

    let header = parse_tbf_header(&buff[0..header_len as usize], ver).unwrap();

    println!("Enabled: {:?}\n", header.enabled());
    println!("Minimum App Ram Size: {:?}\n", header.get_minimum_app_ram_size());
    println!("Init function offset: {:?}\n", header.get_init_function_offset());
    println!("Protected size: {:?}\n", header.get_protected_size());
    println!("Package name: {:?}\n", header.get_package_name().unwrap());
    println!("Kernel version: {:?}\n", header.get_kernel_version().unwrap());

    Ok(())
}