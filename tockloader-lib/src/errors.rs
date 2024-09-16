// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TockloaderError {
    #[error("Error occurred while trying to access core: {0}")]
    CoreAccessError(usize, probe_rs::Error),

    #[error("Failed to initialize probe_rs connection due to a communication error. Inner: {0}")]
    ProbeRsInitializationError(#[from] probe_rs::probe::DebugProbeError),

    #[error("Failed to establish communication with board. Inner: {0}")]
    ProbeRsCommunicationError(#[from] probe_rs::Error),

    #[error("Failed to initialize serial connection due to a communication error. Inner: {0}")]
    SerialInitializationError(#[from] tokio_serial::Error),

    #[error("Expected board attribute to be present")]
    MisonfiguredBoard(String),

    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
}
