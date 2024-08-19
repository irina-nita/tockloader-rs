// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use tbf_parser::types::{TbfFooterV2Credentials, TbfHeader};

pub struct TabTbf {
    pub filename: String,
    pub tbfh: TbfHeader,
    pub app_binary: Vec<u8>,
    pub tbff: TbfFooterV2Credentials,
}

impl TabTbf {}
