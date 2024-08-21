// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

// The "X" commands are for external flash

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
    // Tell the bootloader to reset its buffer to handle a new command
    // SyncMessage = vec![0x00, 0xFC, 0x05],
}

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
