pub mod PIA6820;
pub mod address_spaces;
pub mod clock;
pub mod mc6502;
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
    fn read(address: usize) -> thread::Result<u8>;
    fn write(value: u8) -> thread::Result<()>;
    fn wire(options: IoComponentWireOptions) -> ();
}


struct IoComponentWireOptions {
    logicWrite: fn (value: number)  -> thread::Result<()>,
    logicRead: fn (address: number)  -> thread::Result<(u8)>
}


