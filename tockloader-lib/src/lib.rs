// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

pub mod attributes;
pub mod connection;
pub mod bootloader_serial;
mod errors;
pub mod tabs;

use std::time::Duration;

use attributes::app_attributes::AppAttributes;
use attributes::decode::decode_attribute;
use attributes::general_attributes::GeneralAttributes;
use attributes::system_attributes::SystemAttributes;

use bootloader_serial::{issue_command, ping_bootloader_and_wait_for_response, Command, Response};
use byteorder::{ByteOrder, LittleEndian};
use connection::Connection;
use probe_rs::flashing::DownloadOptions;
use probe_rs::probe::DebugProbeInfo;
use probe_rs::MemoryInterface;

use errors::TockloaderError;
use tabs::tab::Tab;
use tbf_parser::parse::parse_tbf_header_lengths;
use tokio_serial::SerialPortInfo;

pub fn list_debug_probes() -> Vec<DebugProbeInfo> {
    probe_rs::probe::list::Lister::new().list_all()
}

pub fn list_serial_ports() -> Vec<SerialPortInfo> {
    //TODO(Micu Ana): Add error handling
    tokio_serial::available_ports().unwrap()
}

pub async fn list_probe(
    choice: Connection,
    core_index: &usize,
) -> Result<Vec<AppAttributes>, TockloaderError> {
    match choice {
        Connection::ProbeRS(mut session) => {
            let mut core = session.core(*core_index).unwrap();
            let system_attributes = SystemAttributes::read_system_attributes(&mut core);
            Ok(AppAttributes::read_apps_data(
                &mut core,
                system_attributes.appaddr.unwrap(),
            ))
        }
        _ => {
            // TODO(Micu Ana): Add error handling
            Err(TockloaderError::NoPortAvailable)
        }
    }
}

pub async fn list_serial(_choice: Connection) -> Result<Vec<AppAttributes>, TockloaderError> {
    todo!();
}

pub async fn info_probe(
    choice: Connection,
    core_index: &usize,
) -> Result<GeneralAttributes, TockloaderError> {
    match choice {
        Connection::ProbeRS(mut session) => {
            let mut core = session.core(*core_index).unwrap();
            let system_attributes = SystemAttributes::read_system_attributes(&mut core);
            let apps_details =
                AppAttributes::read_apps_data(&mut core, system_attributes.appaddr.unwrap());
            Ok(GeneralAttributes::new(system_attributes, apps_details))
        }
        _ => {
            // TODO(Micu Ana): Add error handling
            Err(TockloaderError::NoPortAvailable)
        }
    }
}

pub async fn info_serial(_choice: Connection) -> Result<GeneralAttributes, TockloaderError> {
    todo!();
}

pub async fn install_app_probe_rs(
    choice: Connection,
    board: &String,
    core_index: &usize,
    tab_file: Tab,
) -> Result<(), TockloaderError> {
    match choice {
        Connection::ProbeRS(mut session) => {
            // Verify if the specified app is compatible with board
            // TODO(Micu Ana): Replace the prints with log messages
            if tab_file.is_compatible_with_board(board) {
                println!("Specified tab is compatible with board.");
            } else {
                panic!("Specified tab is not compatible with board.");
            }

            // Get core - if not specified, by default is 0
            // TODO (Micu Ana): Add error handling
            let mut core = session.core(*core_index).unwrap();

            // Get board data
            let system_attributes = SystemAttributes::read_system_attributes(&mut core);
            let kernel_version = system_attributes.kernel_version.unwrap();
            println!("Kernel version of board: {}", kernel_version);

            // Verify if the specified app is compatible with kernel version
            // TODO(Micu Ana): Replace the prints with log messages
            if tab_file.is_compatible_with_kernel_verison(kernel_version as u32) {
                println!("Specified tab is compatible with your kernel version.");
            } else {
                println!("Specified tab is not compatible with your kernel version.");
            }

            // Get the address from which we start writing the new app
            // TODO: change appaddr to 32 bit
            // TODO for the future: support 64 bit arhitecture
            let mut address: u64 = system_attributes.appaddr.unwrap();

            // Loop to check if there are another apps installed
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
                        Ok((ver, header_len, whole_len)) if header_len != 0 => {
                            (ver, header_len, whole_len)
                        }
                        _ => break, // No more apps
                    };

                address += whole_len as u64;
            }

            let mut binary = tab_file
                .extract_binary(&system_attributes.arch.unwrap()) // use the system_attributes arch or the provided one?
                .unwrap();

            let size = binary.len() as u64;

            // Make sure the app is aligned to a multiple of its size
            let multiple = address / size;

            let (new_address, _gap_size) = if multiple * size != address {
                let new_address = ((address + size) / size) * size;
                let gap_size = new_address - address;
                (new_address, gap_size)
            } else {
                (address, 0)
            };

            // No more need of core
            drop(core);

            // Make sure the binary is a multiple of the page size by padding 0xFFs
            // TODO(Micu Ana): check if the page-size differs
            let page_size = 512;
            let needs_padding = binary.len() % page_size != 0;

            if needs_padding {
                let remaining = page_size - (binary.len() % page_size);
                dbg!(remaining);
                for _i in 0..remaining {
                    binary.push(0xFF);
                }
            }

            // Get indices of pages that have valid data to write
            let mut valid_pages: Vec<u8> = Vec::new();
            for i in 0..(size as usize / page_size) {
                for b in binary[(i * page_size)..((i + 1) * page_size)]
                    .iter()
                    .copied()
                {
                    if b != 0 {
                        valid_pages.push(i.try_into().unwrap());
                        break;
                    }
                }
            }

            // If there are no pages valid, all pages would have been removed, so we write them all
            if valid_pages.is_empty() {
                for i in 0..(size as usize / page_size) {
                    valid_pages.push(i.try_into().unwrap());
                }
            }

            // Include a blank page (if exists) after the end of a valid page. There might be a usable 0 on the next page
            let mut ending_pages: Vec<u8> = Vec::new();
            for &i in &valid_pages {
                let mut iter = valid_pages.iter();
                if !iter.any(|&x| x == (i + 1)) && (i + 1) < (size as usize / page_size) as u8 {
                    ending_pages.push(i + 1);
                }
            }

            for i in ending_pages {
                valid_pages.push(i);
            }

            for i in valid_pages {
                println!("Writing page number {}", i);
                // Create the packet that we send to the bootloader
                // First four bytes are the address of the page
                let mut pkt = Vec::new();

                // Then the bytes that go into the page
                for b in binary[(i as usize * page_size)..((i + 1) as usize * page_size)]
                    .iter()
                    .copied()
                {
                    pkt.push(b);
                }
                let mut loader = session.target().flash_loader();

                loader
                    .add_data(
                        (new_address as u32 + (i as usize * page_size) as u32).into(),
                        &pkt,
                    )
                    .unwrap();

                let mut options = DownloadOptions::default();
                options.keep_unwritten_bytes = true;

                // Finally, the data can be programmed
                loader.commit(&mut session, options).unwrap();
            }
        }

        _ => {
            // TODO(Micu Ana): Add error handling
            return Err(TockloaderError::NoPortAvailable);
        }
    }
    Ok(())
}

