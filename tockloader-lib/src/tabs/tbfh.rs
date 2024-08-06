use byteorder::{ByteOrder, LittleEndian};
#[derive(Debug)]
#[allow(dead_code)]
pub struct TBFHeader {
    is_app: bool,
    tbf_version: u16,
    header_size: u16,
    total_size: u32,
    flag: u32,
    checksum: u32,
}

impl TBFHeader {
    pub fn new(buf: Vec<u8>) -> Self {
        // Whether this TBF header is for an app, or is just padding
        let mut is_app = false;

        let tbf_version = LittleEndian::read_u16(&buf[0..2]);

        let header_size = LittleEndian::read_u16(&buf[2..4]);

        let total_size = LittleEndian::read_u32(&buf[4..8]);

        let flag = LittleEndian::read_u32(&buf[8..12]);

        let checksum = LittleEndian::read_u32(&buf[12..16]);

        let full_buffer = buf;

        if full_buffer.len() >= header_size as usize && header_size >= 16 {
            let mut nbuf = full_buffer[0..header_size as usize].to_vec();
            for i in 12..header_size {
                nbuf[i as usize] = 0;
            }
            let checksum = TBFHeader::_checksum(nbuf);
            let remaining = header_size - 16;
            if remaining > 0 && full_buffer.len() >= remaining as usize {
                // This is an application. That means we need more parsing
                is_app = true;
            }
        }

        TBFHeader {
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
