// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

// The "X" commands are for external flash

use crate::errors;
use bytes::BytesMut;
use errors::TockloaderError;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::SerialPortBuilderExt;
use tokio_serial::{FlowControl, Parity, SerialPort, SerialPortInfo, SerialStream, StopBits};

// Tell the bootloader to reset its buffer to handle a new command
pub const SYNC_MESSAGE: [u8; 3] = [0x00, 0xFC, 0x05];

// "This was chosen as it is infrequent in .bin files" - immesys
pub const ESCAPE_CHAR: u8 = 0xFC;

#[allow(dead_code)]
pub enum Command {
    // Commands from this tool to the bootloader
    CommandPing = 0x01,
    CommandInfo = 0x03,
    CommandID = 0x04,
    CommandReset = 0x05,
    CommandErasePage = 0x06,
    CommandWritePage = 0x07,
    CommandXEBlock = 0x08,
    CommandXWPage = 0x09,
    CommandCRCRX = 0x10,
    CommandReadRange = 0x11,
    CommandXRRange = 0x12,
    CommandSetAttribute = 0x13,
    CommandGetAttribute = 0x14,
    CommandCRCInternalFlash = 0x15,
    CommandCRCEF = 0x16,
    CommandXEPage = 0x17,
    CommandXFinit = 0x18,
    CommandClkOut = 0x19,
    CommandWUser = 0x20,
    CommandChangeBaudRate = 0x21,
    CommandExit = 0x22,
    CommandSetStartAddress = 0x23,
}

#[derive(Clone, Debug)]
pub enum Response {
    // Responses from the bootloader
    ResponseOverflow = 0x10,
    ResponsePong = 0x11,
    ResponseBadAddr = 0x12,
    ResponseIntError = 0x13,
    ResponseBadArgs = 0x14,
    ResponseOK = 0x15,
    ResponseUnknown = 0x16,
    ResponseXFTimeout = 0x17,
    ResponseXFEPE = 0x18,
    ResponseCRCRX = 0x19,
    ResponseReadRange = 0x20,
    ResponseXRRange = 0x21,
    ResponseGetAttribute = 0x22,
    ResponseCRCInternalFlash = 0x23,
    ResponseCRCXF = 0x24,
    ResponseInfo = 0x25,
    ResponseChangeBaudFail = 0x26,
    BadResponse,
}

impl From<u8> for Response {
    fn from(value: u8) -> Self {
        match value {
            0x10 => Response::ResponseOverflow,
            0x11 => Response::ResponsePong,
            0x12 => Response::ResponseBadAddr,
            0x13 => Response::ResponseIntError,
            0x14 => Response::ResponseBadArgs,
            0x15 => Response::ResponseOK,
            0x16 => Response::ResponseUnknown,
            0x17 => Response::ResponseXFTimeout,
            0x18 => Response::ResponseXFEPE,
            0x19 => Response::ResponseCRCRX,
            0x20 => Response::ResponseReadRange,
            0x21 => Response::ResponseXRRange,
            0x22 => Response::ResponseGetAttribute,
            0x23 => Response::ResponseCRCInternalFlash,
            0x24 => Response::ResponseCRCXF,
            0x25 => Response::ResponseInfo,
            0x26 => Response::ResponseChangeBaudFail,

            // This error handling is temmporary
            //TODO(Micu Ana): Add error handling
            _ => Response::BadResponse,
        }
    }
}

#[allow(dead_code)]
pub struct BootloaderSerial {
    port: Option<SerialStream>,
}

impl BootloaderSerial {
    pub fn new(port: SerialPortInfo) -> Self {
        // Open port and configure it with default settings using tokio_serial
        match tokio_serial::new(port.port_name, 115200).open_native_async() {
            Ok(mut port) => {
                port.set_parity(Parity::None).unwrap();
                port.set_stop_bits(StopBits::One).unwrap();
                port.set_flow_control(FlowControl::None).unwrap();
                port.set_timeout(Duration::from_millis(500)).unwrap();
                port.write_request_to_send(false).unwrap();
                port.write_data_terminal_ready(false).unwrap();
                return BootloaderSerial { port: Some(port) };
            }
            Err(_e) => {
                //TODO(Micu Ana): Add error handling
                return BootloaderSerial { port: None };
            }
        }
    }

    pub async fn toggle_bootloader_entry_dtr_rts(&mut self) {
        self.port
            .as_mut()
            .unwrap()
            .write_data_terminal_ready(true)
            .unwrap();
        self.port
            .as_mut()
            .unwrap()
            .write_request_to_send(true)
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.port
            .as_mut()
            .unwrap()
            .write_data_terminal_ready(false)
            .unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;
        self.port
            .as_mut()
            .unwrap()
            .write_request_to_send(false)
            .unwrap();
    }

