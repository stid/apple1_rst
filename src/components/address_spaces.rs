use super::IoAddressable;

struct AddressMapping {
    addr: [u8; 2],
    component: &dyn IoAddressable,
    name: String,
}

struct AddressSpaces {
    address_maps: Vec<AddressMapping>,
}

impl AddressSpaces {
    pub fn init(address_maps: Vec<AddressMapping>) -> AddressSpaces {
        AddressSpaces {
            address_maps: address_maps,
        }
    }

    fn _find_instance_with_address(&self, address: u8) -> Option<&AddressMapping> {
        self.address_maps
            .iter()
            .find(|&item| address >= item.addr[0] && address <= item.addr[1])
    }
}
