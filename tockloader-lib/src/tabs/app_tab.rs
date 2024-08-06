// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use super::{tbff::TBFFooter, tbfh::TBFHeaderV2Base};

pub struct TabTbf {
    _filename: String,
    _tbfh: TBFHeaderV2Base,
    _app_binary: Vec<u8>,
    _tbff: TBFFooter,
}

impl TabTbf {}
