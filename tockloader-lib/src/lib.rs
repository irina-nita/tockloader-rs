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

use crate::bootloader_serial::Command;
use bootloader_serial::Response;

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
    // TODO: change appaddr to 32 bit
    // TODO for the future: support 64 bit arhitecture
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

    dbg!(address);
    // Create app object
    let app = tab_file.extract_app(system_attributes.arch.clone()).unwrap();
    let header_binary = tab_file.extract_header_binary(system_attributes.arch.clone());
    dbg!(header_binary.clone());
    dbg!(header_binary.len());
    let footer_binary = tab_file.extract_footer_binary(system_attributes.arch.clone());
    dbg!(footer_binary.len());
    let mut binary = app.get_app_binary();
    dbg!(binary.len());

    // Concatenate all binaries (header + app + footer)
    let mut full_binary = Vec::new();
    for i in header_binary {
        full_binary.push(i);
    }
    for i in binary {
        full_binary.push(i);
    } 
    for i in footer_binary {
        full_binary.push(i);
    } 

    binary = full_binary;

    let size = binary.len() as u64;

    dbg!(size);

    // Make sure the app is aligned to a multiple of its size
    // let multiple = address / size;
    // let mut app = app;

    /* let (mut new_address, gap_size) = if multiple * size != address {
        let new_address = ((address + size) / size) * size;
        let gap_size = new_address - address;
        (new_address, gap_size)
    } else {
        (address, 0)
    };*/

    let mut new_address = address;

    /*if gap_size > 0 {
        app.set_padding(gap_size);
    }*/

    dbg!(new_address);
    // No more need of core
    drop(core);


    // Make sure the binary is a multiple of the page size by padding 0xFFs
    // TODO(Micu Ana): check if the page-size differs
    let page_size = 512;
    let binary_len = binary.len();
    dbg!(binary_len);
    let needs_padding = binary_len % page_size != 0;

    dbg!(needs_padding);

    if needs_padding {
        let remaining = page_size - (binary_len % page_size);
        dbg!(remaining);
        for _i in 0..remaining {
            binary.push(0xFF);
        }
    }

    // Get indices of pages that have valid data to write
    let valid_pages: Vec<u8> = app.get_valid_pages(binary_len, binary.clone(), page_size);
    dbg!(valid_pages.clone());

    if let Some(port) = probe_session.port.as_mut() {
        for i in valid_pages {
            println!("Writing page number {}", i);
            // Create the packet that we send to the bootloader
            // First four bytes are the address of the page
            let mut pkt = (new_address as u32 + (i as usize * page_size) as u32)
                .to_le_bytes()
                .to_vec();
            dbg!(new_address as u32 + (i as usize * page_size) as u32);
            dbg!(pkt.clone());
            // Then the bytes that go into the page
            for b in binary[(i as usize * page_size)..((i + 1) as usize * page_size)].to_vec() {
                pkt.push(b);
            }
            // dbg!(pkt.clone());

            // Write to bootloader
            let response = port
                .issue_command(
                    Command::CommandWritePage,
                    pkt,
                    true,
                    0,
                    Response::ResponseOK,
                )
                .await
                .unwrap();
            dbg!(response);
        }

        new_address += binary.len() as u64;

        let pkt = (new_address as u32)
                .to_le_bytes()
                .to_vec();
        dbg!(pkt.clone());

        let response = port
                .issue_command(
                    Command::CommandErasePage,
                    pkt,
                    true,
                    0,
                    Response::ResponseOK,
                )
                .await
                .unwrap();
        dbg!(response);

    } else {
        // TODO(Micu Ana): Add error handling: Handle the case where `port` is None
    }



    Ok(())
}
