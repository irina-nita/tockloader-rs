// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use super::{app_attributes::AppAttributes, system_attributes::GeneralAttributes};

#[derive(Debug)]
pub struct Attributes {
    pub general: GeneralAttributes,
    pub apps: Vec<AppAttributes>,
}

impl Attributes {
    pub(crate) fn new(
        general_attributes: GeneralAttributes,
        apps_attributes: Vec<AppAttributes>,
    ) -> Attributes {
        Attributes {
            general: general_attributes,
            apps: apps_attributes,
        }
    }
}
