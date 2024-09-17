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
    ProbeRsCommunicationError(probe_rs::Error),

    #[error("Failed to read from debug probe. Inner: {0}")]
    ProbeRsReadError(probe_rs::Error),

    #[error("Failed to write binary. Inner: {0}")]
    ProbeRsWriteError(#[from] probe_rs::flashing::FlashError),

    #[error("Failed to initialize serial connection due to a communication error. Inner: {0}")]
    SerialInitializationError(#[from] tokio_serial::Error),

    #[error("Bootloader did not respond properly: {0}")]
    BootloaderError(u8),

    #[error("No binary found for {0} architecture.")]
    NoBinaryError(String),

    #[error("Failed to perform read/write operations on serial port. Inner: {0}")]
    IOError(#[from] io::Error),

    #[error("Expected board attribute to be present")]
    MisconfiguredBoard(String),
}
