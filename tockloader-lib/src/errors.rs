// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use std::io;
use thiserror::Error;

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
    ProbeRS(#[from] probe_rs::Error),

    #[error("Serial error: {0}")]
    Serial(#[from] tokio_serial::Error),
}
