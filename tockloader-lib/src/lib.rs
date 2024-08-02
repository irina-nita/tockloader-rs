// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

mod attributes;
mod board_settings;
mod errors;
pub mod probe_session;
pub mod tab;

use std::fs::File;

use std::collections::HashMap;
use std::io::Read;

use attributes::board_attributes::{get_all_attributes, get_bootloader_version};
use attributes::kernel_attributes::kernel_attributes;
use board_settings::BoardSettings;

use clap::ArgMatches;
use errors::TockloaderError;
use inquire::Select;
use probe_rs::probe::list::Lister;
use probe_rs::probe::DebugProbeInfo;
use probe_rs::{MemoryInterface, Permissions};
use probe_session::ProbeSession;
use tab::TabFile;
use tar::Archive;
use tbf_parser::parse::*;
use tbf_parser::types::*;
use utf8_decode::Decoder;

pub async fn list_probe(
    sub_matches: &ArgMatches,
) -> Result<Vec<HashMap<String, String>>, TockloaderError> {
    let lister = Lister::new();
    let probes = lister.list_all();
    println!("Probes: {:?}\n", probes);

    let ans = Select::new("Which probe do you want to use?", probes).prompt();

    let mut apps_details: Vec<HashMap<String, String>> = vec![];

    match ans {
        Ok(choice) => {
            let probe = choice.open().unwrap();

            let chip = sub_matches.get_one::<String>("chip").unwrap();
            let board = sub_matches.get_one::<String>("board").unwrap();

            let board_settings = BoardSettings::new(board.clone(), chip.clone());

            let mut session = probe
                .attach(board_settings.chip, Permissions::default())
                .unwrap();

            let core_index = sub_matches.get_one::<usize>("core").unwrap();

            let mut core = session.core(*core_index).unwrap();

            let mut address = board_settings.start_address;

            // Jump through the linked list of apps
            let mut apps_counter = 0;
            loop {
                let mut temp_details: HashMap<String, String> = HashMap::new();

                // Read a block of 200 8-bit words
                let mut buff = vec![0u8; 200];
                match core.read(address, &mut buff) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error reading memory: {:?}", e);
                        break;
                    }
                }

                let (ver, header_size, total_size) =
                    match parse_tbf_header_lengths(&buff[0..8].try_into().unwrap()) {
                        Ok((ver, header_size, total_size)) if header_size != 0 => {
                            temp_details.insert("version".to_owned(), ver.to_string());
                            temp_details.insert("header_size".to_owned(), header_size.to_string());
                            temp_details.insert("total_size".to_owned(), total_size.to_string());
                            (ver, header_size, total_size)
                        }
                        _ => break, // No more apps
                    };

                let header = parse_tbf_header(&buff[0..header_size as usize], ver);
                match header {
                    Ok(header) => {
                        temp_details.insert("enabled".to_owned(), header.enabled().to_string());
                        temp_details.insert(
                            "minumum_ram_size".to_owned(),
                            header.get_minimum_app_ram_size().to_string(),
                        );
                        temp_details.insert(
                            "name".to_owned(),
                            header
                                .get_package_name()
                                .expect("Package name not found.")
                                .to_string(),
                        );
                        temp_details.insert(
                            "kernel_version".to_owned(),
                            format!(
                                "{}.{}",
                                header
                                    .get_kernel_version()
                                    .expect("Kernel version not found.")
                                    .0,
                                header
                                    .get_kernel_version()
                                    .expect("Kernel version not found.")
                                    .1
                            ),
                        );
                    }
                    // TODO(MicuAna): refactor when reworking errors
                    Err(TbfParseError::ChecksumMismatch(
                        provided_checksum,
                        calculated_checksum,
                    )) => {
                        println!(
                            "Checksum mismatch: provided = {}, calculated = {}",
                            provided_checksum, calculated_checksum
                        );
                        break;
                    }
                    Err(e) => {
                        println!("Failed to parse TBF header: {:?}", e);
                        break;
                    }
                }
                apps_details.insert(apps_counter, temp_details);
                apps_counter += 1;
                address += total_size as u64;
            }
        }
        Err(err) => println!("{}", err),
    }

    Ok(apps_details)
}

