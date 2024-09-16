// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use probe_rs::{probe::DebugProbeInfo, Permissions, Session};
use std::time::Duration;
use tokio_serial::{FlowControl, Parity, SerialPort, SerialStream, StopBits};

use crate::errors::{ForeignError, TockloaderError};

pub enum ConnectionInfo {
    SerialInfo(String),
    ProbeInfo(DebugProbeInfo),
}

impl From<String> for ConnectionInfo {
    fn from(value: String) -> Self {
        ConnectionInfo::SerialInfo(value)
    }
}

pub enum Connection {
    ProbeRS(Session),
    Serial(SerialStream),
}

impl Connection {
    pub fn open(info: ConnectionInfo, chip: Option<String>) -> Result<Connection, TockloaderError> {
        match info {
            ConnectionInfo::SerialInfo(serial_info) => {
                let builder = tokio_serial::new(serial_info, 115200);
                match SerialStream::open(&builder) {
                    Ok(mut port) => {
                        port.set_parity(Parity::None)
                            .map_err(|e| TockloaderError::Connection(ForeignError::Serial(e)))?;
                        port.set_stop_bits(StopBits::One)
                            .map_err(|e| TockloaderError::Connection(ForeignError::Serial(e)))?;
                        port.set_flow_control(FlowControl::None)
                            .map_err(|e| TockloaderError::Connection(ForeignError::Serial(e)))?;
                        port.set_timeout(Duration::from_millis(500))
                            .map_err(|e| TockloaderError::Connection(ForeignError::Serial(e)))?;
                        port.write_request_to_send(false)
                            .map_err(|e| TockloaderError::Connection(ForeignError::Serial(e)))?;
                        port.write_data_terminal_ready(false)
                            .map_err(|e| TockloaderError::Connection(ForeignError::Serial(e)))?;
                        Ok(Connection::Serial(port))
                    }
                    Err(e) => Err(TockloaderError::Connection(ForeignError::Serial(e))),
                }
            }
            ConnectionInfo::ProbeInfo(probe_info) => {
                let probe = probe_info.open().map_err(|e| {
                    TockloaderError::Connection(ForeignError::ProbeRS(probe_rs::Error::Probe(e)))
                })?;
                let session = probe
                    .attach(chip.unwrap(), Permissions::default())
                    .map_err(|e| TockloaderError::Connection(ForeignError::ProbeRS(e)))?;
                Ok(Connection::ProbeRS(session))
            }
        }
    }
}
