// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

#[derive(Debug)]
pub struct AppAttributes {
    pub tbf_version: Option<u16>,
    pub header_size: Option<u16>,
    pub total_size: Option<u32>,
    // pub checksum: Option<>,
    pub flag_enabled: Option<bool>,
    // pub flag_sticky: Option<>,
    //      TLV: Main (1)
    // pub init_fn_offset: Option<>,
    // pub protected_size: Option<>,
    pub minumum_ram_size: Option<u32>,
    //      TLV: Program (9)
    //// pub init_fn_offset: Option<>,
    //// pub protected_size: Option<>,
    ////pub minumum_ram_size: Option<u32>,
    // pub binary_end_offset: Option<>,
    // pub app_version: Option<>,
    //      TLV: Package Name (3)
    pub name: Option<String>,
    //      TLV: Kernel Version (8)
    // pub kernel_major: Option<>
    // pub kernel_minor: Option<>
    pub kernel_version: Option<(u16, u16)>,
    //      Footer
    // pub footer_size: Option<>
    //      Footer TLV: Credentials (128)
    //      Type: Reserved (0)
    //      Length: 7008
}

impl AppAttributes {
    pub(crate) fn new() -> AppAttributes {
        AppAttributes {
            name: None,
            flag_enabled: None,
            header_size: None,
            total_size: None,
            kernel_version: None,
            minumum_ram_size: None,
            tbf_version: None,
        }
    }
}
