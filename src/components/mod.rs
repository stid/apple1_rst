pub mod address_spaces;
pub mod clock;
pub mod mc6502;
pub mod pia6820;
pub mod ram;
pub mod rom;
use std::fmt::Debug;
use std::thread;

pub trait IoAddressable: Debug {
    fn read(&mut self, address: usize) -> u8;
    fn write(&mut self, address: usize, value: u8) -> ();
    fn flash(&mut self, data: &Vec<u8>) -> ();
}

pub trait Clockable: Debug {
    fn get_cycles(&self) -> usize;
    fn step(&mut self) -> usize;
}

pub trait IoComponent: Debug {
    fn read(&mut self, address: usize) -> thread::Result<u8>;
    fn write(&mut self, value: u8) -> thread::Result<()>;
    fn wire(&mut self, options: IoComponentWireOptions) -> ();
}

struct IoComponentWireOptions {
    logic_write: fn(value: u8) -> thread::Result<()>,
    logic_read: fn(address: usize) -> thread::Result<u8>,
}
