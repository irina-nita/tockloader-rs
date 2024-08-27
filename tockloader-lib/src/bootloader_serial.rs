// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

// The "X" commands are for external flash

use crate::errors;
use errors::TockloaderError;
use std::{io::{ErrorKind, Read}, time::Duration};
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

#[derive(Clone)]
#[derive(Debug)]
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
            _ => panic!("Invalid value for Response"),
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
        let builder = tokio_serial::new(port.port_name, 115200);
        match SerialStream::open(&builder) {
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

    #[allow(dead_code)]
    pub async fn issue_command(
        &mut self,
        command: Command,
        mut message: Vec<u8>,
        sync: bool,
        response_len: usize,
        response_code: Response,
    ) -> Result<Response, TockloaderError> {
        while let Ok(_) = self.port.as_mut().unwrap().read(&mut [0; 32]) {}
        //Setup a command to send to the bootloader and handle the response
        // Generate the message to send to the bootloader
        for i in 0..message.len() {
            if message[i] == ESCAPE_CHAR {
                // Escaped by replacing all 0xFC with two consecutive 0xFC - tock bootloader readme
                message.insert(i + 1, ESCAPE_CHAR);
            }
        }
        message.push(command as u8);


        // If there should be a sync/reset message, prepend the outgoing message with it
        if sync {
            message.insert(0, SYNC_MESSAGE[0]);
            message.insert(1, SYNC_MESSAGE[1]);
            message.insert(2, SYNC_MESSAGE[2]);
        }

        // Write the command message
        println!("Waiting for port to become writable");
        let writable_res = self.port.as_mut().unwrap().writable().await;
        println!("Writing {:?}", message);
        if let Ok(()) = writable_res {
            let _ = self.port.as_mut().unwrap().try_write(&message);
        }

        // Response has a two byte header, then response_len bytes
        let bytes_to_read = 2 + response_len;

        // Loop to read in that number of bytes starting with the header
        let mut ret: Vec<u8> = vec![0; 10];

            // We assume that try_read() method doesn't read more than 2 bytes, based on the fact that
            // if the return value of this method is [Ok(n)], then implementations must guarantee at
            // least on Linux that 0 <= n <= ret.len()
        tokio::time::sleep(Duration::from_millis(1000)).await;
        
        println!("Waiting for port to become readable");
        let readable_res = self.port.as_mut().unwrap().readable().await;
        if let Ok(()) = readable_res {
            match self.port.as_mut().unwrap().try_read(&mut ret[0..10]) {
                Ok(2) => {
                    println!("pare ok");
                    dbg!(&ret);
                },
    
                // This error handling is temmporary
                // TODO(Micu Ana): Add error handling
                // We have to stop at this point since otherwise
                // we loop waiting on data we will not get.
                Ok(0) => {
                    // As it is no value we have nothing to return
                    println!("no byte read");
                    return Err(errors::TockloaderError::IOError(
                        ErrorKind::InvalidData.into(),
                    ));
                }
                Ok(1) => {
                    println!("1 byte read");
                    return Ok(Response::from(ret[0]));
                }
    
                // This error handling is temmporary
                // TODO(Micu Ana): Add error handling
                Err(e) => {
                    println!("just error on attempt: {:?}", e);
                    dbg!(ret);
                    return Err(errors::TockloaderError::IOError(e));
                }
    
                // We should never end up in this case, as we can't read more than 2 values
                _ => {
                    dbg!(ret);
                    println!("nasol rau");
                    return Err(errors::TockloaderError::IOError(
                        ErrorKind::InvalidData.into(),
                    ));
                }
            }
        } 

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
            match self
                .port
                .as_mut()
                .unwrap()           
                .try_read(&mut new_data[0..(bytes_to_read - ret.len())])
            {
                Ok(value) => {
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
                        match self.port.as_mut().unwrap().try_read(&mut [byte]) {
                            Ok(_) => {
                                new_data.push(byte);
                            }

                            Err(e) => {
                                // TODO(Micu Ana): Add error handling
                                return Err(errors::TockloaderError::IOError(e));
                            }
                        }
                    }

                    // De-escape and add array of read in the bytes
                    for i in 0..(new_data.len() - 1) {
                        if new_data[i] == ESCAPE_CHAR {
                            if new_data[i + 1] == ESCAPE_CHAR {
                                new_data.remove(i + 1);
                            }
                        }
                    }

                    for i in 0..new_data.len() {
                        ret.push(new_data[i]);
                    }
                }

                Err(e) => {
                    // TODO(Micu Ana): Add error handling
                    return Err(errors::TockloaderError::IOError(e));
                }
            }
        }

        if ret.len() != (2 + response_len) {
            // TODO(Micu Ana): Add error handling
            return Ok(Response::from(ret[1]));
        }

        Ok(Response::from(ret[1]))
    }
}