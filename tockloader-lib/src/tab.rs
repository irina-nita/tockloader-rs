// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use std::{fs::File, io::Read};

use tar::Archive;
use tbf_parser::{
    parse::{parse_tbf_header, parse_tbf_header_lengths},
    types::TbfParseError,
};

use crate::{attributes::app_attributes::AppAttributes, errors::TockloaderError};

pub struct TabFile {
    pub path: String,
}

impl TabFile {
    pub fn new(path: String) -> Self {
        TabFile { path }
    }

    // TODO(MicuAna): add error handling
    pub fn is_compatible_with_kernel_verison(
        &self,
        kernel_version: f32,
    ) -> Result<bool, TockloaderError> {
        let mut value = false;
        let mut archive = Archive::new(File::open(self.path.clone()).unwrap());
        for entry in archive.entries().unwrap() {
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
                                    if item.contains("minimum-tock-kernel-version") {
                                        let columns = item.split("=");
                                        let elem = columns.collect::<Vec<&str>>();
                                        let kernver = elem[1].trim().parse::<f32>().unwrap();
                                        if kernver == kernel_version {
                                            value = true;
                                            break;
                                        }
                                    }
                                }
                            }
                            break;
                        } else {
                            println!("Failed to get path");
                        }
                    }
                }
                Err(e) => {
                    println!("Can't open entry in tab: {:?}", e);
                }
            }
        }
        Ok(value)
    }

    // TODO(MicuAna): add error handling
    pub fn is_compatible_with_board(&self, board: &String) -> Result<bool, TockloaderError> {
        let mut value = false;
        let mut archive = Archive::new(File::open(self.path.clone()).unwrap());
        for entry in archive.entries().unwrap() {
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
                                                value = true;
                                                break;
                                            }
                                        }
                                    } else {
                                        value = true;
                                    }
                                }
                                break;
                            } else {
                                println!("Failed to get path");
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Can't open entry in tab: {:?}", e);
                }
            }
        }
        Ok(value)
    }

    pub fn extract_app(&self, arch: Option<String>) -> Option<TabFile> {
        // Find all filenames that start with the architecture name
        let mut archive = Archive::new(File::open(self.path.clone()).unwrap());
        let mut tabtbf: AppAttributes = AppAttributes::new();
        for entry in archive.entries().unwrap() {
            match entry {
                Ok(mut entry) => {
                    if let Ok(path) = entry.path() {
                        if let Some(file_name) = path.file_name() {
                            let name = file_name.to_str().unwrap();
                            let name_pieces = name.split(".");
                            let name_vec = name_pieces.collect::<Vec<&str>>();
                            if name_vec.len() >= 2 && name_vec[name_vec.len() - 1] == "tbf" {
                                if name_vec[0] == arch.clone().unwrap() {
                                    let mut data = Vec::new();
                                    entry.read_to_end(&mut data).unwrap();
                                    let (ver, header_size, _total_size) =
                                        match parse_tbf_header_lengths(
                                            &data[0..8].try_into().unwrap(),
                                        ) {
                                            Ok((ver, header_size, total_size))
                                                if header_size != 0 =>
                                            {
                                                tabtbf.tbf_version = Some(ver);
                                                tabtbf.header_size = Some(header_size);
                                                tabtbf.total_size = Some(total_size);
                                                (ver, header_size, total_size)
                                            }
                                            _ => break,
                                        };
                                    let header =
                                        parse_tbf_header(&data[0..header_size as usize], ver);
                                    match header {
                                        Ok(header) => {
                                            tabtbf.flag_enabled = Some(header.enabled());
                                            tabtbf.minumum_ram_size =
                                                Some(header.get_minimum_app_ram_size());

                                            tabtbf.kernel_version = Some(
                                                header
                                                    .get_kernel_version()
                                                    .expect("Could not get kernel version."),
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
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Can't open entry in tab: {:?}", e);
                }
            }
        }
        None
    }
}
