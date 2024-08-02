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

use errors::TockloaderError;
use probe_rs::probe::DebugProbeInfo;
use probe_rs::MemoryInterface;
use probe_session::ProbeSession;
use tab::TabFile;
use tbf_parser::parse::*;
use tbf_parser::types::*;

pub async fn list_probe(
    choice: DebugProbeInfo,
    board: &str,
    chip: &str,
    core_index: &usize,
) -> Vec<AppAttributes> {
    let mut probe_session = ProbeSession::new(choice, board, chip);
    let mut address = probe_session
        .address
        .expect("address could not be retreved from ProbeSession.");
    let mut core = probe_session.get_core(*core_index);

    // Jump through the linked list of apps
    let mut apps_counter = 0;
    let mut apps_details: Vec<AppAttributes> = vec![];
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
            Err(TbfParseError::ChecksumMismatch(provided_checksum, calculated_checksum)) => {
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
    apps_details
}

pub async fn info_probe(
    choice: DebugProbeInfo,
    board: &str,
    chip: &str,
    core_index: &usize,
) -> HardwareAttributes {
    let mut probe_session = ProbeSession::new(choice, board, chip);

    let mut core = probe_session.get_core(*core_index);

    let mut attributes: HardwareAttributes = HardwareAttributes::new();

    get_all_attributes(&mut core, &mut attributes);

    get_bootloader_version(&mut core, &mut attributes);

    kernel_attributes(&mut core, &mut attributes);

    attributes
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
