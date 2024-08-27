use probe_rs::{
    config::ScanChainElement,
    probe::{DebugProbe, DebugProbeError, Probe, WireProtocol},
    MemoryInterface,
};

#[derive(Debug)]
pub struct MockProbe {}

impl MockProbe {
    pub fn into_probe(self) -> Probe {
        Probe::from_specific_probe(Box::new(self))
    }
}

impl MemoryInterface for MockProbe {
    fn supports_native_64bit_access(&mut self) -> bool {
        true
    }

    fn read_word_64(&mut self, address: u64) -> Result<u64, probe_rs::Error> {
        todo!()
    }

    fn read_word_32(&mut self, address: u64) -> Result<u32, probe_rs::Error> {
        todo!()
    }

    fn read_word_16(&mut self, address: u64) -> Result<u16, probe_rs::Error> {
        todo!()
    }

    fn read_word_8(&mut self, address: u64) -> Result<u8, probe_rs::Error> {
        todo!()
    }

    fn read_64(&mut self, address: u64, data: &mut [u64]) -> Result<(), probe_rs::Error> {
        todo!()
    }

    fn read_32(&mut self, address: u64, data: &mut [u32]) -> Result<(), probe_rs::Error> {
        todo!()
    }

    fn read_16(&mut self, address: u64, data: &mut [u16]) -> Result<(), probe_rs::Error> {
        todo!()
    }

    fn read_8(&mut self, address: u64, data: &mut [u8]) -> Result<(), probe_rs::Error> {
        todo!()
    }

    fn write_word_64(&mut self, address: u64, data: u64) -> Result<(), probe_rs::Error> {
        todo!()
    }

    fn write_word_32(&mut self, address: u64, data: u32) -> Result<(), probe_rs::Error> {
        todo!()
    }

    fn write_word_16(&mut self, address: u64, data: u16) -> Result<(), probe_rs::Error> {
        todo!()
    }

    fn write_word_8(&mut self, address: u64, data: u8) -> Result<(), probe_rs::Error> {
        todo!()
    }

    fn write_64(&mut self, address: u64, data: &[u64]) -> Result<(), probe_rs::Error> {
        todo!()
    }

    fn write_32(&mut self, address: u64, data: &[u32]) -> Result<(), probe_rs::Error> {
        todo!()
    }

    fn write_16(&mut self, address: u64, data: &[u16]) -> Result<(), probe_rs::Error> {
        todo!()
    }

    fn write_8(&mut self, address: u64, data: &[u8]) -> Result<(), probe_rs::Error> {
        todo!()
    }

    fn supports_8bit_transfers(&self) -> Result<bool, probe_rs::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), probe_rs::Error> {
        todo!()
    }
}

impl DebugProbe for MockProbe {
    fn get_name(&self) -> &str {
        "tockloader-rs-lib Mock Probe"
    }

    fn speed_khz(&self) -> u32 {
        todo!()
    }

    fn set_speed(&mut self, speed_khz: u32) -> Result<u32, DebugProbeError> {
        todo!()
    }

    fn set_scan_chain(&mut self, scan_chain: Vec<ScanChainElement>) -> Result<(), DebugProbeError> {
        todo!()
    }

    fn scan_chain(&self) -> Result<&[ScanChainElement], DebugProbeError> {
        todo!()
    }

    fn attach(&mut self) -> Result<(), DebugProbeError> {
        Ok(())
    }

    fn detach(&mut self) -> Result<(), probe_rs::Error> {
        todo!()
    }

    fn target_reset(&mut self) -> Result<(), DebugProbeError> {
        todo!()
    }

    fn target_reset_assert(&mut self) -> Result<(), DebugProbeError> {
        todo!()
    }

    fn target_reset_deassert(&mut self) -> Result<(), DebugProbeError> {
        todo!()
    }

    fn select_protocol(&mut self, protocol: WireProtocol) -> Result<(), DebugProbeError> {
        todo!()
    }

    fn active_protocol(&self) -> Option<WireProtocol> {
        todo!()
    }

    fn into_probe(self: Box<Self>) -> Box<dyn DebugProbe> {
        self
    }
    
    /**** DEFUALT ****/
    
    fn has_arm_interface(&self) -> bool {
        true
    }
    
    // fn try_get_arm_interface<'probe>(
    //     self: Box<Self>,
    // ) -> Result<Box<dyn UninitializedArmProbe + 'probe>, (Box<dyn DebugProbe>, DebugProbeError)>
    // {
    //     todo!()
    // }    
}
