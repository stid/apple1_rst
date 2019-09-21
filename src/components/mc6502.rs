use super::address_spaces::AddressSpaces;
use super::Clockable;

#[derive(Debug)]
pub struct CPU6502 {
    address_spaces: AddressSpaces,
    PC: u16,

    A: u8,
    X: u8,
    Y: u8,
    S: u8,

    N: bool,
    Z: bool,
    C: bool,
    V: bool,

    I: bool,
    D: bool,

    irq: bool,
    nmi: bool,

    tmp: u16,
    addr: u16,

    opcode: u8,
    cycles: usize,
}

impl CPU6502 {
    pub fn init(address_spaces: AddressSpaces) -> CPU6502 {
        let cpu = CPU6502 {
            address_spaces: address_spaces,
            PC: 0,
            A: 0,
            X: 0,
            Y: 0,
            S: 0,
            N: false,
            Z: false,
            C: false,
            V: false,
            I: false,
            D: false,
            irq: false,
            nmi: false,
            tmp: 0,
            addr: 0,
            opcode: 0,
            cycles: 0,
        };

        return cpu;
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Subroutines - addressing modes & flags
    ////////////////////////////////////////////////////////////////////////////////

    fn izx(&mut self) {
        let a: u16 = (self.read16(self.PC).wrapping_add(self.X as u16)) & 0xFF;
        self.PC += 1;
        self.addr = (self.read16(a.wrapping_add(1)) << 8) | self.read16(a);
        self.cycles += 6;
    }

    fn izy(&mut self) {
        let a: u16 = self.read16(self.PC);
        self.PC += 1;
        let paddr: u16 = (self.read16((a.wrapping_add(1)) & 0xFF) << 8) | self.read16(a);
        self.addr = paddr.wrapping_add(self.Y as u16);
        if (paddr & 0x100) != (self.addr & 0x100) {
            self.cycles += 6;
        } else {
            self.cycles += 5;
        }
    }

    fn ind(&mut self) -> () {
        let mut a = self.read16(self.PC);
        a |= (self.read16((self.PC & 0xff00) | ((self.PC.wrapping_add(1)) & 0xff))) << 8;
        self.addr = self.read16(a);
        self.addr |= (self.read16(a.wrapping_add(1))) << 8;
        self.cycles += 5;
    }

    fn zp(&mut self) -> () {
        self.addr = self.read16(self.PC);
        self.PC += 1;
        self.cycles += 3;
    }

    fn zpx(&mut self) -> () {
        self.addr = (self.read16(self.PC) + self.X as u16) & 0xff;
        self.PC += 1;
        self.cycles += 4;
    }

    fn zpy(&mut self) -> () {
        self.addr = (self.read16(self.PC) + self.Y as u16) & 0xff;
        self.PC += 1;
        self.cycles += 4;
    }

    fn imp(&mut self) -> () {
        self.cycles += 2;
    }

    fn imm(&mut self) -> () {
        self.PC += 1;
        self.addr = self.PC;
        self.cycles += 2;
    }

    fn abs(&mut self) -> () {
        self.addr = self.read16(self.PC);
        self.PC += 1;
        self.addr |= (self.read16(self.PC)) << 8;
        self.PC += 1;
        self.cycles += 4;
    }

    fn abx(&mut self) -> () {
        let mut paddr = self.read16(self.PC);
        self.PC += 1;
        paddr |= (self.read16(self.PC)) << 8;
        self.PC += 1;
        self.addr = paddr + (self.X as u16);
        if (paddr & 0x100) != (self.addr & 0x100) {
            self.cycles += 5;
        } else {
            self.cycles += 4;
        }
    }

    fn aby(&mut self) -> () {
        let mut paddr = self.read16(self.PC);
        self.PC += 1;
        paddr |= (self.read16(self.PC)) << 8;
        self.PC += 1;
        self.addr = paddr + (self.Y as u16);
        if (paddr & 0x100) != (self.addr & 0x100) {
            self.cycles += 5;
        } else {
            self.cycles += 4;
        }
    }

    fn rel(&mut self) -> () {
        self.addr = self.read16(self.PC);
        self.PC += 1;
        if self.addr & 0x80 != 0 {
            self.addr -= 0x100;
        }
        self.addr += self.PC;
        self.cycles += 2;
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn rmw(&mut self) -> () {
        self.write(self.addr, (self.tmp & 0xff) as u8);
        self.cycles += 2;
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn fnz(&mut self, v: u16) -> () {
        self.Z = (v & 0xFF) == 0;
        self.N = (v & 0x80) != 0;
    }

    // Borrow
    fn fnzb(&mut self, v: u16) -> () {
        self.Z = (v & 0xFF) == 0;
        self.N = (v & 0x80) != 0;
        self.C = (v & 0x100) == 0;
    }

    // Carry
    fn fnzc(&mut self, v: u16) -> () {
        self.Z = (v & 0xFF) == 0;
        self.N = (v & 0x80) != 0;
        self.C = (v & 0x100) != 0;
    }

    fn branch(&mut self, taken: bool) -> () {
        if taken {
            if (self.addr & 0x100) != (self.PC & 0x100) {
                self.cycles += 2;
            } else {
                self.cycles += 1;
            }
            self.PC = self.addr;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Subroutines - instructions
    ////////////////////////////////////////////////////////////////////////////////
    fn adc(&mut self) -> () {
        let v = self.read(self.addr);
        let c = if self.C { 1 } else { 0 };
        let r = self.A.wrapping_add(v).wrapping_add(c);
        if self.D {
            let mut al = (self.A & 0x0F).wrapping_add(v & 0x0F).wrapping_add(c);
            if al > 9 {
                al = al.wrapping_add(6);
            };
            let mut ah =
                (self.A >> 4)
                    .wrapping_add(v >> 4)
                    .wrapping_add(if al > 15 { 1 } else { 0 });
            self.Z = (r & 0xFF) == 0;
            self.N = (ah & 8) != 0;
            self.V = (!(self.A ^ v) & (self.A ^ (ah << 4)) & 0x80) != 0;
            if ah > 9 {
                ah = ah.wrapping_add(6);
            };
            self.C = ah > 15;
            self.A = ((ah << 4) | (al & 15)) & 0xFF;
        } else {
            self.Z = (r & 0xFF) == 0;
            self.N = (r & 0x80) != 0;
            self.V = (!(self.A ^ v) & (self.A ^ r) & 0x80) != 0;
            self.C = (r as u16 & 0x100) != 0;
            self.A = r & 0xFF;
        }
    }

    fn ahx(&mut self) -> () {
        self.tmp = ((self.addr >> 8).wrapping_add(1)) & self.A as u16 & self.X as u16;
        self.write(self.addr, self.tmp as u8);
    }

    fn alr(&mut self) -> () {
        self.tmp = self.read16(self.addr) & self.A as u16;
        self.tmp = ((self.tmp & 1) << 8) | (self.tmp >> 1);
        self.fnzc(self.tmp);
        self.A = self.tmp as u8;
    }

    fn anc(&mut self) -> () {
        self.tmp = self.read16(self.addr);
        self.tmp |= ((self.tmp & 0x80) & (self.A as u16 & 0x80)) << 1;
        self.fnzc(self.tmp);
        self.A = self.tmp as u8;
    }

    fn _and(&mut self) -> () {
        self.A &= self.read(self.addr);
        self.fnz(self.A as u16);
    }

    fn ane(&mut self) -> () {
        self.tmp = self.read16(self.addr) & self.A as u16 & (self.A as u16 | 0xEE);
        self.fnz(self.tmp);
        self.A = self.tmp as u8;
    }

    fn arr(&mut self) -> () {
        self.tmp = self.read16(self.addr) & self.A as u16;
        self.C = (self.tmp & 0x80) != 0;
        self.V = (((self.tmp >> 7) & 1) ^ ((self.tmp >> 6) & 1)) != 0;
        if self.D {
            let mut al = (self.tmp & 0x0F) + (self.tmp & 1);
            if al > 5 {
                al = al.wrapping_add(6);
            };
            let ah = ((self.tmp >> 4) & 0x0F) + ((self.tmp >> 4) & 1);
            if ah > 5 {
                al = al.wrapping_add(6);
                self.C = true;
            } else {
                self.C = false;
            }
            self.tmp = (ah << 4) | al;
        }
        self.fnz(self.tmp);
        self.A = self.tmp as u8;
    }

    fn asl(&mut self) -> () {
        self.tmp = self.read16(self.addr) << 1;
        self.fnzc(self.tmp);
        self.tmp &= 0xFF;
    }
    fn asla(&mut self) -> () {
        self.tmp = (self.A as u16) << 1;
        self.fnzc(self.tmp);
        self.A = self.tmp as u8;
    }

    fn bit(&mut self) -> () {
        self.tmp = self.read16(self.addr);
        self.N = (self.tmp & 0x80) != 0;
        self.V = (self.tmp & 0x40) != 0;
        self.Z = (self.tmp & self.A as u16) == 0;
    }

    fn brk(&mut self) -> () {
        self.PC += 1;
        self.write(self.S as u16 + 0x100, (self.PC >> 8) as u8);
        self.S = self.S.wrapping_sub(1);
        self.write(self.S as u16 + 0x100, self.PC as u8);
        self.S = self.S.wrapping_sub(1);
        let mut v = if self.N { 1 << 7 } else { 0 };
        v |= if self.V { 1 << 6 } else { 0 };
        v |= 3 << 4;
        v |= if self.D { 1 << 3 } else { 0 };
        v |= if self.I { 1 << 2 } else { 0 };
        v |= if self.Z { 1 << 1 } else { 0 };
        v |= if self.C { 1 } else { 0 };
        self.write(self.S as u16 + 0x100, v);
        self.S = self.S.wrapping_sub(1);
        self.I = true;
        self.D = false;
        self.PC = (self.read16(0xFFFF) << 8) | self.read16(0xFFFE);
        self.cycles += 5;
    }

    fn bcc(&mut self) {
        self.branch(!self.C);
    }
    fn bcs(&mut self) {
        self.branch(self.C);
    }
    fn beq(&mut self) {
        self.branch(self.Z);
    }
    fn bne(&mut self) {
        self.branch(!self.Z);
    }
    fn bmi(&mut self) {
        self.branch(self.N);
    }
    fn bpl(&mut self) {
        self.branch(!self.N);
    }
    fn bvc(&mut self) {
        self.branch(!self.V);
    }
    fn bvs(&mut self) {
        self.branch(self.V);
    }

    fn clc(&mut self) {
        self.C = false;
    }
    fn cld(&mut self) {
        self.D = false;
    }
    fn cli(&mut self) {
        self.I = false;
    }
    fn clv(&mut self) {
        self.V = false;
    }

    fn cmp(&mut self) {
        self.tmp = self.A.wrapping_sub(self.read(self.addr)) as u16;
        self.fnzb(self.tmp);
    }

    fn cpx(&mut self) {
        self.tmp = self.X.wrapping_sub(self.read(self.addr)) as u16;
        self.fnzb(self.tmp);
    }

    fn cpy(&mut self) {
        self.tmp = self.Y.wrapping_sub(self.read(self.addr)) as u16;
        self.fnzb(self.tmp);
    }

    fn dcp(&mut self) {
        self.tmp = (self.read(self.addr).wrapping_sub(1)) as u16 & 0xFF;
        self.tmp = (self.A as u16).wrapping_sub(self.tmp);
        self.fnzb(self.tmp);
    }

    fn dec(&mut self) {
        self.tmp = (self.read(self.addr).wrapping_sub(1)) as u16 & 0xFF;
        self.fnz(self.tmp);
    }

    fn dex(&mut self) {
        self.X = self.X.wrapping_sub(1);
        self.fnz(self.X as u16);
    }

    fn dey(&mut self) {
        self.Y = self.Y.wrapping_sub(1);
        self.fnz(self.Y as u16);
    }

    fn eor(&mut self) {
        self.A ^= self.read(self.addr);
        self.fnz(self.A as u16);
    }

    fn inc(&mut self) {
        self.tmp = (self.read16(self.addr).wrapping_add(1)) & 0xFF;
        self.fnz(self.tmp);
    }

    fn inx(&mut self) {
        self.X = self.X.wrapping_add(1);
        self.fnz(self.X as u16);
    }

    fn iny(&mut self) {
        self.Y = self.Y.wrapping_add(1);
        self.fnz(self.Y as u16);
    }

    // TODO: CHECK THIS
    fn isc(&mut self) {
        let v: u16 = self.read16(self.addr) + 1;
        let c: u16 = 1 - (if self.C { 1 } else { 0 });
        let r: u16 = self.A as u16 - v - c;
        if self.D {
            let mut al = (self.A as u16 & 0x0F) - (v & 0x0F) - c;
            if al > 0x80 {
                al -= 6
            };
            let mut ah = (self.A as u16 >> 4) - (v >> 4) - (if al > 0x80 { 1 } else { 0 });
            self.Z = (r & 0xFF) == 0;
            self.N = (r & 0x80) != 0;
            self.V = ((self.A as u16 ^ v) & (self.A as u16 ^ r) & 0x80) != 0;
            self.C = if r & 0x100 != 0 { false } else { true };
            if ah > 0x80 {
                ah -= 6
            };
            self.A = ((ah << 4) | (al & 15)) as u8;
        } else {
            self.Z = (r & 0xFF) == 0;
            self.N = (r & 0x80) != 0;
            self.V = ((self.A as u16 ^ v) & (self.A as u16 ^ r) & 0x80) != 0;
            self.C = if (r & 0x100) != 0 { false } else { true };
            self.A = r as u8;
        }
    }

    fn jmp(&mut self) {
        self.PC = self.addr;
        self.cycles -= 1;
    }

    fn jsr(&mut self) {
        self.write(self.S as u16 + 0x100, ((self.PC - 1) >> 8) as u8);
        self.S = (self.S as u16 - 1) as u8;
        self.write(self.S as u16 + 0x100, (self.PC - 1) as u8);
        self.S = (self.S as u16 - 1) as u8;
        self.PC = self.addr;
        self.cycles += 2;
    }

    fn las(&mut self) {
        let t = self.read(self.addr) & self.S;
        self.S = t;
        self.X = t;
        self.A = t;
        self.fnz(self.A as u16);
    }

    fn lax(&mut self) {
        let t = self.read(self.addr);
        self.X = t;
        self.A = t;
        self.fnz(self.A as u16);
    }

    fn lda(&mut self) {
        self.A = self.read(self.addr);
        self.fnz(self.A as u16);
    }

    fn ldx(&mut self) {
        self.X = self.read(self.addr);
        self.fnz(self.X as u16);
    }

    fn ldy(&mut self) {
        self.Y = self.read(self.addr);
        self.fnz(self.Y as u16);
    }

    fn ora(&mut self) {
        self.A |= self.read(self.addr);
        self.fnz(self.A as u16);
    }

    fn rol(&mut self) {
        self.tmp = ((self.read16(self.addr)) << 1) | (if self.C { 1 } else { 0 });
        self.fnzc(self.tmp);
        self.tmp &= 0xFF;
    }
    fn rla(&mut self) {
        self.tmp = ((self.A as u16) << 1) | (if self.C { 1 } else { 0 });
        self.fnzc(self.tmp);
        self.A = self.tmp as u8;
    }

    fn ror(&mut self) {
        self.tmp = self.read16(self.addr);
        self.tmp = ((self.tmp & 1) << 8) | ((if self.C { 1 } else { 0 }) << 7) | (self.tmp >> 1);
        self.fnzc(self.tmp);
        self.tmp &= 0xFF;
    }
    fn rra(&mut self) {
        self.tmp =
            ((self.A as u16 & 1) << 8) | ((if self.C { 1 } else { 0 }) << 7) | (self.A as u16 >> 1);
        self.fnzc(self.tmp);
        self.A = self.tmp as u8;
    }

    fn kil(&self) {}

    fn lsr(&mut self) {
        self.tmp = self.read16(self.addr);
        self.tmp = ((self.tmp & 1) << 8) | (self.tmp >> 1);
        self.fnzc(self.tmp);
        self.tmp &= 0xFF;
    }
    fn lsra(&mut self) {
        self.tmp = ((self.A as u16 & 1) << 8) | (self.A as u16 >> 1);
        self.fnzc(self.tmp);
        self.A = self.tmp as u8;
    }

    fn nop(&mut self) {}

    fn pha(&mut self) {
        self.write(self.S as u16 + 0x100, self.A);
        self.S = self.S.wrapping_sub(1);
        self.cycles += 1;
    }

    fn php(&mut self) {
        let mut v = if self.N { 1 << 7 } else { 0 };
        v |= if self.V { 1 << 6 } else { 0 };
        v |= 3 << 4;
        v |= if self.D { 1 << 3 } else { 0 };
        v |= if self.I { 1 << 2 } else { 0 };
        v |= if self.Z { 1 << 1 } else { 0 };
        v |= if self.C { 1 } else { 0 };
        self.write(self.S as u16 + 0x100, v);
        self.S = self.S.wrapping_sub(1);;
        self.cycles += 1;
    }

    fn pla(&mut self) {
        self.S = self.S.wrapping_add(1);
        self.A = self.read(self.S as u16 + 0x100);
        self.fnz(self.A as u16);
        self.cycles += 2;
    }

    fn plp(&mut self) {
        self.S = self.S.wrapping_add(1);;
        self.tmp = self.read16(self.S as u16 + 0x100);
        self.N = (self.tmp & 0x80) != 0;
        self.V = (self.tmp & 0x40) != 0;
        self.D = (self.tmp & 0x08) != 0;
        self.I = (self.tmp & 0x04) != 0;
        self.Z = (self.tmp & 0x02) != 0;
        self.C = (self.tmp & 0x01) != 0;
        self.cycles += 2;
    }

    fn rti(&mut self) {
        self.S = self.S.wrapping_add(1);
        self.tmp = self.read16(self.S as u16 + 0x100);
        self.N = (self.tmp & 0x80) != 0;
        self.V = (self.tmp & 0x40) != 0;
        self.D = (self.tmp & 0x08) != 0;
        self.I = (self.tmp & 0x04) != 0;
        self.Z = (self.tmp & 0x02) != 0;
        self.C = (self.tmp & 0x01) != 0;
        self.S = self.S.wrapping_add(1);
        self.PC = self.read16(self.S as u16 + 0x100);
        self.S = self.S.wrapping_add(1);
        self.PC |= self.read16(self.S as u16 + 0x100) << 8;
        self.cycles += 4;
    }

    fn rts(&mut self) {
        self.S = self.S.wrapping_add(1);
        self.PC = self.read16(self.S as u16 + 0x100);
        self.S = self.S.wrapping_add(1);
        self.PC |= self.read16(self.S as u16 + 0x100) << 8;
        self.PC += 1;
        self.cycles += 4;
    }

    fn sax(&mut self) {
        self.write(self.addr, self.A & self.X);
    }

    fn sbc(&mut self) {
        let v: u16 = self.read16(self.addr);
        let c: u16 = 1 - (if self.C { 1 } else { 0 });
        let r: u16 = (self.A as u16).wrapping_sub(v).wrapping_sub(c);
        if self.D {
            let mut al = (self.A as u16 & 0x0F) - (v & 0x0F) - c;
            if al > 0x80 {
                al -= 6
            };
            let mut ah = (self.A as u16 >> 4) - (v >> 4) - (if al > 0x80 { 1 } else { 0 });
            self.Z = (r & 0xFF) == 0;
            self.N = (r & 0x80) != 0;
            self.V = ((self.A as u16 ^ v) & (self.A as u16 ^ r) & 0x80) != 0;
            self.C = if (r & 0x100) != 0 { false } else { true };
            if ah > 0x80 {
                ah -= 6
            };
            self.A = ((ah << 4) | (al & 15)) as u8;
        } else {
            self.Z = (r & 0xFF) == 0;
            self.N = (r & 0x80) != 0;
            self.V = ((self.A as u16 ^ v) & (self.A as u16 ^ r) & 0x80) != 0;
            self.C = if (r & 0x100) != 0 { false } else { true };
            self.A = r as u8;
        }
    }

    fn sbx(&mut self) {
        self.tmp = self
            .read16(self.addr)
            .wrapping_sub((self.A & self.X) as u16);
        self.fnzb(self.tmp);
        self.X = self.tmp as u8;
    }

    fn sec(&mut self) {
        self.C = true;
    }
    fn sed(&mut self) {
        self.D = true;
    }
    fn sei(&mut self) {
        self.I = true;
    }

    fn shs(&mut self) {
        self.tmp = ((self.addr >> 8) + 1) & self.A as u16 & self.X as u16;
        self.write(self.addr, self.tmp as u8);
        self.S = self.tmp as u8;
    }

    fn shx(&mut self) {
        self.tmp = ((self.addr >> 8) + 1) & self.X as u16;
        self.write(self.addr, self.tmp as u8);
    }

    fn shy(&mut self) {
        self.tmp = ((self.addr >> 8) + 1) & self.Y as u16;
        self.write(self.addr, self.tmp as u8);
    }

    fn slo(&mut self) {
        self.tmp = self.read16(self.addr) << 1;
        self.tmp |= self.A as u16;
        self.fnzc(self.tmp);
        self.A = self.tmp as u8;
    }

    fn sre(&mut self) {
        let v = self.read16(self.addr);
        self.tmp = ((v & 1) << 8) | (v >> 1);
        self.tmp ^= self.A as u16;
        self.fnzc(self.tmp);
        self.A = self.tmp as u8;
    }

    fn sta(&mut self) {
        self.write(self.addr, self.A);
    }

    fn stx(&mut self) {
        self.write(self.addr, self.X);
    }

    fn sty(&mut self) {
        self.write(self.addr, self.Y);
    }

    fn tax(&mut self) {
        self.X = self.A;
        self.fnz(self.X as u16);
    }

    fn tay(&mut self) {
        self.Y = self.A;
        self.fnz(self.Y as u16);
    }

    fn tsx(&mut self) {
        self.X = self.S;
        self.fnz(self.X as u16);
    }

    fn txa(&mut self) {
        self.A = self.X;
        self.fnz(self.A as u16);
    }

    fn txs(&mut self) {
        self.S = self.X;
    }

    fn tya(&mut self) {
        self.A = self.Y;
        self.fnz(self.A as u16);
    }

    // CPU FUNCTIONS

    pub fn reset(&mut self) -> () {
        self.A = 0;
        self.X = 0;
        self.Y = 0;
        self.S = 0;
        self.N = false;
        self.Z = true;
        self.C = false;
        self.V = false;
        self.I = false;
        self.D = false;
        self.opcode = 0x4C;

        self.PC = u16::from_be_bytes([self.read(0xfffd), self.read(0xfffc)]);
    }

    fn read(&mut self, address: u16) -> u8 {
        println!("READ ADDR::: {:X}", address);
        return self.address_spaces.read(address);
    }

    fn read16(&mut self, address: u16) -> u16 {
        self.read(address) as u16
    }

    fn write(&mut self, address: u16, value: u8) -> () {
        self.address_spaces.write(address, value);
    }

    pub fn exec_op(&mut self, opcode: u8) {
        match opcode {
            /*  BRK     */
            0x00 => {
                self.imp();
                self.brk();
            }
            /*  ORA izx */
            0x01 => {
                self.izx();
                self.ora();
            }
            /* *KIL     */
            0x02 => {
                self.imp();
                self.kil();
            }
            /* *SLO izx */
            0x03 => {
                self.izx();
                self.slo();
                self.rmw();
            }
            /* *NOP zp  */
            0x04 => {
                self.zp();
                self.nop();
            }
            /*  ORA zp  */
            0x05 => {
                self.zp();
                self.ora();
            }
            /*  ASL zp  */
            0x06 => {
                self.zp();
                self.asl();
                self.rmw();
            }
            /* *SLO zp  */
            0x07 => {
                self.zp();
                self.slo();
                self.rmw();
            }
            /*  PHP     */
            0x08 => {
                self.imp();
                self.php();
            }
            /*  ORA imm */
            0x09 => {
                self.imm();
                self.ora();
            }
            /*  ASL     */
            0x0A => {
                self.imp();
                self.asla();
            }
            /* *ANC imm */
            0x0B => {
                self.imm();
                self.anc();
            }
            /* *NOP abs */
            0x0C => {
                self.abs();
                self.nop();
            }
            /*  ORA abs */
            0x0D => {
                self.abs();
                self.ora();
            }
            /*  ASL abs */
            0x0E => {
                self.abs();
                self.asl();
                self.rmw();
            }
            /* *SLO abs */
            0x0F => {
                self.abs();
                self.slo();
                self.rmw();
            }

            /*  BPL rel */
            0x10 => {
                self.rel();
                self.bpl();
            }
            /*  ORA izy */
            0x11 => {
                self.izy();
                self.ora();
            }
            /* *KIL     */
            0x12 => {
                self.imp();
                self.kil();
            }
            /* *SLO izy */
            0x13 => {
                self.izy();
                self.slo();
                self.rmw();
            }
            /* *NOP zpx */
            0x14 => {
                self.zpx();
                self.nop();
            }
            /*  ORA zpx */
            0x15 => {
                self.zpx();
                self.ora();
            }
            /*  ASL zpx */
            0x16 => {
                self.zpx();
                self.asl();
                self.rmw();
            }
            /* *SLO zpx */
            0x17 => {
                self.zpx();
                self.slo();
                self.rmw();
            }
            /*  CLC     */
            0x18 => {
                self.imp();
                self.clc();
            }
            /*  ORA aby */
            0x19 => {
                self.aby();
                self.ora();
            }
            /* *NOP     */
            0x1A => {
                self.imp();
                self.nop();
            }
            /* *SLO aby */
            0x1B => {
                self.aby();
                self.slo();
                self.rmw();
            }
            /* *NOP abx */
            0x1C => {
                self.abx();
                self.nop();
            }
            /*  ORA abx */
            0x1D => {
                self.abx();
                self.ora();
            }
            /*  ASL abx */
            0x1E => {
                self.abx();
                self.asl();
                self.rmw();
            }
            /* *SLO abx */
            0x1F => {
                self.abx();
                self.slo();
                self.rmw();
            }

            /*  JSR abs */
            0x20 => {
                self.abs();
                self.jsr();
            }
            /*  AND izx */
            0x21 => {
                self.izx();
                self._and();
            }
            /* *KIL     */
            0x22 => {
                self.imp();
                self.kil();
            }
            /* *RLA izx */
            0x23 => {
                self.izx();
                self.rla();
                self.rmw();
            }
            /*  BIT zp  */
            0x24 => {
                self.zp();
                self.bit();
            }
            /*  AND zp  */
            0x25 => {
                self.zp();
                self._and();
            }
            /*  ROL zp  */
            0x26 => {
                self.zp();
                self.rol();
                self.rmw();
            }
            /* *RLA zp  */
            0x27 => {
                self.zp();
                self.rla();
                self.rmw();
            }
            /*  PLP     */
            0x28 => {
                self.imp();
                self.plp();
            }
            /*  AND imm */
            0x29 => {
                self.imm();
                self._and();
            }
            /*  ROL     */
            0x2A => {
                self.imp();
                self.rla();
            }
            /* *ANC imm */
            0x2B => {
                self.imm();
                self.anc();
            }
            /*  BIT abs */
            0x2C => {
                self.abs();
                self.bit();
            }
            /*  AND abs */
            0x2D => {
                self.abs();
                self._and();
            }
            /*  ROL abs */
            0x2E => {
                self.abs();
                self.rol();
                self.rmw();
            }
            /* *RLA abs */
            0x2F => {
                self.abs();
                self.rla();
                self.rmw();
            }

            /*  BMI rel */
            0x30 => {
                self.rel();
                self.bmi();
            }
            /*  AND izy */
            0x31 => {
                self.izy();
                self._and();
            }
            /* *KIL     */
            0x32 => {
                self.imp();
                self.kil();
            }
            /* *RLA izy */
            0x33 => {
                self.izy();
                self.rla();
                self.rmw();
            }
            /* *NOP zpx */
            0x34 => {
                self.zpx();
                self.nop();
            }
            /*  AND zpx */
            0x35 => {
                self.zpx();
                self._and();
            }
            /*  ROL zpx */
            0x36 => {
                self.zpx();
                self.rol();
                self.rmw();
            }
            /* *RLA zpx */
            0x37 => {
                self.zpx();
                self.rla();
                self.rmw();
            }
            /*  SEC     */
            0x38 => {
                self.imp();
                self.sec();
            }
            /*  AND aby */
            0x39 => {
                self.aby();
                self._and();
            }
            /* *NOP     */
            0x3A => {
                self.imp();
                self.nop();
            }
            /* *RLA aby */
            0x3B => {
                self.aby();
                self.rla();
                self.rmw();
            }
            /* *NOP abx */
            0x3C => {
                self.abx();
                self.nop();
            }
            /*  AND abx */
            0x3D => {
                self.abx();
                self._and();
            }
            /*  ROL abx */
            0x3E => {
                self.abx();
                self.rol();
                self.rmw();
            }
            /* *RLA abx */
            0x3F => {
                self.abx();
                self.rla();
                self.rmw();
            }

            /*  RTI     */
            0x40 => {
                self.imp();
                self.rti();
            }
            /*  EOR izx */
            0x41 => {
                self.izx();
                self.eor();
            }
            /* *KIL     */
            0x42 => {
                self.imp();
                self.kil();
            }
            /* *SRE izx */
            0x43 => {
                self.izx();
                self.sre();
                self.rmw();
            }
            /* *NOP zp  */
            0x44 => {
                self.zp();
                self.nop();
            }
            /*  EOR zp  */
            0x45 => {
                self.zp();
                self.eor();
            }
            /*  LSR zp  */
            0x46 => {
                self.zp();
                self.lsr();
                self.rmw();
            }
            /* *SRE zp  */
            0x47 => {
                self.zp();
                self.sre();
                self.rmw();
            }
            /*  PHA     */
            0x48 => {
                self.imp();
                self.pha();
            }
            /*  EOR imm */
            0x49 => {
                self.imm();
                self.eor();
            }
            /*  LSR     */
            0x4A => {
                self.imp();
                self.lsra();
            }
            /* *ALR imm */
            0x4B => {
                self.imm();
                self.alr();
            }
            /*  JMP abs */
            0x4C => {
                self.abs();
                self.jmp();
            }
            /*  EOR abs */
            0x4D => {
                self.abs();
                self.eor();
            }
            /*  LSR abs */
            0x4E => {
                self.abs();
                self.lsr();
                self.rmw();
            }
            /* *SRE abs */
            0x4F => {
                self.abs();
                self.sre();
                self.rmw();
            }

            /*  BVC rel */
            0x50 => {
                self.rel();
                self.bvc();
            }
            /*  EOR izy */
            0x51 => {
                self.izy();
                self.eor();
            }
            /* *KIL     */
            0x52 => {
                self.imp();
                self.kil();
            }
            /* *SRE izy */
            0x53 => {
                self.izy();
                self.sre();
                self.rmw();
            }
            /* *NOP zpx */
            0x54 => {
                self.zpx();
                self.nop();
            }
            /*  EOR zpx */
            0x55 => {
                self.zpx();
                self.eor();
            }
            /*  LSR zpx */
            0x56 => {
                self.zpx();
                self.lsr();
                self.rmw();
            }
            /* *SRE zpx */
            0x57 => {
                self.zpx();
                self.sre();
                self.rmw();
            }
            /*  CLI     */
            0x58 => {
                self.imp();
                self.cli();
            }
            /*  EOR aby */
            0x59 => {
                self.aby();
                self.eor();
            }
            /* *NOP     */
            0x5A => {
                self.imp();
                self.nop();
            }
            /* *SRE aby */
            0x5B => {
                self.aby();
                self.sre();
                self.rmw();
            }
            /* *NOP abx */
            0x5C => {
                self.abx();
                self.nop();
            }
            /*  EOR abx */
            0x5D => {
                self.abx();
                self.eor();
            }
            /*  LSR abx */
            0x5E => {
                self.abx();
                self.lsr();
                self.rmw();
            }
            /* *SRE abx */
            0x5F => {
                self.abx();
                self.sre();
                self.rmw();
            }

            /*  RTS     */
            0x60 => {
                self.imp();
                self.rts();
            }
            /*  ADC izx */
            0x61 => {
                self.izx();
                self.adc();
            }
            /* *KIL     */
            0x62 => {
                self.imp();
                self.kil();
            }
            /* *RRA izx */
            0x63 => {
                self.izx();
                self.rra();
                self.rmw();
            }
            /* *NOP zp  */
            0x64 => {
                self.zp();
                self.nop();
            }
            /*  ADC zp  */
            0x65 => {
                self.zp();
                self.adc();
            }
            /*  ROR zp  */
            0x66 => {
                self.zp();
                self.ror();
                self.rmw();
            }
            /* *RRA zp  */
            0x67 => {
                self.zp();
                self.rra();
                self.rmw();
            }
            /*  PLA     */
            0x68 => {
                self.imp();
                self.pla();
            }
            /*  ADC imm */
            0x69 => {
                self.imm();
                self.adc();
            }
            /*  ROR     */
            0x6A => {
                self.imp();
                self.rra();
            }
            /* *ARR imm */
            0x6B => {
                self.imm();
                self.arr();
            }
            /*  JMP ind */
            0x6C => {
                self.ind();
                self.jmp();
            }
            /*  ADC abs */
            0x6D => {
                self.abs();
                self.adc();
            }
            /*  ROR abs */
            0x6E => {
                self.abs();
                self.ror();
                self.rmw();
            }
            /* *RRA abs */
            0x6F => {
                self.abs();
                self.rra();
                self.rmw();
            }

            /*  BVS rel */
            0x70 => {
                self.rel();
                self.bvs();
            }
            /*  ADC izy */
            0x71 => {
                self.izy();
                self.adc();
            }
            /* *KIL     */
            0x72 => {
                self.imp();
                self.kil();
            }
            /* *RRA izy */
            0x73 => {
                self.izy();
                self.rra();
                self.rmw();
            }
            /* *NOP zpx */
            0x74 => {
                self.zpx();
                self.nop();
            }
            /*  ADC zpx */
            0x75 => {
                self.zpx();
                self.adc();
            }
            /*  ROR zpx */
            0x76 => {
                self.zpx();
                self.ror();
                self.rmw();
            }
            /* *RRA zpx */
            0x77 => {
                self.zpx();
                self.rra();
                self.rmw();
            }
            /*  SEI     */
            0x78 => {
                self.imp();
                self.sei();
            }
            /*  ADC aby */
            0x79 => {
                self.aby();
                self.adc();
            }
            /* *NOP     */
            0x7A => {
                self.imp();
                self.nop();
            }
            /* *RRA aby */
            0x7B => {
                self.aby();
                self.rra();
                self.rmw();
            }
            /* *NOP abx */
            0x7C => {
                self.abx();
                self.nop();
            }
            /*  ADC abx */
            0x7D => {
                self.abx();
                self.adc();
            }
            /*  ROR abx */
            0x7E => {
                self.abx();
                self.ror();
                self.rmw();
            }
            /* *RRA abx */
            0x7F => {
                self.abx();
                self.rra();
                self.rmw();
            }

            /* *NOP imm */
            0x80 => {
                self.imm();
                self.nop();
            }
            /*  STA izx */
            0x81 => {
                self.izx();
                self.sta();
            }
            /* *NOP imm */
            0x82 => {
                self.imm();
                self.nop();
            }
            /* *SAX izx */
            0x83 => {
                self.izx();
                self.sax();
            }
            /*  STY zp  */
            0x84 => {
                self.zp();
                self.sty();
            }
            /*  STA zp  */
            0x85 => {
                self.zp();
                self.sta();
            }
            /*  STX zp  */
            0x86 => {
                self.zp();
                self.stx();
            }
            /* *SAX zp  */
            0x87 => {
                self.zp();
                self.sax();
            }
            /*  DEY     */
            0x88 => {
                self.imp();
                self.dey();
            }
            /* *NOP imm */
            0x89 => {
                self.imm();
                self.nop();
            }
            /*  TXA     */
            0x8A => {
                self.imp();
                self.txa();
            }
            /* *ANE imm */
            0x8B => {
                self.imm();
                self.ane();
            }
            /*  STY abs */
            0x8C => {
                self.abs();
                self.sty();
            }
            /*  STA abs */
            0x8D => {
                self.abs();
                self.sta();
            }
            /*  STX abs */
            0x8E => {
                self.abs();
                self.stx();
            }
            /* *SAX abs */
            0x8F => {
                self.abs();
                self.sax();
            }

            /*  BCC rel */
            0x90 => {
                self.rel();
                self.bcc();
            }
            /*  STA izy */
            0x91 => {
                self.izy();
                self.sta();
            }
            /* *KIL     */
            0x92 => {
                self.imp();
                self.kil();
            }
            /* *AHX izy */
            0x93 => {
                self.izy();
                self.ahx();
            }
            /*  STY zpx */
            0x94 => {
                self.zpx();
                self.sty();
            }
            /*  STA zpx */
            0x95 => {
                self.zpx();
                self.sta();
            }
            /*  STX zpy */
            0x96 => {
                self.zpy();
                self.stx();
            }
            /* *SAX zpy */
            0x97 => {
                self.zpy();
                self.sax();
            }
            /*  TYA     */
            0x98 => {
                self.imp();
                self.tya();
            }
            /*  STA aby */
            0x99 => {
                self.aby();
                self.sta();
            }
            /*  TXS     */
            0x9A => {
                self.imp();
                self.txs();
            }
            /* *SHS aby */
            0x9B => {
                self.aby();
                self.shs();
            }
            /* *SHY abx */
            0x9C => {
                self.abx();
                self.shy();
            }
            /*  STA abx */
            0x9D => {
                self.abx();
                self.sta();
            }
            /* *SHX aby */
            0x9E => {
                self.aby();
                self.shx();
            }
            /* *AHX aby */
            0x9F => {
                self.aby();
                self.ahx();
            }

            /*  LDY imm */
            0xA0 => {
                self.imm();
                self.ldy();
            }
            /*  LDA izx */
            0xA1 => {
                self.izx();
                self.lda();
            }
            /*  LDX imm */
            0xA2 => {
                self.imm();
                self.ldx();
            }
            /* *LAX izx */
            0xA3 => {
                self.izx();
                self.lax();
            }
            /*  LDY zp  */
            0xA4 => {
                self.zp();
                self.ldy();
            }
            /*  LDA zp  */
            0xA5 => {
                self.zp();
                self.lda();
            }
            /*  LDX zp  */
            0xA6 => {
                self.zp();
                self.ldx();
            }
            /* *LAX zp  */
            0xA7 => {
                self.zp();
                self.lax();
            }
            /*  TAY     */
            0xA8 => {
                self.imp();
                self.tay();
            }
            /*  LDA imm */
            0xA9 => {
                self.imm();
                self.lda();
            }
            /*  TAX     */
            0xAA => {
                self.imp();
                self.tax();
            }
            /* *LAX imm */
            0xAB => {
                self.imm();
                self.lax();
            }
            /*  LDY abs */
            0xAC => {
                self.abs();
                self.ldy();
            }
            /*  LDA abs */
            0xAD => {
                self.abs();
                self.lda();
            }
            /*  LDX abs */
            0xAE => {
                self.abs();
                self.ldx();
            }
            /* *LAX abs */
            0xAF => {
                self.abs();
                self.lax();
            }

            /*  BCS rel */
            0xB0 => {
                self.rel();
                self.bcs();
            }
            /*  LDA izy */
            0xB1 => {
                self.izy();
                self.lda();
            }
            /* *KIL     */
            0xB2 => {
                self.imp();
                self.kil();
            }
            /* *LAX izy */
            0xB3 => {
                self.izy();
                self.lax();
            }
            /*  LDY zpx */
            0xB4 => {
                self.zpx();
                self.ldy();
            }
            /*  LDA zpx */
            0xB5 => {
                self.zpx();
                self.lda();
            }
            /*  LDX zpy */
            0xB6 => {
                self.zpy();
                self.ldx();
            }
            /* *LAX zpy */
            0xB7 => {
                self.zpy();
                self.lax();
            }
            /*  CLV     */
            0xB8 => {
                self.imp();
                self.clv();
            }
            /*  LDA aby */
            0xB9 => {
                self.aby();
                self.lda();
            }
            /*  TSX     */
            0xBA => {
                self.imp();
                self.tsx();
            }
            /* *LAS aby */
            0xBB => {
                self.aby();
                self.las();
            }
            /*  LDY abx */
            0xBC => {
                self.abx();
                self.ldy();
            }
            /*  LDA abx */
            0xBD => {
                self.abx();
                self.lda();
            }
            /*  LDX aby */
            0xBE => {
                self.aby();
                self.ldx();
            }
            /* *LAX aby */
            0xBF => {
                self.aby();
                self.lax();
            }

            /*  CPY imm */
            0xC0 => {
                self.imm();
                self.cpy();
            }
            /*  CMP izx */
            0xC1 => {
                self.izx();
                self.cmp();
            }
            /* *NOP imm */
            0xC2 => {
                self.imm();
                self.nop();
            }
            /* *DCP izx */
            0xC3 => {
                self.izx();
                self.dcp();
                self.rmw();
            }
            /*  CPY zp  */
            0xC4 => {
                self.zp();
                self.cpy();
            }
            /*  CMP zp  */
            0xC5 => {
                self.zp();
                self.cmp();
            }
            /*  DEC zp  */
            0xC6 => {
                self.zp();
                self.dec();
                self.rmw();
            }
            /* *DCP zp  */
            0xC7 => {
                self.zp();
                self.dcp();
                self.rmw();
            }
            /*  INY     */
            0xC8 => {
                self.imp();
                self.iny();
            }
            /*  CMP imm */
            0xC9 => {
                self.imm();
                self.cmp();
            }
            /*  DEX     */
            0xCA => {
                self.imp();
                self.dex();
            }
            /* *SBX imm */
            0xCB => {
                self.imm();
                self.sbx();
            }
            /*  CPY abs */
            0xCC => {
                self.abs();
                self.cpy();
            }
            /*  CMP abs */
            0xCD => {
                self.abs();
                self.cmp();
            }
            /*  DEC abs */
            0xCE => {
                self.abs();
                self.dec();
                self.rmw();
            }
            /* *DCP abs */
            0xCF => {
                self.abs();
                self.dcp();
                self.rmw();
            }

            /*  BNE rel */
            0xD0 => {
                self.rel();
                self.bne();
            }
            /*  CMP izy */
            0xD1 => {
                self.izy();
                self.cmp();
            }
            /* *KIL     */
            0xD2 => {
                self.imp();
                self.kil();
            }
            /* *DCP izy */
            0xD3 => {
                self.izy();
                self.dcp();
                self.rmw();
            }
            /* *NOP zpx */
            0xD4 => {
                self.zpx();
                self.nop();
            }
            /*  CMP zpx */
            0xD5 => {
                self.zpx();
                self.cmp();
            }
            /*  DEC zpx */
            0xD6 => {
                self.zpx();
                self.dec();
                self.rmw();
            }
            /* *DCP zpx */
            0xD7 => {
                self.zpx();
                self.dcp();
                self.rmw();
            }
            /*  CLD     */
            0xD8 => {
                self.imp();
                self.cld();
            }
            /*  CMP aby */
            0xD9 => {
                self.aby();
                self.cmp();
            }
            /* *NOP     */
            0xDA => {
                self.imp();
                self.nop();
            }
            /* *DCP aby */
            0xDB => {
                self.aby();
                self.dcp();
                self.rmw();
            }
            /* *NOP abx */
            0xDC => {
                self.abx();
                self.nop();
            }
            /*  CMP abx */
            0xDD => {
                self.abx();
                self.cmp();
            }
            /*  DEC abx */
            0xDE => {
                self.abx();
                self.dec();
                self.rmw();
            }
            /* *DCP abx */
            0xDF => {
                self.abx();
                self.dcp();
                self.rmw();
            }

            /*  CPX imm */
            0xE0 => {
                self.imm();
                self.cpx();
            }
            /*  SBC izx */
            0xE1 => {
                self.izx();
                self.sbc();
            }
            /* *NOP imm */
            0xE2 => {
                self.imm();
                self.nop();
            }
            /* *ISC izx */
            0xE3 => {
                self.izx();
                self.isc();
                self.rmw();
            }
            /*  CPX zp  */
            0xE4 => {
                self.zp();
                self.cpx();
            }
            /*  SBC zp  */
            0xE5 => {
                self.zp();
                self.sbc();
            }
            /*  INC zp  */
            0xE6 => {
                self.zp();
                self.inc();
                self.rmw();
            }
            /* *ISC zp  */
            0xE7 => {
                self.zp();
                self.isc();
                self.rmw();
            }
            /*  INX     */
            0xE8 => {
                self.imp();
                self.inx();
            }
            /*  SBC imm */
            0xE9 => {
                self.imm();
                self.sbc();
            }
            /*  NOP     */
            0xEA => {
                self.imp();
                self.nop();
            }
            /* *SBC imm */
            0xEB => {
                self.imm();
                self.sbc();
            }
            /*  CPX abs */
            0xEC => {
                self.abs();
                self.cpx();
            }
            /*  SBC abs */
            0xED => {
                self.abs();
                self.sbc();
            }
            /*  INC abs */
            0xEE => {
                self.abs();
                self.inc();
                self.rmw();
            }
            /* *ISC abs */
            0xEF => {
                self.abs();
                self.isc();
                self.rmw();
            }

            /*  BEQ rel */
            0xF0 => {
                self.rel();
                self.beq();
            }
            /*  SBC izy */
            0xF1 => {
                self.izy();
                self.sbc();
            }
            /* *KIL     */
            0xF2 => {
                self.imp();
                self.kil();
            }
            /* *ISC izy */
            0xF3 => {
                self.izy();
                self.isc();
                self.rmw();
            }
            /* *NOP zpx */
            0xF4 => {
                self.zpx();
                self.nop();
            }
            /*  SBC zpx */
            0xF5 => {
                self.zpx();
                self.sbc();
            }
            /*  INC zpx */
            0xF6 => {
                self.zpx();
                self.inc();
                self.rmw();
            }
            /* *ISC zpx */
            0xF7 => {
                self.zpx();
                self.isc();
                self.rmw();
            }
            /*  SED     */
            0xF8 => {
                self.imp();
                self.sed();
            }
            /*  SBC aby */
            0xF9 => {
                self.aby();
                self.sbc();
            }
            /* *NOP     */
            0xFA => {
                self.imp();
                self.nop();
            }
            /* *ISC aby */
            0xFB => {
                self.aby();
                self.isc();
                self.rmw();
            }
            /* *NOP abx */
            0xFC => {
                self.abx();
                self.nop();
            }
            /*  SBC abx */
            0xFD => {
                self.abx();
                self.sbc();
            }
            /*  INC abx */
            0xFE => {
                self.abx();
                self.inc();
                self.rmw();
            }
            /* *ISC abx */
            0xFF => {
                self.abx();
                self.isc();
                self.rmw();
            }
        }
    }
}

impl Clockable for CPU6502 {
    fn get_cycles(&self) -> usize {
        return self.cycles;
    }

    fn step(&mut self) -> usize {
        let start_cycles = self.cycles;
        self.opcode = self.read(self.PC);
        self.PC += 1;
        println!("::::::::: {:X}", self.opcode);

        self.exec_op(self.opcode);
        return self.cycles - start_cycles;
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    fn build_base_map() -> std::vec::Vec<address_spaces::AddressMap> {
        let mut rom = Box::new(rom::Rom::init_with_size(0xffff - 0xff00));
        let mut ram = Box::new(ram::Ram::init_with_size(100));

        let the_mapping = vec![
            address_spaces::AddressMap {
                addr: [0, 100],
                component: ram,
                name: String::from("RAM"),
            },
            address_spaces::AddressMap {
                addr: [0xff00, 0xffff],
                component: rom,
                name: String::from("ROM"),
            },
        ];

        the_mapping
    }

    #[test]
    fn initial_state() {
        let the_mapping = build_base_map();
        let mut cpu = CPU6502::init(address_spaces::AddressSpaces::init(the_mapping));
        assert_eq!(0x0, cpu.get_cycles());
        let cycles = cpu.step(); // BRK
        assert_eq!(7, cycles);
        assert_eq!(0x0000, cpu.PC);
    }

    #[test]
    fn should_reset() {
        let mut the_mapping = build_base_map();
        let mut rom_data = vec![0x00; 2 + 0xFF];
        rom_data[2 + 0xfd] = 0x0A;
        rom_data[2 + 0xfc] = 0x0B;
        the_mapping[1].component.flash(&rom_data);

        //the_mapping[1]
        //    .component
        //    .flash(&vec![0x00, 0xFF, 0xEA, 0xEA, 0xEA, 0x4C, 02, 0xFF]);

        let mut cpu = CPU6502::init(address_spaces::AddressSpaces::init(the_mapping));

        cpu.reset();

        assert_eq!(0x0A0B, cpu.PC);
    }

    #[test]
    fn read_steps() {
        let mut the_mapping = build_base_map();
        let mut rom_data = vec![0x00; 2 + 0xFF];
        rom_data[2 + 0xfd] = 0xFF;
        rom_data[2 + 0xfc] = 0x00;
        the_mapping[1].component.flash(&rom_data);

        the_mapping[1]
            .component
            .flash(&vec![0x00, 0xFF, 0xEA, 0xEA, 0xEA, 0x4C, 02, 0xFF]);

        let mut cpu = CPU6502::init(address_spaces::AddressSpaces::init(the_mapping));

        println!("{:?}", cpu);

        // 1  0000 ????
        // 2  0000 ????						; TEST ADDRESSING MODES
        // 3  0000 ????						; $FFFC and $FFFD - RESET PROGRAM COUNTER
        // 4  0000 ????
        // 5  0000 ????				      processor	6502
        // 6  ff00					      org	$FF00
        // 7  ff00
        // 8  ff00				   loop
        // 9  ff00		       ea		      nop
        // 10  ff01		       ea		      nop
        // 11  ff02				   loop2
        // 12  ff02		       ea		      nop
        // 13  ff03		       4c 02 ff 	      jmp	loop2

        cpu.reset();
        assert_eq!(0xFF00, cpu.PC);
        assert_eq!(0, cpu.cycles);

        let step_res = cpu.step(); // nop
        assert_eq!(0xFF01, cpu.PC);
        assert_eq!(2, cpu.cycles);
        assert_eq!(2, step_res);

        let step_res = cpu.step(); // nop
        assert_eq!(0xFF02, cpu.PC);
        assert_eq!(4, cpu.cycles);
        assert_eq!(2, step_res);

        let step_res = cpu.step(); // nop
        assert_eq!(0xFF03, cpu.PC);
        assert_eq!(6, cpu.cycles);
        assert_eq!(2, step_res);

        let step_res = cpu.step(); // jmp
        assert_eq!(0xFF02, cpu.PC);
        assert_eq!(9, cpu.cycles);
        assert_eq!(3, step_res);
    }

}
