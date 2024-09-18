// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use crate::errors::TockloaderError;
use crate::tabs::metadata::Metadata;
use std::{fs::File, io::Read};
use tar::Archive;

struct TbfFile {
    pub filename: String,
    pub data: Vec<u8>,
}

pub struct Tab {
    metadata: Metadata,
    tbf_files: Vec<TbfFile>,
}

impl Tab {
    pub fn open(path: String) -> Result<Self, TockloaderError> {
        let mut metadata = None;
        let mut tbf_files = Vec::new();

        match File::open(path) {
            Ok(file) => {
                let mut archive = Archive::new(file);
                match archive.entries() {
                    Ok(entries) => {
                        for file in entries {

                            match file {
                                Ok(mut file) => {

                                    match file.path() {
                                        Ok(path) => {

                                            match path.file_name() {
                                                Some(file_name) => {
                                                    let file_name = file_name.to_str().unwrap_or("").to_owned();
                                                    if file_name == "metadata.toml" {
                                                        let mut buf = String::new();
                                                        match file.read_to_string(&mut buf) {
                                                            Ok(_) => metadata = Some(Metadata::new(buf).unwrap()),
                                                            Err(e) => return Err(TockloaderError::UnusableTab(e)),
                                                        };
                                                    } else if file_name.ends_with(".tbf") {
                                                        let mut data = Vec::new();

                                                        match file.read_to_end(&mut data) {
                                                            Ok(_) => {
                                                                tbf_files.push(TbfFile {
                                                                    filename: file_name.to_string(),
                                                                    data,
                                                                });
                                                            },
                                                            Err(e) => return Err(TockloaderError::UnusableTab(e)),
                                                        }
                                                    }
                                                },
                                                None => continue,
                                            }
                                        },
                                        Err(e) => return Err(TockloaderError::UnusableTab(e)),
                                    }
                                },
                                Err(e) => return Err(TockloaderError::UnusableTab(e)),
                            }
                        }
        
                        if metadata.is_none() {
                            panic!("No metadata.toml found in tab");
                        }
        
                        Ok(Tab {
                            metadata: metadata.unwrap(),
                            tbf_files,
                        })
                    },
                    Err(e) => return Err(TockloaderError::UnusableTab(e)),
                }
            }
            Err(e) => return Err(TockloaderError::UnusableTab(e)),
        }
    }

    pub fn is_compatible_with_kernel_verison(&self, _kernel_version: u32) -> bool {
        // Kernel version seems to not be working properly on the microbit bootloader. It is always
        // "1" despite the actual version.
        // return self.metadata.minimum_tock_kernel_version.major <= kernel_version;
        true
    }

    pub fn is_compatible_with_board(&self, board: &String) -> bool {
        if let Some(boards) = &self.metadata.only_for_boards {
            boards.contains(board)
        } else {
            true
        }
    }

    pub fn extract_binary(&self, arch: &str) -> Result<Vec<u8>, TockloaderError> {
        for file in &self.tbf_files {
            if file.filename.starts_with(arch) {
                return Ok(file.data.clone());
            }
        }

        Err(TockloaderError::NoBinaryError(arch.to_owned()))
    }
}