    pub async fn ping_bootloader_and_wait_for_response(
        &mut self,
    ) -> Result<Response, TockloaderError> {
        let ping_pkt = vec![ESCAPE_CHAR, Command::CommandPing as u8];

        let mut ret = BytesMut::with_capacity(200);

        for i in 0..30 {
            println!("Iteration number {}", i);
            let mut bytes_written = 0;
            while bytes_written != ping_pkt.len() {
                bytes_written += self
                    .port
                    .as_mut()
                    .unwrap()
                    .write_buf(&mut &ping_pkt[bytes_written..])
                    .await?;
                println!("Wrote {} bytes", bytes_written);
            }
            let mut read_bytes = 0;
            while read_bytes < 2 {
                read_bytes += self.port.as_mut().unwrap().read_buf(&mut ret).await?;
            }
            println!("Read {} bytes", read_bytes);
            if ret[1] == Response::ResponsePong as u8 {
                return Ok(Response::from(ret[1]));
            }
        }
        // TODO(Micu Ana): Add error handling
        return Ok(Response::from(ret[1]));
    }

    #[allow(dead_code)]
    pub async fn issue_command(
        &mut self,
        command: Command,
        mut message: Vec<u8>,
        sync: bool,
        response_len: usize,
        response_code: Response,
    ) -> Result<Response, TockloaderError> {
        // Setup a command to send to the bootloader and handle the response
        // Generate the message to send to the bootloader
        let mut i = 0;
        while i < message.len() {
            if message[i] == ESCAPE_CHAR {
                // Escaped by replacing all 0xFC with two consecutive 0xFC - tock bootloader readme
                message.insert(i + 1, ESCAPE_CHAR);
                // Skip the inserted character
                i += 2;
            } else {
                i += 1;
            }
        }
        message.push(ESCAPE_CHAR);
        message.push(command as u8);

        // If there should be a sync/reset message, prepend the outgoing message with it
        if sync {
            message.insert(0, SYNC_MESSAGE[0]);
            message.insert(1, SYNC_MESSAGE[1]);
            message.insert(2, SYNC_MESSAGE[2]);
        }

        println!("Want to write {} bytes.", message.len());

        // Write the command message
        let mut bytes_written = 0;
        while bytes_written != message.len() {
            bytes_written += self
                .port
                .as_mut()
                .unwrap()
                .write_buf(&mut &message[bytes_written..])
                .await?;
        }
        println!("Wrote {} bytes", bytes_written);

        // Response has a two byte header, then response_len bytes
        let bytes_to_read = 2 + response_len;
        let mut ret = BytesMut::with_capacity(2);

        // We are waiting for 2 bytes to be read
        let mut read_bytes = 0;
        while read_bytes < 2 {
            read_bytes += self.port.as_mut().unwrap().read_buf(&mut ret).await?;
        }
        println!("Read {} bytes", read_bytes);
        println!("{:?}", ret);

        if ret[0] != ESCAPE_CHAR {
            //TODO(Micu Ana): Add error handling
            return Ok(Response::from(ret[1]));
        }

        if ret[1] != response_code.clone() as u8 {
            //TODO(Micu Ana): Add error handling
            return Ok(Response::from(ret[1]));
        }

        let mut new_data: Vec<u8> = Vec::new();

        while bytes_to_read - ret.len() > 0 {
            let value = self.port.as_mut().unwrap().read(&mut new_data).await?;

            // Odd number of escape characters
            // These can only come in pairs, so read another byte
            let mut count = 0;
            for i in 0..value {
                if new_data[i] == ESCAPE_CHAR {
                    count += 1;
                }
            }
            if count % 2 == 1 {
                let byte: u8 = 0;
                while byte == 0 {
                    self.port.as_mut().unwrap().read(&mut [byte]).await?;
                }

                new_data.push(byte);
            }

            // De-escape and add array of read in the bytes
            for i in 0..(new_data.len() - 1) {
                if new_data[i] == ESCAPE_CHAR {
                    if new_data[i + 1] == ESCAPE_CHAR {
                        new_data.remove(i + 1);
                    }
                }
            }

            ret.extend_from_slice(&new_data);
        }

        if ret.len() != (2 + response_len) {
            // TODO(Micu Ana): Add error handling
            return Ok(Response::from(ret[1]));
        }

        Ok(Response::from(ret[1]))
    }
}
