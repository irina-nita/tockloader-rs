// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use inquire::{InquireError, Select};
use probe_rs::{
    integration::FakeProbe,
    probe::{ftdi, list::Lister, DebugProbeInfo, Probe},
};

pub fn select_probe() -> Result<DebugProbeInfo, InquireError> {
    // let lister = Lister::new();
    // let probes = lister.list_all();
    // let ans = Select::new("Which probe do you want to use?", probes).prompt();
    // ans
    Ok(DebugProbeInfo::new(
        "Mock probe",
        0x12,
        0x23,
        Some("mock_serial".to_owned()),
        &ftdi::FtdiProbeFactory,
        None,
    ))
}
