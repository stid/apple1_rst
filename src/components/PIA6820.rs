use super::IoAddressable;
use super::IoComponent;


// PIA MAPPING 6821
const DATA_A_ADDR: u8 = 0x0;
const CRT_A_ADDR: u8 = 0x1;

const DATA_B_ADDR: u8 = 0x2;
const CRT_B_ADDR: u8 = 0x3;

#[derive(Debug)]
pub struct PIA6820 {
    pub data: Vec<u8, 4>,
    ioA: Option<Box<dyn IoComponent>>,
    ioB: Option<Box<dyn IoComponent>>,
}

impl PIA6820 {
    pub fn new() -> PIA6820 {
        PIA6820 {
            data: vec![0x00; 4],
            ioA: None,
            ioB: None
        }
    }

    pub fn wireIOA(&mut self, ioA: Box<dyn IoComponent>) -> () {
        self.ioA = ioA;
    }

    pub fn wireIOB(&mut self, ioB: Box<dyn IoComponent>) -> () {
        self.ioB = ioB;
    }

    pub fn setBitDataA(&mut self, bit: u8) -> () {
        //self.data[DATA_A_ADDR] = self.data[DATA_A_ADDR] ~= (1 << bit);
    }

}
