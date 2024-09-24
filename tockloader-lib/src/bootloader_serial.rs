// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use crate::errors;
use bytes::BytesMut;
use errors::TockloaderError;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::SerialStream;

pub const SYNC_MESSAGE: [u8; 3] = [0x00, 0xFC, 0x05];
pub const ESCAPE_CHAR: u8 = 0xFC;

#[allow(dead_code)]
pub enum Command {
    // Commands from this tool to the bootloader
    Ping = 0x01,
    Info = 0x03,
    ID = 0x04,
    Reset = 0x05,
    ErasePage = 0x06,
    WritePage = 0x07,
    XEBlock = 0x08,
    XWPage = 0x09,
    Crcx = 0x10,
    ReadRange = 0x11,
    XRRange = 0x12,
    SetAttribute = 0x13,
    GetAttribute = 0x14,
    CRCInternalFlash = 0x15,
    Crcef = 0x16,
    XEPage = 0x17,
    XFinit = 0x18,
    ClkOut = 0x19,
    WUser = 0x20,
    ChangeBaudRate = 0x21,
    Exit = 0x22,
    SetStartAddress = 0x23,
}

#[derive(Clone, Debug, Copy)]
pub enum Response {
    // Responses from the bootloader
    Overflow = 0x10,
    Pong = 0x11,
    BadAddr = 0x12,
    IntError = 0x13,
    BadArgs = 0x14,
    OK = 0x15,
    Unknown = 0x16,
    XFTimeout = 0x17,
    Xfepe = 0x18,
    Crcrx = 0x19,
    ReadRange = 0x20,
    XRRange = 0x21,
    GetAttribute = 0x22,
    CRCInternalFlash = 0x23,
    Crcxf = 0x24,
    Info = 0x25,
    ChangeBaudFail = 0x26,
    BadResp,
}

impl From<u8> for Response {
    fn from(value: u8) -> Self {
        match value {
            0x10 => Response::Overflow,
            0x11 => Response::Pong,
            0x12 => Response::BadAddr,
            0x13 => Response::IntError,
            0x14 => Response::BadArgs,
            0x15 => Response::OK,
            0x16 => Response::Unknown,
            0x17 => Response::XFTimeout,
            0x18 => Response::Xfepe,
            0x19 => Response::Crcrx,
            0x20 => Response::ReadRange,
            0x21 => Response::XRRange,
            0x22 => Response::GetAttribute,
            0x23 => Response::CRCInternalFlash,
            0x24 => Response::Crcxf,
            0x25 => Response::Info,
            0x26 => Response::ChangeBaudFail,

            // This error handling is temmporary
            //TODO(Micu Ana): Add error handling
            _ => Response::BadResp,
        }
    }
}

pub trait BootloaderCommand<R> {
    async fn issue_command(&mut self) -> Result<R, TockloaderError>;
}

pub struct PingCommand<'a> {
    pub(crate) port: &'a mut SerialStream,
    pub(crate) sync: bool,
}

impl<'a> PingCommand<'a> {
    pub async fn ping_bootloader_and_wait_for_response(
        &mut self,
    ) -> Result<Response, TockloaderError> {
        let ping_pkt = [ESCAPE_CHAR, Command::Ping as u8];

        let mut ret = BytesMut::with_capacity(200);

        for _ in 0..30 {
            let mut bytes_written = 0;
            while bytes_written != ping_pkt.len() {
                bytes_written += self.port.write_buf(&mut &ping_pkt[bytes_written..]).await?;
            }
            let mut read_bytes = 0;
            while read_bytes < 2 {
                read_bytes += self.port.read_buf(&mut ret).await?;
            }
            if (ret.len() == 2) && ret[1] == Response::Pong as u8 {
                return Ok(Response::from(ret[1]));
            }
        }
        Ok(Response::from(ret[1]))
    }
}

pub struct WritePageCommand<'a> {
    pub(crate) port: &'a mut SerialStream,
    pub(crate) data: Vec<u8>,
    pub(crate) sync: bool,
    pub(crate) expected_response: Response,
    pub(crate) address: u32,
}

pub struct ReadRangeCommand<'a> {
    pub(crate) port: &'a mut SerialStream,
    pub(crate) length: u16,
    pub(crate) sync: bool,
    pub(crate) expected_response: Response,
    pub(crate) address: u32,
}

pub struct ErasePageCommand<'a> {
    pub(crate) port: &'a mut SerialStream,
    pub(crate) sync: bool,
    pub(crate) expected_response: Response,
    pub(crate) address: Vec<u8>,
}

