// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use std::{fs::File, io::Read};

use tar::Archive;

use crate::errors::TockloaderError;

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
                                    }
                                }
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
}
