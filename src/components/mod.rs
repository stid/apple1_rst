pub mod address_spaces;
pub mod mc6502;
pub mod ram;
pub mod rom;
pub mod clock;
use std::fmt::Debug;

pub trait IoAddressable: Debug {
    fn read(&mut self, address: usize) -> u8;
    fn write(&mut self, address: usize, value: u8) -> ();
    fn flash(&mut self, data: &Vec<u8>) -> ();
}


pub trait Clockable: Debug  {
    fn get_cycles(&self) -> usize;
    fn step(&mut self) -> usize;
}
