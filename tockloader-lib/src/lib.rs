// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

pub mod attributes;
mod board_settings;
mod errors;
pub mod probe_session;
pub mod tab;

use attributes::app_attributes::AppAttributes;
use attributes::get_board_attributes::{get_all_attributes, get_bootloader_version};
use attributes::get_kernel_attributes::{get_kernel_version, kernel_attributes};
use attributes::hardware_attributes::HardwareAttributes;
use board_settings::BoardSettings;

use clap::ArgMatches;
use errors::TockloaderError;
use inquire::Select;
use probe_rs::probe::list::Lister;
use probe_rs::probe::DebugProbeInfo;
use probe_rs::{MemoryInterface, Permissions};
use probe_session::ProbeSession;
use tab::TabFile;
use tbf_parser::parse::*;
use tbf_parser::types::*;

pub async fn list_probe(sub_matches: &ArgMatches) -> Result<Vec<AppAttributes>, TockloaderError> {
    let lister = Lister::new();
    let probes = lister.list_all();
    println!("Probes: {:?}\n", probes);

    let ans = Select::new("Which probe do you want to use?", probes).prompt();

    let mut apps_details: Vec<AppAttributes> = vec![];

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
                let mut temp_details: AppAttributes = AppAttributes::new();

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
                            temp_details.tbf_version = Some(ver);
                            temp_details.header_size = Some(header_size);
                            temp_details.total_size = Some(total_size);
                            (ver, header_size, total_size)
                        }
                        _ => break, // No more apps
                    };

                let header = parse_tbf_header(&buff[0..header_size as usize], ver);
                match header {
                    Ok(header) => {
                        temp_details.enabled = Some(header.enabled());
                        temp_details.minumum_ram_size = Some(header.get_minimum_app_ram_size());
                        temp_details.name = Some(
                            header
                                .get_package_name()
                                .expect("Package name not found.")
                                .to_string(),
                        );

                        temp_details.kernel_version = Some(
                            header
                                .get_kernel_version()
                                .expect("Could not get kernel version."),
                        );
                        // temp_details.kernel_version = Some(format!(
                        //     "{}.{}",
                        //     header
                        //         .get_kernel_version()
                        //         .expect("Kernel version not found.")
                        //         .0,
                        //     header
                        //         .get_kernel_version()
                        //         .expect("Kernel version not found.")
                        //         .1
                        // ));
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

pub async fn info_probe(sub_matches: &ArgMatches) -> Result<HardwareAttributes, TockloaderError> {
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

            let mut attributes: HardwareAttributes = HardwareAttributes::new();

            get_all_attributes(&mut core, &mut attributes);

            get_bootloader_version(&mut core, &mut attributes);

            kernel_attributes(&mut core, &mut attributes);

            println!("{:?}", attributes);

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
    chip: &str,
    core_index: &usize,
    tab_file: TabFile,
) -> Result<(), TockloaderError> {
    // Open port and configure it
    let mut probe_session = ProbeSession::new(choice, board, chip);
    let mut address = probe_session
        .address
        .expect("address could not be retreved from ProbeSession.");
    let mut core = probe_session.get_core(*core_index);

    // Jump through the linked list of apps to check the address to install the app
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

    if tab_file.is_compatible_with_board(board) {
        println!("Specified tab is compatible with board.");
    } else {
        println!("Specified tab is not compatible with board.");
    }

    if tab_file.is_compatible_with_kernel_verison(get_kernel_version(&mut core) as f32) {
        println!("Specified tab is compatible with your kernel version.");
    } else {
        println!("Specified tab is not compatible with your kernel version.");
    }
    Ok(())
}
