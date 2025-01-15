use std::sync::Arc;

use parking_lot::FairMutex;
use probe_rs::{probe::DebugProbeInfo, Permissions};

use crate::errors::TockloaderError;

pub enum ConnectionInfo {
    ProbeInfo(DebugProbeInfo),
}

pub enum Connection {
    // A probe-rs session can be shared between threads.
    ProbeRS(Arc<FairMutex<probe_rs::Session>>),
}

impl Connection {
    pub fn open(info: ConnectionInfo, chip: Option<String>) -> Result<Connection, TockloaderError> {
        match info {
            ConnectionInfo::ProbeInfo(probe_info) => {
                let probe = probe_info
                    .open()
                    .map_err(TockloaderError::ProbeRsInitializationError)?;
                match probe.attach(chip.unwrap(), Permissions::default()) {
                    Ok(session) => Ok(Connection::ProbeRS(Arc::new(FairMutex::new(session)))),
                    Err(e) => Err(TockloaderError::ProbeRsCommunicationError(e)),
                }
            }
        }
    }
}
