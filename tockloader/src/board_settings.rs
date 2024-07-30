use probe_rs::{Core, MemoryInterface};

pub struct BoardSettings {
    pub board: String,
    pub chip: String,
    pub start_address: u64,
}

impl BoardSettings {
    pub fn new(board: String, chip: String) -> Self {
        match board.as_str() {
            "microbit_v2" => BoardSettings {
                board,
                chip,
                start_address: 0x0004_0000,
            },
            // TODO(MicuAna): inform the user we assumed we have the default settings if board is not found
            _ => BoardSettings {
                board,
                chip,
                start_address: 0x0003_0000,
            },
        }
    }
}

pub fn get_bootloader_version(mut board_core: Core) -> String {
    let address = 0x40E;

    let mut buf = [0u8; 10];

    board_core.read_8(address, &mut buf);

    let decoder = utf8_decode::Decoder::new(buf.iter().cloned());

    let mut string = String::new();
    for n in decoder {
        string.push(n.expect("Error decoding bootloader version"));
    }

    string
}
