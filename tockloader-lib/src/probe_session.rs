// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use crate::bootloader_serial::BootloaderSerial;
use probe_rs::probe::DebugProbeInfo;
use probe_rs::{Core, Permissions, Session};
use tokio_serial::SerialPortType;

pub struct ProbeSession {
    pub session: Option<Session>,
}

impl ProbeSession {
    pub fn new(probe_info: DebugProbeInfo, chip: &str) -> ProbeSession {
        let serial_nr = probe_info.clone().serial_number.unwrap();
        let mut probe = Some(probe_info.open().unwrap());
        let mut session = None;

        if let Some(probe) = probe.take() {
            session = Some(probe.attach(chip, Permissions::default()).unwrap());
        }
        let ports = tokio_serial::available_ports().unwrap();
        for port_info in ports {
            if let SerialPortType::UsbPort(ref inner) = port_info.port_type {
                if inner.serial_number.as_deref().unwrap() == serial_nr {
                    // Open port using the port info found
                    let _port = BootloaderSerial::new(port_info);
                }
            }
        }
        let probe_session = session.expect("Couldn't create a session");
        ProbeSession {
            session: Some(probe_session),
        }
    }

    pub fn get_core(&mut self, core_index: usize) -> Core {
        return self.session.as_mut().unwrap().core(core_index).unwrap();
    }
}
