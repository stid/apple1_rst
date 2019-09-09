pub mod address_spaces;
pub mod mc6502;
pub mod ram;
pub mod rom;

pub trait IoAddressable {
    fn read(&mut self, address: usize) -> u8;
    fn write(&mut self, address: usize, value: u8) -> ();
    fn flash(&mut self, data: &Vec<u8>) -> ();
}
