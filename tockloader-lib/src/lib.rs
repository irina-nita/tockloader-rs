// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

pub mod attributes;
mod board_settings;
mod bootloader_serial;
mod errors;
pub mod probe_session;
pub mod tabs;

use attributes::app_attributes::AppAttributes;
use attributes::get_app_attributes::{get_apps_data, get_start_address};
use attributes::get_board_attributes::{get_appaddr, get_board_attributes, get_bootloader_version};
use attributes::get_kernel_attributes::{get_kernel_attributes, get_kernel_version};
use attributes::hardware_attributes::HardwareAttributes;

use tabs::tab::Tab;

use errors::TockloaderError;
use probe_rs::probe::DebugProbeInfo;
use probe_session::ProbeSession;

pub async fn list_probe(
    choice: DebugProbeInfo,
    board: &str,
    chip: &str,
    core_index: &usize,
) -> Vec<AppAttributes> {
    let mut probe_session = ProbeSession::new(choice, board, chip);
    let mut core = probe_session.get_core(*core_index);

    get_apps_data(&mut core)
}

pub async fn info_probe(
    choice: DebugProbeInfo,
    board: &str,
    chip: &str,
    core_index: &usize,
) -> (HardwareAttributes, Vec<AppAttributes>) {
    let mut probe_session = ProbeSession::new(choice, board, chip);

    let mut core = probe_session.get_core(*core_index);

    let mut attributes: HardwareAttributes = HardwareAttributes::new();

    get_board_attributes(&mut core, &mut attributes);

    get_bootloader_version(&mut core, &mut attributes);

    get_kernel_attributes(&mut core, &mut attributes);

    let apps_details = get_apps_data(&mut core);

    (attributes, apps_details)
}

pub async fn install_app(
    choice: DebugProbeInfo,
    board: &String,
    chip: &str,
    core_index: &usize,
    tab_file: Tab,
) -> Result<(), TockloaderError> {
    // Open port and configure it
    let mut probe_session = ProbeSession::new(choice, board, chip);
    let mut core = probe_session.get_core(*core_index);
    let address: u64 = get_appaddr(&mut core).expect("Could not find app address.");

    // Jump through the linked list of apps to check the address to install the app
    let start_address = get_start_address(&mut core, address).unwrap();

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

    // Verify if the specified app is compatible with kernel version
    match tab_file.is_compatible_with_kernel_verison(get_kernel_version(&mut core) as f32) {
        Ok(value) => {
            if value {
                println!("Specified tab is compatible with your kernel version.");
            } else {
                println!("Specified tab is not compatible with your kernel version.");
            }
        }
        Err(e) => println!("Something went wrong: {:?}", e),
    }

    // Get board data
    let mut attr = HardwareAttributes::new();
    get_board_attributes(&mut core, &mut attr);

    // Create app object
    let app = tab_file.extract_app(attr.arch).unwrap();
    let size = app.get_size() as u64;

    // Make sure the app is aligned to a multiple of its size
    let multiple = start_address / size;
    let (new_address, gap_size) = if multiple * size != start_address {
        let new_address = ((start_address + size) / size) * size;
        let gap_size = new_address - start_address;
        (new_address, gap_size)
    } else {
        (start_address, 0)
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