impl<'a> BootloaderCommand<Response> for PingCommand<'a> {
    async fn issue_command(&mut self) -> Result<Response, TockloaderError> {
        let mut message = vec![];

        if self.sync {
            message.splice(0..0, SYNC_MESSAGE.iter().cloned());
        }

        message.push(ESCAPE_CHAR);
        message.push(Command::Ping as u8);

        let mut ret = BytesMut::with_capacity(2);
        let mut bytes_written = 0;

        while bytes_written != message.len() {
            bytes_written += self.port.write_buf(&mut &message[bytes_written..]).await?;
        }

        let mut read_bytes = 0;
        while read_bytes < 2 {
            read_bytes += self.port.read_buf(&mut ret).await?;
        }

        Ok(Response::from(ret[1]))
    }
}

impl<'a> BootloaderCommand<Response> for WritePageCommand<'a> {
    async fn issue_command(&mut self) -> Result<Response, TockloaderError> {
        let mut message = self.data.clone();

        for i in (0..message.len()).rev() {
            if message[i] == ESCAPE_CHAR {
                message.insert(i + 1, ESCAPE_CHAR);
            }
        }

        let address = self.address.to_le_bytes().to_vec();

        for (i, byte) in address.iter().enumerate().take(4)  {
            message.insert(i, *byte);
        }

        message.push(ESCAPE_CHAR);
        message.push(Command::WritePage as u8);

        if self.sync {
            message.splice(0..0, SYNC_MESSAGE.iter().cloned());
        }

        let mut bytes_written = 0;
        while bytes_written != message.len() {
            bytes_written += self.port.write_buf(&mut &message[bytes_written..]).await?;
        }

        let mut ret = BytesMut::with_capacity(2);
        let mut read_bytes = 0;

        while read_bytes < 2 {
            read_bytes += self.port.read_buf(&mut ret).await?;
        }

        if ret[1] != self.expected_response as u8 {
            return Err(TockloaderError::BootloaderError(ret[1]));
        }

        Ok(Response::from(ret[1]))
    }
}

impl<'a> BootloaderCommand<(Response, Vec<u8>)> for ReadRangeCommand<'a> {
    async fn issue_command(&mut self) -> Result<(Response, Vec<u8>), TockloaderError> {
        let mut message = vec![];

        let address = self.address.to_le_bytes().to_vec();
        let length = self.length.to_le_bytes().to_vec();

        for i in address.iter() {
            message.push(*i);
        }

        for i in length.iter() {
            message.push(*i);
        }

        message.push(ESCAPE_CHAR);
        message.push(Command::ReadRange as u8);

        if self.sync {
            message.splice(0..0, SYNC_MESSAGE.iter().cloned());
        }

        let mut bytes_written = 0;
        while bytes_written != message.len() {
            bytes_written += self.port.write_buf(&mut &message[bytes_written..]).await?;
        }

        let bytes_to_read = 2 + self.length;
        let mut ret = BytesMut::with_capacity(2);

        // We are waiting for 2 bytes to be read
        let mut read_bytes = 0;
        while read_bytes < 2 {
            read_bytes += self.port.read_buf(&mut ret).await?;
        }

        if ret[0] != ESCAPE_CHAR {
            return Err(TockloaderError::BootloaderError(ret[0]));
        }

        if ret[1] != self.expected_response as u8 {
            return Err(TockloaderError::BootloaderError(ret[1]));
        }

        let mut new_data: Vec<u8> = Vec::new();
        let mut value = 2;

        while bytes_to_read > value {
            value += self.port.read_buf(&mut new_data).await? as u16;
        }

        // De-escape and add array of read in the bytes
        for i in 0..(new_data.len() - 1) {
            if new_data[i] == ESCAPE_CHAR && new_data[i + 1] == ESCAPE_CHAR {
                new_data.remove(i + 1);
            }
        }

        ret.extend_from_slice(&new_data);

        Ok((Response::from(ret[1]), ret[2..].to_vec()))
    }
}

impl<'a> BootloaderCommand<Response> for ErasePageCommand<'a> {
    async fn issue_command(&mut self) -> Result<Response, TockloaderError> {
        let mut message = vec![];

        for i in 0..4 {
            message.push(self.address[i]);
        }

        message.push(ESCAPE_CHAR);
        message.push(Command::ErasePage as u8);

        if self.sync {
            message.splice(0..0, SYNC_MESSAGE.iter().cloned());
        }

        let mut bytes_written = 0;
        while bytes_written != message.len() {
            bytes_written += self.port.write_buf(&mut &message[bytes_written..]).await?;
        }

        let mut ret = BytesMut::with_capacity(2);
        let mut read_bytes = 0;

        while read_bytes < 2 {
            read_bytes += self.port.read_buf(&mut ret).await?;
        }

        if ret[1] != self.expected_response as u8 {
            return Err(TockloaderError::BootloaderError(ret[1]));
        }

        Ok(Response::from(ret[1]))
    }
}
