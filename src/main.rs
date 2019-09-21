mod components;
use components::*;

fn main() {
    let the_mapping = vec![address_spaces::AddressMap {
        addr: [100, 200],
        component: Box::new(ram::Ram::init_with_size(100)),
        name: String::from("RAM"),
    }];

    let _cpu = mc6502::CPU6502::init(address_spaces::AddressSpaces::init(the_mapping));
}
