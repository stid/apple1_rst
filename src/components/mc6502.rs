use super::address_spaces::AddressSpaces;

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
        return self.address_spaces.read(address);
    }

    fn read16(&mut self, address: u16) -> u16 {
        self.read(address) as u16
    }

    fn write(&mut self, address: u16, value: u8) -> () {
        self.address_spaces.write(address, value);
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Subroutines - addressing modes & flags
    ////////////////////////////////////////////////////////////////////////////////

    fn izx(&mut self) {
        self.PC += 1;
        let a: u16 = (self.read16(self.PC) + self.X as u16) & 0xFF;
        self.addr = (self.read16(a + 1) << 8) | self.read16(a);
        self.cycles += 6;
    }

    fn izy(&mut self) {
        self.PC += 1;
        let a: u16 = self.read16(self.PC);
        let paddr: u16 = (self.read16((a + 1) & 0xFF) << 8) | self.read16(a);
        self.addr = paddr + self.Y as u16;
        if (paddr & 0x100) != (self.addr & 0x100) {
            self.cycles += 6;
        } else {
            self.cycles += 5;
        }
    }

    fn ind(&mut self) -> () {
        let mut a = self.read16(self.PC);
        a |= (self.read16((self.PC & 0xff00) | ((self.PC + 1) & 0xff))) << 8;
        self.addr = self.read16(a);
        self.addr |= (self.read16(a + 1)) << 8;
        self.cycles += 5;
    }

    fn zp(&mut self) -> () {
        self.PC += 1;
        self.addr = self.read16(self.PC);
        self.cycles += 3;
    }

    fn zpx(&mut self) -> () {
        self.PC += 1;
        self.addr = (self.read16(self.PC) + self.X as u16) & 0xff;
        self.cycles += 4;
    }

    fn zpy(&mut self) -> () {
        self.PC += 1;
        self.addr = (self.read16(self.PC) + self.Y as u16) & 0xff;
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
        self.PC += 1;
        self.addr = self.read16(self.PC);
        self.PC += 1;
        self.addr |= (self.read16(self.PC)) << 8;
        self.cycles += 4;
    }

    fn abx(&mut self) -> () {
        self.PC += 1;
        let mut paddr = self.read16(self.PC);
        self.PC += 1;
        paddr |= (self.read16(self.PC)) << 8;
        self.addr = paddr + (self.X as u16);
        if (paddr & 0x100) != (self.addr & 0x100) {
            self.cycles += 5;
        } else {
            self.cycles += 4;
        }
    }

    fn aby(&mut self) -> () {
        self.PC += 1;
        let mut paddr = self.read16(self.PC);
        self.PC += 1;
        paddr |= (self.read16(self.PC)) << 8;
        self.addr = paddr + (self.Y as u16);
        if (paddr & 0x100) != (self.addr & 0x100) {
            self.cycles += 5;
        } else {
            self.cycles += 4;
        }
    }

    fn rel(&mut self) -> () {
        self.PC += 1;
        self.addr = self.read16(self.PC);
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
        let r = self.A + v + c;
        if self.D {
            let mut al = (self.A & 0x0F) + (v & 0x0F) + c;
            if al > 9 {
                al += 6
            };
            let mut ah = (self.A >> 4) + (v >> 4) + (if al > 15 { 1 } else { 0 });
            self.Z = (r & 0xFF) == 0;
            self.N = (ah & 8) != 0;
            self.V = (!(self.A ^ v) & (self.A ^ (ah << 4)) & 0x80) != 0;
            if ah > 9 {
                ah += 6
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
        self.tmp = ((self.addr >> 8) + 1) & self.A as u16 & self.X as u16;
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
                al += 6
            };
            let ah = ((self.tmp >> 4) & 0x0F) + ((self.tmp >> 4) & 1);
            if ah > 5 {
                al += 6;
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
        self.S = (self.S - 1) & 0xFF;
        self.write(self.S as u16 + 0x100, self.PC as u8);
        self.S = (self.S - 1) & 0xFF;
        let mut v = if self.N { 1 << 7 } else { 0 };
        v |= if self.V { 1 << 6 } else { 0 };
        v |= 3 << 4;
        v |= if self.D { 1 << 3 } else { 0 };
        v |= if self.I { 1 << 2 } else { 0 };
        v |= if self.Z { 1 << 1 } else { 0 };
        v |= if self.C { 1 } else { 0 };
        self.write(self.S as u16 + 0x100, v);
        self.S = (self.S - 1) & 0xFF;
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

    fn cmp(&self) {
        self.tmp = self.A as u16 - self.read16(self.addr);
        self.fnzb(self.tmp);
    }

    fn cpx(&self) {
        self.tmp = self.X as u16 - self.read16(self.addr);
        self.fnzb(self.tmp);
    }

    fn cpy(&self) {
        self.tmp = self.Y as u16 - self.read16(self.addr);
        self.fnzb(self.tmp);
    }

    fn dcp(&self) {
        self.tmp = (self.read16(self.addr) - 1) & 0xFF;
        self.tmp = self.A as u16 - self.tmp;
        self.fnzb(self.tmp);
    }

    fn dec(&self) {
        self.tmp = (self.read16(self.addr) - 1) & 0xFF;
        self.fnz(self.tmp);
    }

    fn dex(&self) {
        self.X = (self.X - 1) & 0xFF;
        self.fnz(self.X as u16);
    }

    fn dey(&self) {
        self.Y = (self.Y as u16 - 1) as u8;
        self.fnz(self.Y as u16);
    }

    fn eor(&self) {
        self.A ^= self.read(self.addr);
        self.fnz(self.A as u16);
    }

    fn inc(&self) {
        self.tmp = (self.read16(self.addr) + 1) & 0xFF;
        self.fnz(self.tmp);
    }

    fn inx(&self) {
        self.X = (self.X as u16 + 1) as u8;
        self.fnz(self.X as u16);
    }

    fn iny(&self) {
        self.Y = (self.Y as u16 + 1) as u8;
        self.fnz(self.Y as u16);
    }


    // TODO: CHECK THIS
    fn isc(&self) {
        let v: u16 = self.read16(self.addr) + 1;
        let c: u16 = 1 - (if self.C  {1} else {0});
        let r: u16 = self.A as u16 - v - c;
        if self.D {
            let mut al = (self.A as u16 & 0x0F) - (v & 0x0F) - c;
            if al > 0x80 {al -= 6};
            let mut ah = (self.A as u16 >> 4) - (v >> 4) - (if al > 0x80 {1} else {0});
            self.Z = (r & 0xFF) == 0;
            self.N = (r & 0x80) != 0;
            self.V = ((self.A as u16 ^ v) & (self.A as u16 ^ r) & 0x80) != 0;
            self.C = if r & 0x100 != 0 {false} else {true};
            if ah > 0x80 {ah -= 6};
            self.A = ((ah << 4) | (al & 15)) as u8;
        } else {
            self.Z = (r & 0xFF) == 0;
            self.N = (r & 0x80) != 0;
            self.V = ((self.A as u16 ^ v) & (self.A as u16 ^ r) & 0x80) != 0;
            self.C = if (r & 0x100) != 0 {false} else {true};
            self.A = r as u8;
        }
    }


    fn jmp(&self) {
        self.PC = self.addr;
        self.cycles -= 1;
    }

    fn jsr(&self) {
        self.write(self.S as u16 + 0x100, ((self.PC - 1) >> 8) as u8);
        self.S = (self.S as u16 - 1) as u8;
        self.write(self.S as u16 + 0x100, (self.PC - 1) as u8);
        self.S = (self.S as u16 - 1) as u8;
        self.PC = self.addr;
        self.cycles += 2;
    }

    fn las(&self) {
        let t = self.read(self.addr) & self.S;
        self.S = t;
        self.X = t;
        self.A = t;
        self.fnz(self.A as u16);
    }


    fn lax(&self) {
        let t = self.read(self.addr);
        self.X = t;
        self.A = t;
        self.fnz(self.A as u16);
    }


    fn lda(&self) {
        self.A = self.read(self.addr);
        self.fnz(self.A as u16);
    }

    fn ldx(&self) {
        self.X = self.read(self.addr);
        self.fnz(self.X as u16);
    }

    fn ldy(&self) {
        self.Y = self.read(self.addr);
        self.fnz(self.Y as u16);
    }

    fn ora(&self) {
        self.A |= self.read(self.addr);
        self.fnz(self.A as u16);
    }

    fn rol(&self) {
        self.tmp = ((self.read16(self.addr)) << 1) | (if self.C {1} else {0});
        self.fnzc(self.tmp);
        self.tmp &= 0xFF;
    }
    fn rla(&self) {
        self.tmp = ((self.A as u16) << 1) | (if self.C {1} else {0});
        self.fnzc(self.tmp);
        self.A = self.tmp as u8;
    }

    fn ror(&self) {
        self.tmp = self.read16(self.addr);
        self.tmp = ((self.tmp & 1) << 8) | ((if self.C {1} else {0}) << 7) | (self.tmp >> 1);
        self.fnzc(self.tmp);
        self.tmp &= 0xFF;
    }
    fn rra(&self) {
        self.tmp = ((self.A as u16 & 1) << 8) | ((if self.C {1} else {0}) << 7) | (self.A as u16 >> 1);
        self.fnzc(self.tmp);
        self.A = self.tmp as u8;
    }

    fn kil(&self) {

    }

    fn lsr(&self) {
        self.tmp = self.read16(self.addr);
        self.tmp = ((self.tmp & 1) << 8) | (self.tmp >> 1);
        self.fnzc(self.tmp);
        self.tmp &= 0xFF;
    }
    fn lsra(&self) {
        self.tmp = ((self.A as u16 & 1) << 8) | (self.A as u16 >> 1);
        self.fnzc(self.tmp);
        self.A = self.tmp as u8;
    }


    fn nop(&self) { }

    fn pha(&self) {
        self.write(self.S as u16 + 0x100, self.A);
        self.S = (self.S as u16 - 1) as u8;
        self.cycles +=1;
    }

    fn php(&self) {
        let mut v = if self.N {1 << 7} else {0};
        v |= if self.V {1 << 6} else {0};
        v |= 3 << 4;
        v |= if self.D {1<< 3} else {0};
        v |= if self.I {1 << 2} else {0};
        v |= if self.Z {1 << 1} else {0};
        v |= if self.C {1} else {0};
        self.write(self.S as u16 + 0x100, v);
        self.S = (self.S as u16 - 1) as u8;
        self.cycles+=1;
    }

    fn pla(&self) {
        self.S = (self.S + 1) & 0xFF;
        self.A = self.read(self.S + 0x100);
        self.fnz(self.A);
        self.cycles += 2;
    }

    fn plp(&self) {
        self.S = (self.S + 1) & 0xFF;
        self.tmp = self.read(self.S + 0x100);
        self.N = ((self.tmp & 0x80) != 0);
        self.V = ((self.tmp & 0x40) != 0);
        self.D = ((self.tmp & 0x08) != 0);
        I = ((self.tmp & 0x04) != 0);
        self.Z = ((self.tmp & 0x02) != 0);
        self.C = ((self.tmp & 0x01) != 0);
        self.cycles += 2;
    }

    fn rti(&self) {
        self.S = (self.S + 1) & 0xFF;
        self.tmp = self.read(self.S + 0x100);
        self.N = ((self.tmp & 0x80) != 0);
        self.V = ((self.tmp & 0x40) != 0);
        self.D = ((self.tmp & 0x08) != 0);
        I = ((self.tmp & 0x04) != 0);
        self.Z = ((self.tmp & 0x02) != 0);
        self.C = ((self.tmp & 0x01) != 0);
        self.S = (self.S + 1) & 0xFF;
        self.PC = self.read(self.S + 0x100);
        self.S = (self.S + 1) & 0xFF;
        self.PC |= self.read(self.S + 0x100) << 8;
        self.cycles += 4;
    }

    fn rts(&self) {
        self.S = (self.S + 1) & 0xFF;
        self.PC = self.read(self.S + 0x100);
        self.S = (self.S + 1) & 0xFF;
        self.PC |= self.read(self.S + 0x100) << 8;
        self.PC++;
        self.cycles += 4;
    }

    fn sax(&self) {
        self.write(self.addr, self.A & self.X);
    }

    fn sbc(&self) {
        let v: u16 = self.read(self.addr);
        let c: u16 = 1 - (self.C ? 1 : 0);
        let r: u16 = self.A - v - c;
        if (self.D) {
            let al = (self.A & 0x0F) - (v & 0x0F) - c;
            if (al > 0x80) al -= 6;
            let ah = (self.A >> 4) - (v >> 4) - ((al > 0x80) ? 1 : 0);
            self.Z = ((r & 0xFF) == 0);
            self.N = ((r & 0x80) != 0);
            self.V = (((self.A ^ v) & (self.A ^ r) & 0x80) != 0);
            self.C = ((r & 0x100) != 0) ? 0 : 1;
            if (ah > 0x80) ah -= 6;
            self.A = ((ah << 4) | (al & 15)) & 0xFF;
        } else {
            self.Z = ((r & 0xFF) == 0);
            self.N = ((r & 0x80) != 0);
            self.V = (((self.A ^ v) & (self.A ^ r) & 0x80) != 0);
            self.C = ((r & 0x100) != 0) ? 0 : 1;
            self.A = r & 0xFF;
        }
    }

    fn sbx(&self) {
        self.tmp = self.read(self.addr) - (self.A & self.X);
        self.fnzb(self.tmp);
        self.X = (self.tmp & 0xFF);
    }

    fn sec(&self) { self.C = 1; }
    fn sed(&self) { self.D = 1; }
    fn sei(&self) { I = 1; }

    fn shs(&self) {
        self.tmp = ((self.addr >> 8) + 1) & self.A & self.X;
        self.write(self.addr, self.tmp & 0xFF);
        self.S = (self.tmp & 0xFF);
    }

    fn shx(&self) {
        self.tmp = ((self.addr >> 8) + 1) & self.X;
        self.write(self.addr, self.tmp & 0xFF);
    }

    fn shy(&self) {
        self.tmp = ((self.addr >> 8) + 1) & Y;
        self.write(self.addr, self.tmp & 0xFF);
    }


    fn slo(&self) {
        self.tmp = self.read(self.addr) << 1;
        self.tmp |= self.A;
        self.fnzc(self.tmp);
        self.A = self.tmp & 0xFF;
    }

    fn sre(&self) {
        let v = self.read(self.addr);
        self.tmp = ((v & 1) << 8) | (v >> 1);
        self.tmp ^= self.A;
        self.fnzc(self.tmp);
        self.A = self.tmp & 0xFF;
    }


    fn sta(&self) {
        self.write(self.addr, self.A);
    }

    fn stx(&self) {
        self.write(self.addr, self.X);
    }

    fn sty(&self) {
        self.write(self.addr, Y);
    }

    fn tax(&self) {
        self.X = self.A;
        self.fnz(self.X);
    }

    fn tay(&self) {
        Y = self.A;
        self.fnz(Y);
    }

    fn tsx(&self) {
        self.X = self.S;
        self.fnz(self.X);
    }

    fn txa(&self) {
        self.A = self.X;
        self.fnz(self.A);
    }

    fn txs(&self) {
        self.S = self.X;
    }

    fn tya(&self) {
        self.A = Y;
        self.fnz(self.A);
    }


}
