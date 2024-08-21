// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

pub mod attributes;
mod errors;
pub mod probe_session;

use attributes::app_attributes::AppAttributes;
use attributes::get_app_attributes::get_apps_data;
use attributes::get_board_attributes::{get_board_attributes, get_bootloader_version};
use attributes::get_kernel_attributes::get_kernel_attributes;
use attributes::hardware_attributes::HardwareAttributes;

use probe_rs::probe::DebugProbeInfo;
use probe_session::ProbeSession;

pub async fn list_probe(
    choice: DebugProbeInfo,
    chip: &str,
    core_index: &usize,
) -> Vec<AppAttributes> {
    let mut probe_session = ProbeSession::new(choice, chip);
    let mut core = probe_session.get_core(*core_index);

    get_apps_data(&mut core)
}

pub async fn info_probe(
    choice: DebugProbeInfo,
    chip: &str,
    core_index: &usize,
) -> (HardwareAttributes, Vec<AppAttributes>) {
    let mut probe_session = ProbeSession::new(choice, chip);

    let mut core = probe_session.get_core(*core_index);

    let mut attributes: HardwareAttributes = HardwareAttributes::new();

    get_board_attributes(&mut core, &mut attributes);

    get_bootloader_version(&mut core, &mut attributes);

    get_kernel_attributes(&mut core, &mut attributes);

    let apps_details = get_apps_data(&mut core);

    (attributes, apps_details)
}
