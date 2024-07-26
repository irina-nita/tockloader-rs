pub struct BoardSettings {
    pub board: String,
    pub chip: String,
    pub start_address: u64,
}

impl BoardSettings {
    pub fn new(
        board: String,
        chip: String,
    ) -> Self {
        match board.as_str() {
            "microbit_v2" => BoardSettings {
                board,
                chip,
                start_address: 0x0004_0000,
            },
            _ => BoardSettings {
                board,
                chip,
                start_address: 0x0003_0000,
            }
        }
    } 
}