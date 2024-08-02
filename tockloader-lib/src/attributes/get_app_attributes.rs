// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use probe_rs::{Core, MemoryInterface};
use tbf_parser::{
    parse::{parse_tbf_header, parse_tbf_header_lengths},
    types::TbfParseError,
};

use super::{app_attributes::AppAttributes, get_board_attributes::get_appaddr};

pub(crate) fn get_apps_data(board_core: &mut Core) -> Vec<AppAttributes> {
    let mut address: u64 = get_appaddr(board_core).expect("Could not find app address.");
    let mut apps_counter = 0;
    let mut apps_details: Vec<AppAttributes> = vec![];

    loop {
        let mut details: AppAttributes = AppAttributes::new();

        let mut buff = vec![0u8; 200];
        match board_core.read(address, &mut buff) {
            Ok(_) => {}
            Err(e) => {
                println!("Error reading memory: {:?}", e);
                break;
            }
        }

        let (ver, header_size, total_size) =
            match parse_tbf_header_lengths(&buff[0..8].try_into().unwrap()) {
                Ok((ver, header_size, total_size)) if header_size != 0 => {
                    details.tbf_version = Some(ver);
                    details.header_size = Some(header_size);
                    details.total_size = Some(total_size);
                    (ver, header_size, total_size)
                }
                _ => break,
            };

        let header = parse_tbf_header(&buff[0..header_size as usize], ver);
        match header {
            Ok(header) => {
                details.enabled = Some(header.enabled());
                details.minumum_ram_size = Some(header.get_minimum_app_ram_size());
                details.name = Some(
                    header
                        .get_package_name()
                        .expect("Package name not found.")
                        .to_string(),
                );

                details.kernel_version = Some(
                    header
                        .get_kernel_version()
                        .expect("Could not get kernel version."),
                );
                // details.kernel_version = Some(format!(
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
        apps_details.insert(apps_counter, details);
        apps_counter += 1;
        address += total_size as u64;
    }
    apps_details
}

// TODO(MicuAna): add error handling
pub(crate) fn get_start_address(board_core: &mut Core, mut address: u64) -> Option<u64> {
    loop {
        // Read a block of 200 8-bit words
        let mut buff = vec![0u8; 200];
        match board_core.read(address, &mut buff) {
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
    Some(address)
}
