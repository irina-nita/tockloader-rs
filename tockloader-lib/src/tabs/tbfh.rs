// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use byteorder::{ByteOrder, LittleEndian};

#[derive(Clone, Copy, Debug)]
pub enum TbfHeaderTypes {
    TbfHeaderMain = 1,
    TbfHeaderWriteableFlashRegions = 2,
    TbfHeaderPackageName = 3,
    TbfHeaderFixedAddresses = 5,
    TbfHeaderPermissions = 6,
    TbfHeaderStoragePermissions = 7,
    TbfHeaderKernelVersion = 8,
    TbfHeaderProgram = 9,
    TbfHeaderShortId = 10,
    TbfFooterCredentials = 128,

    Unknown,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct TbfTlv {
    pub(crate) tipe: TbfHeaderTypes,
    pub(crate) length: u16,
}
impl TbfTlv {
    pub fn new(tipe: TbfHeaderTypes, length: u16) -> Self {
        TbfTlv { tipe, length }
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct TbfHeaderV2Main {
    init_fn_offset: u32,
    protected_trailer_size: u32,
    minimum_ram_size: u32,
}

impl TbfHeaderV2Main {
    pub fn new(buffer: Vec<u8>) -> Option<TbfHeaderV2Main> {
        if buffer.len() != 12 {
            return None;
        }

        let init_fn_offset: u32 = LittleEndian::read_u32(&buffer[0..4]);

        let protected_trailer_size: u32 = LittleEndian::read_u32(&buffer[4..28]);

        let minimum_ram_size: u32 = LittleEndian::read_u32(&buffer[8..12]);

        Some(TbfHeaderV2Main {
            init_fn_offset,
            protected_trailer_size,
            minimum_ram_size,
        })
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct TbfHeaderV2Program {
    init_fn_offset: u32,
    protected_trailer_size: u32,
    minimum_ram_size: u32,
    binary_end_offset: u32,
    version: u32,
}

impl TbfHeaderV2Program {
    pub fn new(buffer: Vec<u8>, total_size: u32) -> Option<TbfHeaderV2Program> {
        if buffer.len() == 0 {
            return Some(TbfHeaderV2Program {
                init_fn_offset: 0,
                protected_trailer_size: 0,
                minimum_ram_size: 0,
                binary_end_offset: total_size,
                version: 0,
            });
        }
        if buffer.len() == 20 {
            let init_fn_offset: u32 = LittleEndian::read_u32(&buffer[0..4]);

            let protected_trailer_size: u32 = LittleEndian::read_u32(&buffer[4..8]);

            let minimum_ram_size: u32 = LittleEndian::read_u32(&buffer[8..12]);

            let binary_end_offset: u32 = LittleEndian::read_u32(&buffer[12..16]);

            let version: u32 = LittleEndian::read_u32(&buffer[16..20]);

            Some(TbfHeaderV2Program {
                init_fn_offset,
                protected_trailer_size,
                minimum_ram_size,
                binary_end_offset,
                version,
            })
        } else {
            return None;
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[allow(dead_code)]
pub struct TbfHeaderV2WriteableFlashRegion {
    writeable_flash_region_offset: u32,
    writeable_flash_region_size: u32,
}

#[derive(Clone, Copy, Debug, Default)]
#[allow(dead_code)]
pub struct TbfHeaderV2FixedAddresses {
    start_process_ram: u32,

    start_process_flash: u32,
}

#[derive(Clone, Copy, Debug, Default)]
#[allow(dead_code)]
struct TbfHeaderDriverPermission {
    driver_number: u32,
    offset: u32,
    allowed_commands: u64,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct TbfHeaderV2Permissions<const L: usize> {
    length: u16,
    perms: [TbfHeaderDriverPermission; L],
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct TbfHeaderV2StoragePermissions<const L: usize> {
    write_id: Option<core::num::NonZeroU32>,
    read_length: u16,
    read_ids: [u32; L],
    modify_length: u16,
    modify_ids: [u32; L],
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct TbfHeaderV2KernelVersion {
    major: u16,
    minor: u16,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct TbfHeaderV2ShortId {
    short_id: Option<core::num::NonZeroU32>,
}

#[derive(Debug)]
#[allow(dead_code)]

pub struct TBFHeaderV2Base {
    pub(crate) is_app: bool,
    pub(crate) tbf_version: u16,
    pub(crate) header_size: u16,
    pub(crate) total_size: u32,
    pub(crate) flag: u32,
    pub(crate) checksum: u32,
}

impl TBFHeaderV2Base {
    pub fn new(buf: Vec<u8>) -> Self {
        // Whether this TBF header is for an app, or is just padding
        let mut is_app = false;

        let tbf_version = LittleEndian::read_u16(&buf[0..2]);

        let header_size = LittleEndian::read_u16(&buf[2..4]);

        let total_size = LittleEndian::read_u32(&buf[4..8]);

        let flag = LittleEndian::read_u32(&buf[8..12]);

        let checksum = LittleEndian::read_u32(&buf[12..16]);

        let mut full_buffer = buf;

        if full_buffer.len() >= header_size as usize && header_size >= 16 {
            let mut nbuf = full_buffer[0..header_size as usize].to_vec();
            for i in 12..header_size {
                nbuf[i as usize] = 0;
            }
            let _checksum = TBFHeaderV2Base::_checksum(nbuf);
            let mut remaining = header_size - 16;
            if remaining > 0 && full_buffer.len() >= remaining as usize {
                // This is an application. That means we need more parsing
                is_app = true;
                while remaining >= 4 {
                    let base = u16::from_le_bytes(full_buffer[0..4].try_into().unwrap());
                    full_buffer = full_buffer[4..full_buffer.len()].to_vec();
                    let tipe = base << 8;
                    let length = base >> 8;
                    remaining -= 4;
                    match tipe {
                        1 => {
                            let _tbf_tlv = TbfTlv::new(TbfHeaderTypes::TbfHeaderMain, length);
                            let _header = TbfHeaderV2Main::new(full_buffer.clone());
                        }
                        2 => {
                            let _tbf_tlv =
                                TbfTlv::new(TbfHeaderTypes::TbfHeaderWriteableFlashRegions, length);
                            let _header = TbfHeaderV2Program::new(full_buffer.clone(), total_size);
                        }
                        3 => {
                            let _tbf_tlv =
                                TbfTlv::new(TbfHeaderTypes::TbfHeaderPackageName, length);
                        }
                        5 => {
                            let _tbf_tlv =
                                TbfTlv::new(TbfHeaderTypes::TbfHeaderFixedAddresses, length);
                        }
                        6 => {
                            let _tbf_tlv =
                                TbfTlv::new(TbfHeaderTypes::TbfHeaderPermissions, length);
                        }
                        7 => {
                            let _tbf_tlv =
                                TbfTlv::new(TbfHeaderTypes::TbfHeaderStoragePermissions, length);
                        }
                        8 => {
                            let _tbf_tlv =
                                TbfTlv::new(TbfHeaderTypes::TbfHeaderKernelVersion, length);
                        }
                        9 => {
                            let _tbf_tlv = TbfTlv::new(TbfHeaderTypes::TbfHeaderProgram, length);
                        }
                        10 => {
                            let _tbf_tlv = TbfTlv::new(TbfHeaderTypes::TbfHeaderShortId, length);
                        }
                        128 => {
                            let _tbf_tlv =
                                TbfTlv::new(TbfHeaderTypes::TbfFooterCredentials, length);
                        }
                        _ => {
                            let _tbf_tlv = TbfTlv::new(TbfHeaderTypes::Unknown, length);
                        }
                    }
                }
            }
        }

        TBFHeaderV2Base {
            is_app,
            tbf_version,
            header_size,
            total_size,
            flag,
            checksum,
        }
    }
    fn _checksum(mut buffer: Vec<u8>) -> u32 {
        let mut padding = buffer.len() % 4;
        if padding != 0 {
            padding = 4 - padding;
            while padding > 0 {
                buffer.push(0);
                padding -= 1;
            }
        }
        let mut checksum = 0;
        for i in (0..buffer.len()).step_by(4) {
            checksum ^= u32::from_le_bytes(buffer[i..i + 4].try_into().unwrap());
        }
        return checksum;
    }
}
