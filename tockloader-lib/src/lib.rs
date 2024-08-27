// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

pub mod attributes;
mod errors;
pub mod probe_session;
mod mock_probe;

use attributes::app_attributes::AppAttributes;
use attributes::general_attributes::GeneralAttributes;
use attributes::system_attributes::SystemAttributes;

use probe_rs::integration::FakeProbe;
use probe_rs::probe::DebugProbeInfo;
use probe_rs::Permissions;
use probe_session::ProbeSession;

pub async fn list_probe(
    choice: DebugProbeInfo,
    chip: &str,
    core_index: &usize,
) -> Vec<AppAttributes> {
    let mut probe_session = ProbeSession::new(choice, chip);
    let mut core = probe_session.get_core(*core_index);

    let system_attributes = SystemAttributes::read_system_attributes(&mut core);

    AppAttributes::read_apps_data(&mut core, system_attributes.appaddr.unwrap())
}

pub async fn info_probe(
    choice: DebugProbeInfo,
    chip: &str,
    core_index: &usize,
) -> GeneralAttributes {
    let mut fake_probe = FakeProbe::with_mocked_core();
    fake_probe.set_arm_read_handler(Box::new(|addr, data| {
        println!("Read memory: {:?} {:?}", addr, data);
        Ok(())
    })).unwrap();
    fake_probe.set_arm_write_handler(Box::new(|addr, data| {
        println!("Write memory: {:?} {:?}", addr, data);
        Ok(())
    })).unwrap();

    let mut probe_session: ProbeSession = ProbeSession { session: Some(fake_probe.into_probe().attach("nrf51822_xxAC", Permissions::default()).unwrap()) };


    let mut core = probe_session.get_core(*core_index);

    let system_attributes = SystemAttributes::read_system_attributes(&mut core);

    let apps_details = AppAttributes::read_apps_data(&mut core, system_attributes.appaddr.unwrap());

    GeneralAttributes::new(system_attributes, apps_details)
}

/*
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let fake_probe = MockProbe {};
    let probe = fake_probe.into_probe();
    let probe_rs_session = probe
        .attach("nrf51822_xxAC", Permissions::default())
        .unwrap();
    let mut probe_session: ProbeSession = ProbeSession {
        session: Some(probe_rs_session),
    };
*/