pub async fn install_app_serial(
    choice: Connection,
    board: &String,
    tab_file: Tab,
) -> Result<(), TockloaderError> {
    match choice {
        Connection::Serial(mut port) => {
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

            let response = ping_bootloader_and_wait_for_response(&mut port).await?;

            dbg!(response.clone());

            if response as u8 != Response::ResponsePong as u8 {
                tokio::time::sleep(Duration::from_millis(100)).await;
                let response = ping_bootloader_and_wait_for_response(&mut port).await?;
                dbg!(response.clone());
            }

            let mut pkt = (0x600 as u32).to_le_bytes().to_vec();
            let length = (1024 as u16).to_le_bytes().to_vec();
            for i in length {
                pkt.push(i);
            }

            let (_, message) = issue_command(
                    &mut port,
                    Command::CommandReadRange,
                    pkt,
                    true,
                    64 * 16,
                    Response::ResponseReadRange,
                )
                .await
                .unwrap();

            let mut data = message.chunks(64);

            let mut address = 0 as u64;
            let mut arch = String::new();    

            for index_data in 0..data.len() {
                let step = match data.next() {
                    Some(data) => data,
                    None => break,
                };

                let step_option = decode_attribute(step);

                if step_option.is_none() {
                    continue;
                }

                let decoded_attributes = step_option.unwrap();

                match index_data {
                    0 => {}
                    1 => {
                        arch = decoded_attributes.value.to_string();
                    }
                    2 => {
                        dbg!(&decoded_attributes);
                        address = u64::from_str_radix(&decoded_attributes.value[2..], 16).unwrap();
                    }
                    3 => {}
                    _ => panic!("Board data not found!"),
                }
            }

            let mut pkt = ((address - 100) as u32).to_le_bytes().to_vec();
            let length = (100 as u16).to_le_bytes().to_vec();
            for i in length {
                pkt.push(i);
            }

            let (_, message) = issue_command(
                    &mut port,
                    Command::CommandReadRange,
                    pkt,
                    true,
                    100,
                    Response::ResponseReadRange,
                )
                .await
                .unwrap();

            let kernel_version = LittleEndian::read_uint(&message[95..96], 1);

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

            loop {
                // Read a block of 200 8-bit words
                let mut pkt = (address as u32).to_le_bytes().to_vec();
                let length = (200 as u16).to_le_bytes().to_vec();
                for i in length {
                    pkt.push(i);
                }

                let (_, message) = issue_command(
                        &mut port,
                        Command::CommandReadRange,
                        pkt,
                        true,
                        200,
                        Response::ResponseReadRange,
                    )
                    .await
                    .unwrap();

                let (_ver, _header_len, whole_len) =
                    match parse_tbf_header_lengths(&message[0..8].try_into().unwrap()) {
                        Ok((ver, header_len, whole_len)) if header_len != 0 => (ver, header_len, whole_len),
                        _ => break, // No more apps
                    };
        
                address += whole_len as u64;
            }

            let mut binary = tab_file
                .extract_binary(Some(arch.clone()))
                .unwrap();


            let size = binary.len() as u64;

            let multiple = address / size;

            let (mut new_address, _gap_size) = if multiple * size != address {
                let new_address = ((address + size) / size) * size;
                let gap_size = new_address - address;
                (new_address, gap_size)
            } else {
                (address, 0)
            };

            // Make sure the binary is a multiple of the page size by padding 0xFFs
            // TODO(Micu Ana): check if the page-size differs
            let page_size = 512;
            let needs_padding = binary.len() % page_size != 0;

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

                // Write to bootloader
                let (_, _) = issue_command(
                        &mut port,
                        Command::CommandWritePage,
                        pkt,
                        true,
                        0,
                        Response::ResponseOK,
                    )
                    .await
                    .unwrap();
            }

            new_address += binary.len() as u64;

            let pkt = (new_address as u32).to_le_bytes().to_vec();
            dbg!(pkt.clone());

            let (_, _) = issue_command(
                    &mut port,
                    Command::CommandErasePage,
                    pkt,
                    true,
                    0,
                    Response::ResponseOK,
                )
                .await
                .unwrap();

        }
        _ => {
            // TODO(Micu Ana): Add error handling
            return Err(TockloaderError::NoPortAvailable);
        }
    }
    Ok(())
}
