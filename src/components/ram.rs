use super::IoAddressable;

#[derive(Debug)]
pub struct Ram {
    pub data: Vec<u8>,
    read_ops: u64,
    write_ops: u64,
}

impl Ram {
    pub fn init_with_size(size: usize) -> Ram {
        Ram {
            data: vec![0x0; size],
            read_ops: 0,
            write_ops: 0,
        }
    }
}

impl IoAddressable for Ram {
    fn read(&mut self, address: usize) -> u8 {
        self.read_ops += 1;
        return if self.data.len() > address {
            self.data[address]
        } else {
            0
        };
    }

    fn write(&mut self, address: usize, value: u8) -> () {
        self.write_ops += 1;
        self.data[address] = value;
    }
    fn flash(&mut self, data: &Vec<u8>) -> () {
        let prg_addr = u16::from_be_bytes([data[0], data[1]]);

        for i in 2..data.len() {
            self.data[(prg_addr as usize) + i - 2] = data[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state() {
        let mut rom = Ram::init_with_size(4);
        assert_eq!(0x0, rom.read(0));
        assert_eq!(0x0, rom.read(1));
        assert_eq!(0x0, rom.read(2));
        assert_eq!(0x0, rom.read(3));
    }

    #[test]
    fn initial_flash() {
        let mut rom = Ram::init_with_size(4);
        rom.flash(&vec![0, 2, 3, 4]);
        assert_eq!(0x0, rom.read(0));
        assert_eq!(0x0, rom.read(1));
        assert_eq!(3, rom.read(2));
        assert_eq!(4, rom.read(3));
    }

    #[test]
    fn should_write() {
        let mut rom = Ram::init_with_size(4);
        rom.write(0, 5);
        assert_eq!(5, rom.read(0));
        rom.write(3, 10);
        assert_eq!(10, rom.read(3));
    }

    #[test]
    fn should_increment_read_ops() {
        let mut ram = Ram::init_with_size(4);
        assert_eq!(0, ram.read_ops);
        ram.read(0);
        ram.read(0);
        ram.read(0);
        assert_eq!(3, ram.read_ops);
    }

    #[test]
    fn should_increment_write_ops() {
        let mut ram = Ram::init_with_size(4);
        assert_eq!(0, ram.write_ops);
        ram.write(0, 10);
        ram.write(0, 10);
        ram.write(0, 10);
        assert_eq!(3, ram.write_ops);
    }
}
