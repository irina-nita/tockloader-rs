// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

pub mod attributes;
mod errors;
pub mod probe_session;

use attributes::app_attributes::get_apps_data;
use attributes::app_attributes::AppAttributes;
use attributes::general_attributes::Attributes;
use attributes::system_attributes::GeneralAttributes;

use probe_rs::probe::DebugProbeInfo;
use probe_session::ProbeSession;

pub async fn list_probe(
    choice: DebugProbeInfo,
    chip: &str,
    core_index: &usize,
) -> Vec<AppAttributes> {
    let mut probe_session = ProbeSession::new(choice, chip);
    let mut core = probe_session.get_core(*core_index);

    let mut general_attributes = GeneralAttributes::new();
    general_attributes.get_general_attributes(&mut core);

    get_apps_data(&mut core, general_attributes.appaddr.unwrap())
}

pub async fn info_probe(choice: DebugProbeInfo, chip: &str, core_index: &usize) -> Attributes {
    let mut probe_session = ProbeSession::new(choice, chip);

    let mut core = probe_session.get_core(*core_index);

    let mut general_attributes = GeneralAttributes::new();
    general_attributes.get_general_attributes(&mut core);

    let apps_details = get_apps_data(&mut core, general_attributes.appaddr.unwrap());

    Attributes::new(general_attributes, apps_details)
}
