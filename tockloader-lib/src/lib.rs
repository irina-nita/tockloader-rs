// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

pub mod attributes;
mod bootloader_serial;
mod errors;
pub mod probe_session;
pub mod tabs;

use std::time::Duration;

use attributes::app_attributes::AppAttributes;
use attributes::general_attributes::GeneralAttributes;
use attributes::system_attributes::SystemAttributes;

use probe_rs::flashing::DownloadOptions;
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
    let app = tab_file
        .extract_app(system_attributes.arch.clone())
        .unwrap();
    let header_binary = tab_file.extract_header_binary(system_attributes.arch.clone());
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

    // full_binary.append(&mut vec![0x60, 0x20, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0xd4, 0x00, 0x00, 0x00, 0x34, 0x21, 0x00, 0x00, 0xd4, 0x08, 0x00, 0x00, 0x68, 0x00, 0x00, 0x00, 0x3c, 0x09, 0x00, 0x00, 0x74, 0x01, 0x00, 0x00, 0x9c, 0x21, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x44, 0x6a, 0x04, 0xf1, 0x07, 0x04, 0x0c, 0x44, 0x07, 0x25, 0x24, 0xea, 0x05, 0x04, 0x85, 0x69, 0xc6, 0x69, 0x35, 0x44, 0x0d, 0x44, 0x06, 0x00, 0x0f, 0x00, 0x9d, 0x42, 0x00, 0xdc, 0xa5, 0x46]);
    for i in binary {
        full_binary.push(i);
    }
    for i in footer_binary {
        full_binary.push(i);
    }

    binary = full_binary;

    let size = binary.len() as u64;

    dbg!(size);

    //This should work but it doesn't

    // Make sure the app is aligned to a multiple of its size
    let multiple = address / size;

    let (new_address, gap_size) = if multiple * size != address {
        let new_address = ((address + size) / size) * size;
        let gap_size = new_address - address;
        (new_address, gap_size)
    } else {
        (address, 0)
    };

    // binary.insert(0, 2);
    // binary.insert(1, 0);
    // binary.insert(2, 16);
    // binary.insert(3, 0);
    // binary.insert(4, ((gap_size as u32) << 8) as u8);
    // binary.insert(5, ((gap_size as u32) >> 8) as u8);

    // for i in 9..(gap_size - 64) {
    //     binary.insert(i.try_into().unwrap(), 0);
    // }

    // for _i in 0..8 {
    //     binary.push(0xFF);
    // }

    dbg!(new_address);

    //dbg!(binary.clone());

    // No more need of core
    drop(core);

    // Make sure the binary is a multiple of the page size by padding 0xFFs
    // TODO(Micu Ana): check if the page-size differs
    let page_size = 512;
    let needs_padding = binary.len() % page_size != 0;

    dbg!(needs_padding);

    if needs_padding {
        let remaining = page_size - (binary.len() % page_size);
        dbg!(remaining);
        for _i in 0..remaining {
            binary.push(0xFF);
        }
    }

    let binary_len = binary.len();

    // Get indices of pages that have valid data to write
    let mut valid_pages: Vec<u8> = Vec::new();
    for i in 0..(binary_len / page_size) {
        for b in binary[(i * page_size)..((i + 1) * page_size)].to_vec() {
            if b != 0 {
                valid_pages.push(i.try_into().unwrap());
                break;
            }
        }
    }

    // If there are no pages valid, all pages would have been removed, so we write them all
    if valid_pages.len() == 0 {
        for i in 0..(binary_len / page_size) {
            valid_pages.push(i.try_into().unwrap());
        }
    }

    // Include a blank page (if exists) after the end of a valid page. There might be a usable 0 on the next page
    let mut ending_pages: Vec<u8> = Vec::new();
    for &i in &valid_pages {
        let mut iter = valid_pages.iter();
        if iter.find(|&&x| x == (i + 1)).is_none() && (i + 1) < (binary_len / page_size) as u8 {
            ending_pages.push(i + 1);
        }
    }

    for i in ending_pages {
        valid_pages.push(i);
    }
    dbg!(valid_pages.clone());

    //let valid_pages : Vec<u8> = (0..(binary_len / page_size) as u8).collect();

    for i in valid_pages {
        println!("Writing page number {}", i);
        // Create the packet that we send to the bootloader
        // First four bytes are the address of the page
        let mut pkt = Vec::new();
                 
        // Then the bytes that go into the page
        for b in binary[(i as usize * page_size)..((i + 1) as usize * page_size)].to_vec() {
            pkt.push(b);
        }
        let mut loader = probe_session.session.as_mut().unwrap().target().flash_loader();

        loader.add_data(((new_address as u32 + (i as usize * page_size) as u32)).into(), &pkt).unwrap();

        let mut options = DownloadOptions::default();
        options.keep_unwritten_bytes = true;
        // Finally, the data can be programmed:
        loader.commit(&mut probe_session.session.as_mut().unwrap(), options).unwrap();
    }

    // if let Some(port) = probe_session.port.as_mut() {
    //     let response = port.ping_bootloader_and_wait_for_response().await?;

    //     dbg!(response.clone());

    //     if response as u8 != Response::ResponsePong as u8 {
    //         tokio::time::sleep(Duration::from_millis(100)).await;
    //         let response = port.ping_bootloader_and_wait_for_response().await?;
    //         dbg!(response.clone());
    //     }

    //     for i in valid_pages {
    //         println!("Writing page number {}", i);
    //         // Create the packet that we send to the bootloader
    //         // First four bytes are the address of the page
    //         let mut pkt = (new_address as u32 + (i as usize * page_size) as u32)
    //             .to_le_bytes()
    //             .to_vec();
    //         dbg!(new_address as u32 + (i as usize * page_size) as u32);
    //         dbg!(pkt.clone());
    //         // Then the bytes that go into the page
    //         for b in binary[(i as usize * page_size)..((i + 1) as usize * page_size)].to_vec() {
    //             pkt.push(b);
    //         }
    //         // dbg!(pkt.clone());

    //         // Write to bootloader
    //         let response = port
    //             .issue_command(
    //                 Command::CommandWritePage,
    //                 pkt,
    //                 true,
    //                 0,
    //                 Response::ResponseOK,
    //             )
    //             .await
    //             .unwrap();
    //         dbg!(response);
    //     }

    //     new_address += binary.len() as u64;

    //     let pkt = (new_address as u32).to_le_bytes().to_vec();
    //     dbg!(pkt.clone());

    //     let response = port
    //         .issue_command(
    //             Command::CommandErasePage,
    //             pkt,
    //             true,
    //             0,
    //             Response::ResponseOK,
    //         )
    //         .await
    //         .unwrap();
    //     dbg!(response);
    // } else {
    //     // TODO(Micu Ana): Add error handling: Handle the case where `port` is None
    // }

    Ok(())
}
