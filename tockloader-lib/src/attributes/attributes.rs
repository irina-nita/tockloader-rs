// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use super::{app_attributes::AppAttributes, general_attributes::GeneralAttributes};


#[derive(Debug)]
pub struct Attributes {
    pub general: Option<GeneralAttributes>,
    pub apps: Option<Vec<AppAttributes>>,
}

impl Attributes {
    pub(crate) fn new(general_attributes: GeneralAttributes, apps_attributes: Vec<AppAttributes>) -> Attributes {
        Attributes {
            general: Some(general_attributes),
            apps: Some(apps_attributes),
        }
    }
}


