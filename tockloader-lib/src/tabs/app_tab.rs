// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use tbf_parser::types::{TbfFooterV2Credentials, TbfHeader};

#[allow(dead_code)]
pub struct TabTbf {
    filename: String,
    tbfh: TbfHeader,
    app_binary: Vec<u8>,
    tbff: TbfFooterV2Credentials,
    size: usize,
}

impl TabTbf {
    pub fn new(
        filename: String,
        tbfh: TbfHeader,
        app_binary: Vec<u8>,
        tbff: TbfFooterV2Credentials,
        size: usize,
    ) -> Self {
        TabTbf {
            filename,
            tbfh,
            app_binary,
            tbff,
            size,
        }
    }

    pub fn get_size(self) -> usize {
        return self.size;
    }
}
