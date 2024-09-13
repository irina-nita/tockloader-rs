// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use probe_rs::Error as ProbeRSError;
use std::io;
use thiserror::Error;
use tokio_serial::Error as SerialError;

#[derive(Debug, Error)]
pub enum TockloaderError {
    #[error("Error occurred while trying to establish connection: {0}")]
    Connection(ForeignError),

    #[error("IO error: {0}")]
    IOError(#[from] io::Error),

    #[error("Task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

#[derive(Debug, Error)]
pub enum ForeignError {
    #[error("ProbeRS error: {0}")]
    ProbeRS(#[from] ProbeRSError),

    #[error("Serial error: {0}")]
    Serial(#[from] SerialError),
}
