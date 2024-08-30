// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use tbf_parser::types::{TbfFooterV2Credentials, TbfHeader};

#[allow(dead_code)]
pub struct TabTbf {
    tbfh: TbfHeader,
    app_binary: Vec<u8>,
    tbff: TbfFooterV2Credentials,
    size: usize,
    padding: Option<u64>,
}

impl TabTbf {
    pub fn new(
        tbfh: TbfHeader,
        app_binary: Vec<u8>,
        tbff: TbfFooterV2Credentials,
        size: usize,
    ) -> Self {
        TabTbf {
            tbfh,
            app_binary,
            tbff,
            size,
            padding: None,
        }
    }

    pub fn get_size(&self) -> usize {
        return self.size;
    }

    pub fn get_app_binary(&self) -> Vec<u8> {
        return self.app_binary.clone();
    }

    pub fn set_padding(&mut self, padding: u64) {
        self.padding = Some(padding);
    }

    pub fn get_valid_pages(self, binary_len: usize, binary: Vec<u8>, page_size: usize) -> Vec<u8> {
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

        valid_pages
    }
}
