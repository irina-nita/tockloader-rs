// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

// The "X" commands are for external flash

use crate::errors;
use errors::TockloaderError;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::{SerialPort, SerialStream};
use std::time::Duration;
use bytes::BytesMut;

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
    ResponseOverflow,
    ResponsePong,
    ResponseBadAddr,
    ResponseIntError,
    ResponseBadArgs,
    ResponseOK,
    ResponseUnknown,
    ResponseXFTimeout,
    ResponseXFEPE,
    ResponseCRCRX,
    ResponseReadRange,
    ResponseXRRange,
    ResponseGetAttribute,
    ResponseCRCInternalFlash,
    ResponseCRCXF,
    ResponseInfo,
    ResponseChangeBaudFail,
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
pub async fn toggle_bootloader_entry_dtr_rts(port: &mut SerialStream) {
    port
        .write_data_terminal_ready(true)
        .unwrap();
    port
        .write_request_to_send(true)
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    port
        .write_data_terminal_ready(false)
        .unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;
    port
        .write_request_to_send(false)
        .unwrap();
}

#[allow(dead_code)]
pub async fn ping_bootloader_and_wait_for_response(
    port: &mut SerialStream,
) -> Result<Response, TockloaderError> {
    let ping_pkt = vec![ESCAPE_CHAR, Command::CommandPing as u8];

    let mut ret = BytesMut::with_capacity(200);

    for i in 0..30 {
        println!("Iteration number {}", i);
        let mut bytes_written = 0;
        while bytes_written != ping_pkt.len() {
            bytes_written += 
                port
                .write_buf(&mut &ping_pkt[bytes_written..])
                .await?;
            println!("Wrote {} bytes", bytes_written);
        }
        let mut read_bytes = 0;
        while read_bytes < 2 {
            read_bytes += port.read_buf(&mut ret).await?;
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
    port: &mut SerialStream,
    command: Command,
    mut message: Vec<u8>,
    sync: bool,
    response_len: usize,
    response_code: Response,
) -> Result<(Response, Vec<u8>), TockloaderError> {
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
        bytes_written += 
            port
            .write_buf(&mut &message[bytes_written..])
            .await?;
    }
    println!("Wrote {} bytes", bytes_written);

    tokio::time::sleep(Duration::from_millis(100)).await;
    // Response has a two byte header, then response_len bytes
    let bytes_to_read = 2 + response_len;
    let mut ret = BytesMut::with_capacity(2);

    // We are waiting for 2 bytes to be read
    let mut read_bytes = 0;
    while read_bytes < 2 {
        read_bytes += port.read_buf(&mut ret).await?;
    }
    println!("Read {} bytes", read_bytes);
    println!("{:?}", ret);

    if ret[0] != ESCAPE_CHAR {
        //TODO(Micu Ana): Add error handling
        return Ok((Response::from(ret[1]), vec![]));
    }

    if ret[1] != response_code.clone() as u8 {
        //TODO(Micu Ana): Add error handling
        return Ok((Response::from(ret[1]), vec![]));
    }

    let mut new_data: Vec<u8> = Vec::new();

    while bytes_to_read - ret.len() > 0 {
        let value = port.read(&mut new_data).await?;

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
                port.read(&mut [byte]).await?;
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
        return Ok((Response::from(ret[1]), vec![]));
    }

    Ok((Response::from(ret[1]), ret[2..].to_vec()))
}
