use super::IoAddressable;
use super::IoComponent;

// PIA MAPPING 6821
const DATA_A_ADDR: u8 = 0x0;
const CRT_A_ADDR: u8 = 0x1;

const DATA_B_ADDR: u8 = 0x2;
const CRT_B_ADDR: u8 = 0x3;

#[derive(Debug)]
pub struct pia6820 {
    pub data: Vec<u8>,
    io_a: Option<Box<dyn IoComponent>>,
    io_b: Option<Box<dyn IoComponent>>,
}

impl pia6820 {
    pub fn new() -> pia6820 {
        pia6820 {
            data: vec![0x00; 4],
            io_a: None,
            io_b: None,
        }
    }

    pub fn wire_ioa(&mut self, io_a: Option<Box<dyn IoComponent>>) -> () {
        self.io_a = io_a;
    }

    pub fn wire_iob(&mut self, io_b: Option<Box<dyn IoComponent>>) -> () {
        self.io_b = io_b;
    }
}
