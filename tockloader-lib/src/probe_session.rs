// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use crate::board_settings::BoardSettings;
use probe_rs::probe::{DebugProbeInfo, Probe};
use probe_rs::{Core, Permissions, Session};
use std::time::Duration;
use tokio_serial::{FlowControl, Parity, SerialPort, SerialPortType, SerialStream, StopBits};

pub struct ProbeSession {
    pub probe_info: DebugProbeInfo,
    pub board_settings: BoardSettings,
    pub core_index: usize,
    pub probe: Option<Probe>,
    pub session: Option<Session>,
}

impl ProbeSession {
    pub fn new(
        probe_info: DebugProbeInfo,
        board_settings: BoardSettings,
        core_index: usize,
    ) -> Self {
        ProbeSession {
            probe_info,
            board_settings,
            core_index,
            probe: None,
            session: None,
        }
    }

    pub fn open(&mut self) {
        let serial_nr = self.probe_info.clone().serial_number.unwrap();
        self.probe = Some(self.probe_info.open().unwrap());
        // Take the Probe out of the Option, leaving None in its place
        if let Some(probe) = self.probe.take() {
            self.session = Some(
                probe
                    .attach(self.board_settings.chip.clone(), Permissions::default())
                    .unwrap(),
            );
        }
        let ports = tokio_serial::available_ports().unwrap();
        for port in ports {
            if let SerialPortType::UsbPort(inner) = port.port_type {
                if inner.serial_number.unwrap() == serial_nr {
                    // Open port and configure it with tokio_serial
                    let builder = tokio_serial::new(port.port_name, 115200);
                    match SerialStream::open(&builder) {
                        Ok(mut port) => {
                            println!("Serial port opened successfully!\n");
                            port.set_parity(Parity::None).unwrap();
                            port.set_stop_bits(StopBits::One).unwrap();
                            port.set_flow_control(FlowControl::None).unwrap();
                            port.set_timeout(Duration::from_millis(500)).unwrap();
                            port.write_request_to_send(false).unwrap();
                            port.write_data_terminal_ready(false).unwrap();
                        }
                        Err(e) => {
                            eprintln!("Failed to open serial port: {}\n", e);
                        }
                    }
                }
            }
        }
    }

    pub fn get_core(&mut self) -> Core {
        return self
            .session
            .as_mut()
            .unwrap()
            .core(self.core_index)
            .unwrap();
    }
}