pub async fn info_probe(
    sub_matches: &ArgMatches,
) -> Result<HashMap<String, String>, TockloaderError> {
    let lister = Lister::new();
    let probes = lister.list_all();

    let ans = Select::new("Which probe do you want to use?", probes).prompt();
    match ans {
        Ok(choice) => {
            let probe = choice.open().unwrap();

            let chip = sub_matches.get_one::<String>("chip").unwrap();
            let board = sub_matches.get_one::<String>("board").unwrap();

            let board_settings = BoardSettings::new(board.clone(), chip.clone());

            let mut session = probe
                .attach(board_settings.chip, Permissions::default())
                .unwrap();

            let core_index = sub_matches.get_one::<usize>("core").unwrap();

            let mut core = session.core(*core_index).unwrap();

            let mut attributes = get_all_attributes(&mut core);

            let bootloader_version = get_bootloader_version(&mut core);

            let kernel_attributes = kernel_attributes(&mut core, &mut attributes);

            attributes.insert("bootloader_version".to_owned(), bootloader_version);

            attributes.extend(kernel_attributes.into_iter());

            println!("{:?}", attributes);

            // println!("Bootloader Version: {}", bootloader_version);

            Ok(attributes)
        }
        Err(err) => {
            println!("While picking probe:{}", err);
            //TODD(NegrilaRares) JUST TEMPLATE ERROR MUST BE CHANGED LATER
            Err(TockloaderError::NoPortAvailable)
        }
    }
}

pub async fn install_app(
    choice: DebugProbeInfo,
    board: &String,
    chip: &String,
    core_index: &usize,
    kernel_version: &String, /*this is temporary*/
    tab_file: TabFile,
) -> Result<(), TockloaderError> {
    // Get hard-coded start_address
    let board_settings = BoardSettings::new(board.clone(), chip.clone());
    let mut address = board_settings.start_address;

    let mut probe_session = ProbeSession::new(choice, board_settings, *core_index);
    probe_session.open();
    let mut core = probe_session.get_core();
    // Jump through the linked list of apps
    loop {
        // Read a block of 200 8-bit words
        let mut buff = vec![0u8; 200];
        match core.read(address, &mut buff) {
            Ok(_) => {}
            Err(e) => {
                println!("Error reading memory: {:?}", e);
                break;
            }
        }
        let (_ver, _header_len, whole_len) =
            match parse_tbf_header_lengths(&buff[0..8].try_into().unwrap()) {
                Ok((ver, header_len, whole_len)) if header_len != 0 => (ver, header_len, whole_len),
                _ => break, // No more apps
            };

        address += whole_len as u64;
    }

    let mut bytes = vec![0u8; 1024];
    match core.read(0x600, &mut bytes) {
        Ok(_) => {
            let mut i: u16 = 0;
            while i < 1024 {
                let decoder = Decoder::new(bytes[i as usize..(i + 8) as usize].iter().cloned());
                let mut key = String::new();
                for c in decoder {
                    key.push(c?);
                }
                println!("{}", key);
                let vlen = bytes[(i + 8) as usize];
                let index: u16 = vlen as u16 + 9;
                let decoder = Decoder::new(
                    bytes[(i + 9) as usize..(i + index) as usize]
                        .iter()
                        .cloned(),
                );
                let mut value = String::new();
                for c in decoder {
                    value.push(c?);
                }
                println!("{}", value);
                i = i + 64;
            }
        }
        Err(e) => {
            println!("Error reading memory: {:?}", e);
        }
    }

    let mut archive = Archive::new(File::open(tab_file.path).unwrap());
    for entry in archive.entries().unwrap().into_iter() {
        match entry {
            Ok(mut entry) => {
                if let Ok(path) = entry.path() {
                    if let Some(file_name) = path.file_name() {
                        if file_name == "metadata.toml" {
                            let mut buf = String::new();
                            entry.read_to_string(&mut buf).unwrap();
                            let replaced = buf.replace("\"", "");
                            let parts = replaced.split("\n");
                            let collection = parts.collect::<Vec<&str>>();

                            for item in collection {
                                if item.contains("only-for-boards") {
                                    let columns = item.split("=");
                                    let elem = columns.collect::<Vec<&str>>();
                                    let all_boards = elem[1].split(", ");
                                    let boards = all_boards.collect::<Vec<&str>>();
                                    for bd in boards {
                                        if bd == board {
                                            println!("App is compatible with board!");
                                            break;
                                        }
                                    }
                                    println! {"App is not compatible with board!"};
                                    break;
                                }
                                if item.contains("minimum-tock-kernel-version") {
                                    let columns = item.split("=");
                                    let elem = columns.collect::<Vec<&str>>();
                                    let kernver = elem[1];
                                    if kernver == kernel_version {
                                        println!("App is compatible with this kernel version!");
                                    } else {
                                        println! {"App is not compatible with this kernel version!"};
                                    }
                                    break;
                                }
                            }
                        }
                    } else {
                        eprintln!("Failed to get path");
                    }
                }
            }
            Err(e) => eprintln!("Failed to get entry: {}", e),
        }
    }

    Ok(())
}
