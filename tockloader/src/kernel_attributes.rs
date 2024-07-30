// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use std::{collections::HashMap, string};

use probe_rs::{Core, MemoryInterface};
use tbf_parser::parse;

pub fn kernel_attributes(board_core: &mut Core, attributes: &mut HashMap<String, String>)
{

    let address_apps = i32::from_str_radix(attributes
        .get("appaddr")
        .expect("Error encountered while getting the appaddr for kernel attributes.")
        .trim_start_matches("0x"), 16)
        .unwrap();
    let mut kernel_attr_binary = [0u8; 100];
    let _= board_core.read((address_apps - 100).try_into().unwrap(), &mut kernel_attr_binary);
    println!("{:?}",kernel_attr_binary);

    let sentinel = bytes_to_string(&kernel_attr_binary[96..100]);

    println!("Sentinel: {:?}", sentinel);

    let version = kernel_attr_binary[95];

    println!("Version: {:?}", version);
    
}


// TODO(RARES): will have to use this in board attributes too where needed to debload some of the code
pub fn bytes_to_string(raw: &[u8]) -> String {
    let decoder = utf8_decode::Decoder::new(raw.iter().cloned());

    let mut string = String::new();
    for n in decoder {
        string.push(n.expect("Error getting key for attributes."));
    }
    string
}