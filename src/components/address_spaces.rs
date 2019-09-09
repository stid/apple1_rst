use super::IoAddressable;

pub struct AddressMap {
    pub addr: [u16; 2],
    pub component: Box<dyn IoAddressable>,
    pub name: String,
}

pub struct AddressSpaces {
    address_maps: Vec<AddressMap>,
}

impl AddressSpaces {
    pub fn init(address_maps: Vec<AddressMap>) -> AddressSpaces {
        AddressSpaces {
            address_maps: address_maps,
        }
    }

    fn _find_instance_with_address(&mut self, address: u16) -> Option<&mut AddressMap> {
        let result = self
            .address_maps
            .iter_mut()
            .find(|item| address >= item.addr[0] && address <= item.addr[1]);

        return result;
    }

    pub fn read(&mut self, address: u16) -> u8 {
        let addr_mapping = self._find_instance_with_address(address);

        match addr_mapping {
            None => 0,
            Some(addr_mapping) => {
                let res = &mut addr_mapping.component;
                return res.read((address - addr_mapping.addr[0]) as usize);
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8) -> () {
        let addr_mapping = self._find_instance_with_address(address);

        match addr_mapping {
            None => {
                0;
            }
            Some(addr_mapping) => {
                let res = &mut addr_mapping.component;
                res.write((address - addr_mapping.addr[0]) as usize, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAddressable {}

    impl IoAddressable for TestAddressable {
        fn read(&mut self, address: usize) -> u8 {
            b'a'
        }
        fn write(&mut self, address: usize, value: u8) -> () {}
        fn flash(&mut self, data: &Vec<u8>) -> () {}
    }

    #[test]
    fn initial_state() {
        let addressable = TestAddressable {};

        let the_mapping = vec![AddressMap {
            addr: [100, 200],
            component: Box::new(addressable),
            name: String::from("MyNiceComp"),
        }];
        let result = &mut AddressSpaces::init(the_mapping);
        assert_eq!(b'a', result.read(100));
        assert_eq!(0x00, result.read(5));
    }

}
