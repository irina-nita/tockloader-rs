// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use std::collections::HashMap;

use probe_rs::{Core, MemoryInterface};

pub fn get_bootloader_version(board_core: &mut Core) -> String {
    let address = 0x40E;

    let mut buf = [0u8; 8];

    let _ = board_core.read_8(address, &mut buf);

    let decoder = utf8_decode::Decoder::new(buf.iter().cloned());

    let mut string = String::new();
    for n in decoder {
        string.push(n.expect("Error decoding bootloader version"));
    }

    let string = string.trim_matches(char::from(0));

    string.to_owned()
}

pub fn get_all_attributes(board_core: &mut Core) -> HashMap<String, String> {
    let address = 0x600;
    let mut buf = [0u8; 64 * 16];
    let _ = board_core.read(address, &mut buf);

    let mut data = buf.chunks(64);

    let mut attributes: HashMap<String, String> = HashMap::new();

    for _ in 0..data.len() {
        let step = match data.next() {
            Some(data) => data,
            None => break,
        };

        let step_option = decode_attribute(step);

        if step_option.is_none() {
            continue;
        }

        let step_hashmap = step_option.unwrap();

        attributes.insert(step_hashmap[0].to_string(), step_hashmap[1].to_string());
    }

    attributes
}

fn decode_attribute(step: &[u8]) -> Option<[String; 2]> {
    let raw_key = &step[0..8];

    let decoder_key = utf8_decode::Decoder::new(raw_key.iter().cloned());

    let mut key = String::new();
    for n in decoder_key {
        key.push(n.expect("Error getting key for attributes."));
    }

    key = key.trim_end_matches('\0').to_string();
    let vlen = step[8];

    if vlen > 55 || vlen == 0 {
        return None;
    }
    let raw_value = &step[9..(9 + vlen as usize)];
    let decoder_value = utf8_decode::Decoder::new(raw_value.iter().cloned());

    let mut value = String::new();

    for n in decoder_value {
        value.push(n.expect("Error getting key for attributes."));
    }

    value = value.trim_end_matches('\0').to_string();
    Some([key, value])
}
