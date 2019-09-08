use super::IoAddressable;

#[derive(Debug)]
pub struct Rom {
    pub data: Vec<u8>,
    read_ops: u64,
}

impl Rom {
    pub fn init_with_size(size: usize) -> Rom {
        Rom {
            data: vec![0xFF; size],
            read_ops: 0,
        }
    }
}

impl IoAddressable for Rom {
    fn read(&mut self, address: usize) -> u8 {
        self.read_ops += 1;
        return if self.data.len() > address {
            self.data[address]
        } else {
            0
        };
    }

    fn write(&mut self, _address: usize, _value: u8) -> () {}

    fn flash(&mut self, data: &Vec<u8>) -> () {
        for i in 2..data.len() {
            self.data[i - 2] = data[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state() {
        let mut rom = Rom::init_with_size(4);
        assert_eq!(0xFF, rom.read(0));
        assert_eq!(0xFF, rom.read(1));
        assert_eq!(0xFF, rom.read(2));
        assert_eq!(0xFF, rom.read(3));
    }

    #[test]
    fn initial_flash() {
        let mut rom = Rom::init_with_size(4);
        rom.flash(&vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(3, rom.read(0));
        assert_eq!(4, rom.read(1));
        assert_eq!(5, rom.read(2));
        assert_eq!(6, rom.read(3));
    }

    #[test]
    fn should_not_write() {
        let mut rom = Rom::init_with_size(4);
        rom.write(0, 5);
        assert_eq!(0xFF, rom.read(0));
    }

    #[test]
    fn should_increment_read_ops() {
        let mut rom = Rom::init_with_size(4);
        rom.read(0);
        rom.read(0);
        rom.read(0);
        assert_eq!(3, rom.read_ops);
    }
}
