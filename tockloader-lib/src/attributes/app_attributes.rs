// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

#[derive(Debug)]
pub struct AppAttributes {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub header_size: Option<u16>,
    pub total_size: Option<u32>,
    pub kernel_version: Option<(u16, u16)>,
    pub minumum_ram_size: Option<u32>,
    pub tbf_version: Option<u16>,
}

impl AppAttributes {
    pub(crate) fn new() -> AppAttributes {
        AppAttributes {
            name: None,
            enabled: None,
            header_size: None,
            total_size: None,
            kernel_version: None,
            minumum_ram_size: None,
            tbf_version: None,
        }
    }
}
