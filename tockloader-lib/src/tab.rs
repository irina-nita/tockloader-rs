// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

pub struct TabFile {
    pub path: String,
}

impl TabFile {
    pub fn new(
        path: String
    ) -> Self {
        TabFile {
            path
        }
    }
}