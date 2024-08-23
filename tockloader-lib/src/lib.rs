// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

pub mod attributes;
mod bootloader_serial;
mod errors;
pub mod probe_session;
pub mod tabs;

use attributes::app_attributes::AppAttributes;
use attributes::general_attributes::GeneralAttributes;
use attributes::system_attributes::SystemAttributes;

use probe_rs::probe::DebugProbeInfo;
use probe_rs::MemoryInterface;
use probe_session::ProbeSession;

use errors::TockloaderError;
use tabs::tab::Tab;
use tbf_parser::parse::parse_tbf_header_lengths;

pub async fn list_probe(
    choice: DebugProbeInfo,
    chip: &str,
    core_index: &usize,
) -> Vec<AppAttributes> {
    let mut probe_session = ProbeSession::new(choice, chip);
    let mut core = probe_session.get_core(*core_index);

    let system_attributes = SystemAttributes::read_system_attributes(&mut core);

    AppAttributes::read_apps_data(&mut core, system_attributes.appaddr.unwrap())
}

pub async fn info_probe(
    choice: DebugProbeInfo,
    chip: &str,
    core_index: &usize,
) -> GeneralAttributes {
    let mut probe_session = ProbeSession::new(choice, chip);

    let mut core = probe_session.get_core(*core_index);

    let system_attributes = SystemAttributes::read_system_attributes(&mut core);

    let apps_details = AppAttributes::read_apps_data(&mut core, system_attributes.appaddr.unwrap());

    GeneralAttributes::new(system_attributes, apps_details)
}

pub async fn install_app(
    choice: DebugProbeInfo,
    board: &String,
    chip: &str,
    core_index: &usize,
    tab_file: Tab,
) -> Result<(), TockloaderError> {
    let mut probe_session = ProbeSession::new(choice, chip);
    let mut core = probe_session.get_core(*core_index);

    // Verify if the specified app is compatible with board
    match tab_file.is_compatible_with_board(board) {
        Ok(value) => {
            if value {
                println!("Specified tab is compatible with board.");
            } else {
                println!("Specified tab is not compatible with board.");
            }
        }
        Err(e) => println!("Something went wrong: {:?}", e),
    }

    // Get board data
    let system_attributes = SystemAttributes::read_system_attributes(&mut core);
    let kernel_version = system_attributes.kernel_version.unwrap();

    // Verify if the specified app is compatible with kernel version
    match tab_file.is_compatible_with_kernel_verison(kernel_version as f32) {
        Ok(value) => {
            if value {
                println!("Specified tab is compatible with your kernel version.");
            } else {
                println!("Specified tab is not compatible with your kernel version.");
            }
        }
        Err(e) => println!("Something went wrong: {:?}", e),
    }

    // Get the address from which we start writing the new app
    let mut address: u64 = system_attributes.appaddr.unwrap();

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

    // Create app object
    let app = tab_file.extract_app(system_attributes.arch).unwrap();
    let size = app.get_size() as u64;

    // Make sure the app is aligned to a multiple of its size
    let multiple = address / size;
    let (new_address, gap_size) = if multiple * size != address {
        let new_address = ((address + size) / size) * size;
        let gap_size = new_address - address;
        (new_address, gap_size)
    } else {
        (address, 0)
    };

    // Make sure the binary is a multiple of the page size by padding 0xFFs
    // TODO(Micu Ana): check if the page-size differs
    let page_size = 512;
    let binary = app.get_app_binary();
    let binary_len = binary.len();
    let needs_padding = binary_len % page_size != 0;

    let mut app = app;
    if gap_size > 0 {
        app.set_padding(gap_size);
    }

    if needs_padding {
        let remaining = page_size - (binary_len % page_size);
        app.add_padding_to_app_binary(remaining);
    }

    // Get indices of pages that have valid data to write
    let valid_pages: Vec<u8> = app.get_valid_pages(binary_len, page_size);

    for i in valid_pages {
        // Create the packet that we send to the bootloader
        // First four bytes are the address of the page
        let mut pkt = (new_address + page_size as u64).to_le_bytes().to_vec();

        // Then the bytes that go into the page
        for b in binary[(i as usize * page_size)..((i + 1) as usize * page_size)].to_vec() {
            pkt.push(b);
        }

        // TODO(Micu Ana): Write to bootloader
    }

    Ok(())
}
