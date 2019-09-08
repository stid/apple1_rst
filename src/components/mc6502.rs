use super::address_spaces::AddressSpaces;



pub struct CPU6502 {
    address_space: AddressSpaces,
    PC: u8,
    A: u8,
    X: u8,
    Y: u8,
    S: u8,
    N: u8,
    Z: u8,
    C: u8,
    V: u8,
    I: u8,
    D: u8,
    irq: u8,
    nmi: u8,
    tmp: u8,
    addr: u8,
    opcode: u8,
    cycles: u32
}

impl CPU6502 {
    pub fn init(address_space: AddressSpaces) -> CPU6502 {
        let mut cpu = CPU6502 {
            address_space: ,
            PC: 0,
            A: 0,
            X: 0,
            Y: 0,
            S: 0,
            N: 0,
            Z: 0,
            C: 0,
            V: 0,
            I: 0,
            D: 0,
            irq: 0,
            nmi: 0,
            tmp: 0,
            addr: 0,
            opcode: 0,
            cycles: 0
        };

        return cpu;
    }



    pub fn reset(&self) -> () {
        self.A = 0;
        self.X = 0;
        self.Y = 0;
        self.S = 0;
        self.N = 0;
        self.Z = 1;
        self.C = 0;
        self.V = 0;
        self.I = 0;
        self.D = 0;

        self.PC = (self.read(&0xfffd) << 8) | self.read(&0xfffc);
    }

    pub fn step(&self) -> u32 {
        let startCycles = self.cycles;
        self.PC+=1;
        self.opcode = self.read(&self.PC);

        self.exec_op(&self.opcode);
        return self.cycles - startCycles;
    }

    pub fn get_cycles(&self) -> u32 {
        return self.cycles;
    }

    fn read(&self, address: &u8) -> u8 {
        return self.addressSpace.read(address);
    }

    fn write(&self, address: &u8, value: &u8) -> () {
        self.addressSpace.write(address, value);
    }


    pub fn exec_op(&self, &opcode: &u8) -> () {
        match opcode {
            /*  BRK     */ 0x00 => {
                self.imp();
                self.brk();
            },
            /*  ORA     */ 0x01 => {
                self.izx();
                self.ora();
            },
            /* *KIL     */ 0x02 => {
                self.imp();
                self.kil();
            },
            /* *SLO izx */ 0x03 => {
                self.izx();
                self.slo();
                self.rmw();
            },
            /* *NOP zp  */ 0x04 => {
                self.zp();
                self.nop();
            },
            /*  ORA zp  */ 0x05 => {
                self.zp();
                self.ora();
            },
            /*  ASL zp  */ 0x06 => {
                self.zp();
                self.asl();
                self.rmw(),
            };
            /* *SLO zp  */ 0x07 => {
                self.zp();
                self.slo();
                self.rmw(),
            };
            /*  PHP     */ 0x08 => {
                self.imp();
                self.php();
            },
            /*  ORA imm */ 0x09 => {
                self.imm();
                self.ora();
            },
            /*  ASL     */ 0x0a => {
                self.imp();
                self.asla();
            },
            /* *ANC imm */ 0x0b => {
                self.imm();
                self.anc();
            },
            /* *NOP abs */ 0x0c => {
                self.abs();
                self.nop();
            },
            /*  ORA abs */ 0x0d => {
                self.abs();
                self.ora();
            },
            /*  ASL abs */ 0x0e => {
                self.abs();
                self.asl();
                self.rmw(),
            };
            /* *SLO abs */ 0x0f => {
                self.abs();
                self.slo();
                self.rmw(),
            };

            /*  BPL rel */ 0x10 => {
                self.rel();
                self.bpl();
            },
            /*  ORA izy */ 0x11 => {
                self.izy();
                self.ora();
            },
            /* *KIL     */ 0x12 => {
                self.imp();
                self.kil();
            },
            /* *SLO izy */ 0x13 => {
                self.izy();
                self.slo();
                self.rmw(),
            };
            /* *NOP zpx */ 0x14 => {
                self.zpx();
                self.nop();
            },
            /*  ORA zpx */ 0x15 => {
                self.zpx();
                self.ora();
            },
            /*  ASL zpx */ 0x16 => {
                self.zpx();
                self.asl();
                self.rmw(),
            };
            /* *SLO zpx */ 0x17 => {
                self.zpx();
                self.slo();
                self.rmw(),
            };
            /*  CLC     */ 0x18 => {
                self.imp();
                self.clc();
            },
            /*  ORA aby */ 0x19 => {
                self.aby();
                self.ora();
            },
            /* *NOP     */ 0x1a => {
                self.imp();
                self.nop();
            },
            /* *SLO aby */ 0x1b => {
                self.aby();
                self.slo();
                self.rmw(),
            };
            /* *NOP abx */ 0x1c => {
                self.abx();
                self.nop();
            },
            /*  ORA abx */ 0x1d => {
                self.abx();
                self.ora();
            },
            /*  ASL abx */ 0x1e => {
                self.abx();
                self.asl();
                self.rmw(),
            };
            /* *SLO abx */ 0x1f => {
                self.abx();
                self.slo();
                self.rmw(),
            };

            /*  JSR abs */ 0x20 => {
                self.abs();
                self.jsr();
            },
            /*  AND izx */ 0x21 => {
                self.izx();
                self.and();
            },
            /* *KIL     */ 0x22 => {
                self.imp();
                self.kil();
            },
            /* *RLA izx */ 0x23 => {
                self.izx();
                self.rla();
                self.rmw(),
            };
            /*  BIT zp  */ 0x24 => {
                self.zp();
                self.bit();
            },
            /*  AND zp  */ 0x25 => {
                self.zp();
                self.and();
            },
            /*  ROL zp  */ 0x26 => {
                self.zp();
                self.rol();
                self.rmw(),
            };
            /* *RLA zp  */ 0x27 => {
                self.zp();
                self.rla();
                self.rmw(),
            };
            /*  PLP     */ 0x28 => {
                self.imp();
                self.plp();
            },
            /*  AND imm */ 0x29 => {
                self.imm();
                self.and();
            },
            /*  ROL     */ 0x2a => {
                self.imp();
                self.rola();
            },
            /* *ANC imm */ 0x2b => {
                self.imm();
                self.anc();
            },
            /*  BIT abs */ 0x2c => {
                self.abs();
                self.bit();
            },
            /*  AND abs */ 0x2d => {
                self.abs();
                self.and();
            },
            /*  ROL abs */ 0x2e => {
                self.abs();
                self.rol();
                self.rmw(),
            };
            /* *RLA abs */ 0x2f => {
                self.abs();
                self.rla();
                self.rmw(),
            };

            /*  BMI rel */ 0x30 => {
                self.rel();
                self.bmi();
            },
            /*  AND izy */ 0x31 => {
                self.izy();
                self.and();
            },
            /* *KIL     */ 0x32 => {
                self.imp();
                self.kil();
            },
            /* *RLA izy */ 0x33 => {
                self.izy();
                self.rla();
                self.rmw(),
            };
            /* *NOP zpx */ 0x34 => {
                self.zpx();
                self.nop();
            },
            /*  AND zpx */ 0x35 => {
                self.zpx();
                self.and();
            },
            /*  ROL zpx */ 0x36 => {
                self.zpx();
                self.rol();
                self.rmw(),
            };
            /* *RLA zpx */ 0x37 => {
                self.zpx();
                self.rla();
                self.rmw(),
            };
            /*  SEC     */ 0x38 => {
                self.imp();
                self.sec();
            },
            /*  AND aby */ 0x39 => {
                self.aby();
                self.and();
            },
            /* *NOP     */ 0x3a => {
                self.imp();
                self.nop();
            },
            /* *RLA aby */ 0x3b => {
                self.aby();
                self.rla();
                self.rmw(),
            };
            /* *NOP abx */ 0x3c => {
                self.abx();
                self.nop();
            },
            /*  AND abx */ 0x3d => {
                self.abx();
                self.and();
            },
            /*  ROL abx */ 0x3e => {
                self.abx();
                self.rol();
                self.rmw(),
            };
            /* *RLA abx */ 0x3f => {
                self.abx();
                self.rla();
                self.rmw(),
            };

            /*  RTI     */ 0x40 => {
                self.imp();
                self.rti();
            },
            /*  EOR izx */ 0x41 => {
                self.izx();
                self.eor();
            },
            /* *KIL     */ 0x42 => {
                self.imp();
                self.kil();
            },
            /* *SRE izx */ 0x43 => {
                self.izx();
                self.sre();
                self.rmw(),
            };
            /* *NOP zp  */ 0x44 => {
                self.zp();
                self.nop();
            },
            /*  EOR zp  */ 0x45 => {
                self.zp();
                self.eor();
            },
            /*  LSR zp  */ 0x46 => {
                self.zp();
                self.lsr();
                self.rmw(),
            };
            /* *SRE zp  */ 0x47 => {
                self.zp();
                self.sre();
                self.rmw(),
            };
            /*  PHA     */ 0x48 => {
                self.imp();
                self.pha();
            },
            /*  EOR imm */ 0x49 => {
                self.imm();
                self.eor();
            },
            /*  LSR     */ 0x4a => {
                self.imp();
                self.lsra();
            },
            /* *ALR imm */ 0x4b => {
                self.imm();
                self.alr();
            },
            /*  JMP abs */ 0x4c => {
                self.abs();
                self.jmp();
            },
            /*  EOR abs */ 0x4d => {
                self.abs();
                self.eor();
            },
            /*  LSR abs */ 0x4e => {
                self.abs();
                self.lsr();
                self.rmw(),
            };
            /* *SRE abs */ 0x4f => {
                self.abs();
                self.sre();
                self.rmw(),
            };

            /*  BVC rel */ 0x50 => {
                self.rel();
                self.bvc();
            },
            /*  EOR izy */ 0x51 => {
                self.izy();
                self.eor();
            },
            /* *KIL     */ 0x52 => {
                self.imp();
                self.kil();
            },
            /* *SRE izy */ 0x53 => {
                self.izy();
                self.sre();
                self.rmw(),
            };
            /* *NOP zpx */ 0x54 => {
                self.zpx();
                self.nop();
            },
            /*  EOR zpx */ 0x55 => {
                self.zpx();
                self.eor();
            },
            /*  LSR zpx */ 0x56 => {
                self.zpx();
                self.lsr();
                self.rmw(),
            };
            /* *SRE zpx */ 0x57 => {
                self.zpx();
                self.sre();
                self.rmw(),
            };
            /*  CLI     */ 0x58 => {
                self.imp();
                self.cli();
            },
            /*  EOR aby */ 0x59 => {
                self.aby();
                self.eor();
            },
            /* *NOP     */ 0x5a => {
                self.imp();
                self.nop();
            },
            /* *SRE aby */ 0x5b => {
                self.aby();
                self.sre();
                self.rmw(),
            };
            /* *NOP abx */ 0x5c => {
                self.abx();
                self.nop();
            },
            /*  EOR abx */ 0x5d => {
                self.abx();
                self.eor();
            },
            /*  LSR abx */ 0x5e => {
                self.abx();
                self.lsr();
                self.rmw(),
            };
            /* *SRE abx */ 0x5f => {
                self.abx();
                self.sre();
                self.rmw(),
            };

            /*  RTS     */ 0x60 => {
                self.imp();
                self.rts();
            },
            /*  ADC izx */ 0x61 => {
                self.izx();
                self.adc();
            },
            /* *KIL     */ 0x62 => {
                self.imp();
                self.kil();
            },
            /* *RRA izx */ 0x63 => {
                self.izx();
                self.rra();
                self.rmw(),
            };
            /* *NOP zp  */ 0x64 => {
                self.zp();
                self.nop();
            },
            /*  ADC zp  */ 0x65 => {
                self.zp();
                self.adc();
            },
            /*  ROR zp  */ 0x66 => {
                self.zp();
                self.ror();
                self.rmw(),
            };
            /* *RRA zp  */ 0x67 => {
                self.zp();
                self.rra();
                self.rmw(),
            };
            /*  PLA     */ 0x68 => {
                self.imp();
                self.pla();
            },
            /*  ADC imm */ 0x69 => {
                self.imm();
                self.adc();
            },
            /*  ROR     */ 0x6a => {
                self.imp();
                self.rora();
            },
            /* *ARR imm */ 0x6b => {
                self.imm();
                self.arr();
            },
            /*  JMP ind */ 0x6c => {
                self.ind();
                self.jmp();
            },
            /*  ADC abs */ 0x6d => {
                self.abs();
                self.adc();
            },
            /*  ROR abs */ 0x6e => {
                self.abs();
                self.ror();
                self.rmw(),
            };
            /* *RRA abs */ 0x6f => {
                self.abs();
                self.rra();
                self.rmw(),
            };

            /*  BVS rel */ 0x70 => {
                self.rel();
                self.bvs();
            },
            /*  ADC izy */ 0x71 => {
                self.izy();
                self.adc();
            },
            /* *KIL     */ 0x72 => {
                self.imp();
                self.kil();
            },
            /* *RRA izy */ 0x73 => {
                self.izy();
                self.rra();
                self.rmw(),
            };
            /* *NOP zpx */ 0x74 => {
                self.zpx();
                self.nop();
            },
            /*  ADC zpx */ 0x75 => {
                self.zpx();
                self.adc();
            },
            /*  ROR zpx */ 0x76 => {
                self.zpx();
                self.ror();
                self.rmw(),
            };
            /* *RRA zpx */ 0x77 => {
                self.zpx();
                self.rra();
                self.rmw(),
            };
            /*  SEI     */ 0x78 => {
                self.imp();
                self.sei();
            },
            /*  ADC aby */ 0x79 => {
                self.aby();
                self.adc();
            },
            /* *NOP     */ 0x7a => {
                self.imp();
                self.nop();
            },
            /* *RRA aby */ 0x7b => {
                self.aby();
                self.rra();
                self.rmw(),
            };
            /* *NOP abx */ 0x7c => {
                self.abx();
                self.nop();
            },
            /*  ADC abx */ 0x7d => {
                self.abx();
                self.adc();
            },
            /*  ROR abx */ 0x7e => {
                self.abx();
                self.ror();
                self.rmw(),
            };
            /* *RRA abx */ 0x7f => {
                self.abx();
                self.rra();
                self.rmw(),
            };

            /* *NOP imm */ 0x80 => {
                self.imm();
                self.nop();
            },
            /*  STA izx */ 0x81 => {
                self.izx();
                self.sta();
            },
            /* *NOP imm */ 0x82 => {
                self.imm();
                self.nop();
            },
            /* *SAX izx */ 0x83 => {
                self.izx();
                self.sax();
            },
            /*  STY zp  */ 0x84 => {
                self.zp();
                self.sty();
            },
            /*  STA zp  */ 0x85 => {
                self.zp();
                self.sta();
            },
            /*  STX zp  */ 0x86 => {
                self.zp();
                self.stx();
            },
            /* *SAX zp  */ 0x87 => {
                self.zp();
                self.sax();
            },
            /*  DEY     */ 0x88 => {
                self.imp();
                self.dey();
            },
            /* *NOP imm */ 0x89 => {
                self.imm();
                self.nop();
            },
            /*  TXA     */ 0x8a => {
                self.imp();
                self.txa();
            },
            /* *XAA imm */ 0x8b => {
                self.imm();
                self.xaa();
            },
            /*  STY abs */ 0x8c => {
                self.abs();
                self.sty();
            },
            /*  STA abs */ 0x8d => {
                self.abs();
                self.sta();
            },
            /*  STX abs */ 0x8e => {
                self.abs();
                self.stx();
            },
            /* *SAX abs */ 0x8f => {
                self.abs();
                self.sax();
            },

            /*  BCC rel */ 0x90 => {
                self.rel();
                self.bcc();
            },
            /*  STA izy */ 0x91 => {
                self.izy();
                self.sta();
            },
            /* *KIL     */ 0x92 => {
                self.imp();
                self.kil();
            },
            /* *AHX izy */ 0x93 => {
                self.izy();
                self.ahx();
            },
            /*  STY zpx */ 0x94 => {
                self.zpx();
                self.sty();
            },
            /*  STA zpx */ 0x95 => {
                self.zpx();
                self.sta();
            },
            /*  STX zpy */ 0x96 => {
                self.zpy();
                self.stx();
            },
            /* *SAX zpy */ 0x97 => {
                self.zpy();
                self.sax();
            },
            /*  TYA     */ 0x98 => {
                self.imp();
                self.tya();
            },
            /*  STA aby */ 0x99 => {
                self.aby();
                self.sta();
            },
            /*  TXS     */ 0x9a => {
                self.imp();
                self.txs();
            },
            /* *TAS aby */ 0x9b => {
                self.aby();
                self.tas();
            },
            /* *SHY abx */ 0x9c => {
                self.abx();
                self.shy();
            },
            /*  STA abx */ 0x9d => {
                self.abx();
                self.sta();
            },
            /* *SHX aby */ 0x9e => {
                self.aby();
                self.shx();
            },
            /* *AHX aby */ 0x9f => {
                self.aby();
                self.ahx();
            },

            /*  LDY imm */ 0xa0 => {
                self.imm();
                self.ldy();
            },
            /*  LDA izx */ 0xa1 => {
                self.izx();
                self.lda();
            },
            /*  LDX imm */ 0xa2 => {
                self.imm();
                self.ldx();
            },
            /* *LAX izx */ 0xa3 => {
                self.izx();
                self.lax();
            },
            /*  LDY zp  */ 0xa4 => {
                self.zp();
                self.ldy();
            },
            /*  LDA zp  */ 0xa5 => {
                self.zp();
                self.lda();
            },
            /*  LDX zp  */ 0xa6 => {
                self.zp();
                self.ldx();
            },
            /* *LAX zp  */ 0xa7 => {
                self.zp();
                self.lax();
            },
            /*  TAY     */ 0xa8 => {
                self.imp();
                self.tay();
            },
            /*  LDA imm */ 0xa9 => {
                self.imm();
                self.lda();
            },
            /*  TAX     */ 0xaa => {
                self.imp();
                self.tax();
            },
            /* *LAX imm */ 0xab => {
                self.imm();
                self.lax();
            },
            /*  LDY abs */ 0xac => {
                self.abs();
                self.ldy();
            },
            /*  LDA abs */ 0xad => {
                self.abs();
                self.lda();
            },
            /*  LDX abs */ 0xae => {
                self.abs();
                self.ldx();
            },
            /* *LAX abs */ 0xaf => {
                self.abs();
                self.lax();
            },

            /*  BCS rel */ 0xb0 => {
                self.rel();
                self.bcs();
            },
            /*  LDA izy */ 0xb1 => {
                self.izy();
                self.lda();
            },
            /* *KIL     */ 0xb2 => {
                self.imp();
                self.kil();
            },
            /* *LAX izy */ 0xb3 => {
                self.izy();
                self.lax();
            },
            /*  LDY zpx */ 0xb4 => {
                self.zpx();
                self.ldy();
            },
            /*  LDA zpx */ 0xb5 => {
                self.zpx();
                self.lda();
            },
            /*  LDX zpy */ 0xb6 => {
                self.zpy();
                self.ldx();
            },
            /* *LAX zpy */ 0xb7 => {
                self.zpy();
                self.lax();
            },
            /*  CLV     */ 0xb8 => {
                self.imp();
                self.clv();
            },
            /*  LDA aby */ 0xb9 => {
                self.aby();
                self.lda();
            },
            /*  TSX     */ 0xba => {
                self.imp();
                self.tsx();
            },
            /* *LAS aby */ 0xbb => {
                self.aby();
                self.las();
            },
            /*  LDY abx */ 0xbc => {
                self.abx();
                self.ldy();
            },
            /*  LDA abx */ 0xbd => {
                self.abx();
                self.lda();
            },
            /*  LDX aby */ 0xbe => {
                self.aby();
                self.ldx();
            },
            /* *LAX aby */ 0xbf => {
                self.aby();
                self.lax();
            },

            /*  CPY imm */ 0xc0 => {
                self.imm();
                self.cpy();
            },
            /*  CMP izx */ 0xc1 => {
                self.izx();
                self.cmp();
            },
            /* *NOP imm */ 0xc2 => {
                self.imm();
                self.nop();
            },
            /* *DCP izx */ 0xc3 => {
                self.izx();
                self.dcp();
                self.rmw(),
            };
            /*  CPY zp  */ 0xc4 => {
                self.zp();
                self.cpy();
            },
            /*  CMP zp  */ 0xc5 => {
                self.zp();
                self.cmp();
            },
            /*  DEC zp  */ 0xc6 => {
                self.zp();
                self.dec();
                self.rmw(),
            };
            /* *DCP zp  */ 0xc7 => {
                self.zp();
                self.dcp();
                self.rmw(),
            };
            /*  INY     */ 0xc8 => {
                self.imp();
                self.iny();
            },
            /*  CMP imm */ 0xc9 => {
                self.imm();
                self.cmp();
            },
            /*  DEX     */ 0xca => {
                self.imp();
                self.dex();
            },
            /* *AXS imm */ 0xcb => {
                self.imm();
                self.axs();
            },
            /*  CPY abs */ 0xcc => {
                self.abs();
                self.cpy();
            },
            /*  CMP abs */ 0xcd => {
                self.abs();
                self.cmp();
            },
            /*  DEC abs */ 0xce => {
                self.abs();
                self.dec();
                self.rmw(),
            };
            /* *DCP abs */ 0xcf => {
                self.abs();
                self.dcp();
                self.rmw(),
            };

            /*  BNE rel */ 0xd0 => {
                self.rel();
                self.bne();
            },
            /*  CMP izy */ 0xd1 => {
                self.izy();
                self.cmp();
            },
            /* *KIL     */ 0xd2 => {
                self.imp();
                self.kil();
            },
            /* *DCP izy */ 0xd3 => {
                self.izy();
                self.dcp();
                self.rmw(),
            };
            /* *NOP zpx */ 0xd4 => {
                self.zpx();
                self.nop();
            },
            /*  CMP zpx */ 0xd5 => {
                self.zpx();
                self.cmp();
            },
            /*  DEC zpx */ 0xd6 => {
                self.zpx();
                self.dec();
                self.rmw(),
            };
            /* *DCP zpx */ 0xd7 => {
                self.zpx();
                self.dcp();
                self.rmw(),
            };
            /*  CLD     */ 0xd8 => {
                self.imp();
                self.cld();
            },
            /*  CMP aby */ 0xd9 => {
                self.aby();
                self.cmp();
            },
            /* *NOP     */ 0xda => {
                self.imp();
                self.nop();
            },
            /* *DCP aby */ 0xdb => {
                self.aby();
                self.dcp();
                self.rmw(),
            };
            /* *NOP abx */ 0xdc => {
                self.abx();
                self.nop();
            },
            /*  CMP abx */ 0xdd => {
                self.abx();
                self.cmp();
            },
            /*  DEC abx */ 0xde => {
                self.abx();
                self.dec();
                self.rmw(),
            };
            /* *DCP abx */ 0xdf => {
                self.abx();
                self.dcp();
                self.rmw(),
            };

            /*  CPX imm */ 0xe0 => {
                self.imm();
                self.cpx();
            },
            /*  SBC izx */ 0xe1 => {
                self.izx();
                self.sbc();
            },
            /* *NOP imm */ 0xe2 => {
                self.imm();
                self.nop();
            },
            /* *ISC izx */ 0xe3 => {
                self.izx();
                self.isc();
                self.rmw(),
            };
            /*  CPX zp  */ 0xe4 => {
                self.zp();
                self.cpx();
            },
            /*  SBC zp  */ 0xe5 => {
                self.zp();
                self.sbc();
            },
            /*  INC zp  */ 0xe6 => {
                self.zp();
                self.inc();
                self.rmw(),
            };
            /* *ISC zp  */ 0xe7 => {
                self.zp();
                self.isc();
                self.rmw(),
            };
            /*  INX     */ 0xe8 => {
                self.imp();
                self.inx();
            },
            /*  SBC imm */ 0xe9 => {
                self.imm();
                self.sbc();
            },
            /*  NOP     */ 0xea => {
                self.imp();
                self.nop();
            },
            /* *SBC imm */ 0xeb => {
                self.imm();
                self.sbc();
            },
            /*  CPX abs */ 0xec => {
                self.abs();
                self.cpx();
            },
            /*  SBC abs */ 0xed => {
                self.abs();
                self.sbc();
            },
            /*  INC abs */ 0xee => {
                self.abs();
                self.inc();
                self.rmw(),
            };
            /* *ISC abs */ 0xef => {
                self.abs();
                self.isc();
                self.rmw(),
            };

            /*  BEQ rel */ 0xf0 => {
                self.rel();
                self.beq();
            },
            /*  SBC izy */ 0xf1 => {
                self.izy();
                self.sbc();
            },
            /* *KIL     */ 0xf2 => {
                self.imp();
                self.kil();
            },
            /* *ISC izy */ 0xf3 => {
                self.izy();
                self.isc();
                self.rmw(),
            };
            /* *NOP zpx */ 0xf4 => {
                self.zpx();
                self.nop();
            },
            /*  SBC zpx */ 0xf5 => {
                self.zpx();
                self.sbc();
            },
            /*  INC zpx */ 0xf6 => {
                self.zpx();
                self.inc();
                self.rmw(),
            };
            /* *ISC zpx */ 0xf7 => {
                self.zpx();
                self.isc();
                self.rmw(),
            };
            /*  SED     */ 0xf8 => {
                self.imp();
                self.sed();
            },
            /*  SBC aby */ 0xf9 => {
                self.aby();
                self.sbc();
            },
            /* *NOP     */ 0xfa => {
                self.imp();
                self.nop();
            },
            /* *ISC aby */ 0xfb => {
                self.aby();
                self.isc();
                self.rmw(),
            };
            /* *NOP abx */ 0xfc => {
                self.abx();
                self.nop();
            },
            /*  SBC abx */ 0xfd => {
                self.abx();
                self.sbc();
            },
            /*  INC abx */ 0xfe => {
                self.abx();
                self.inc();
                self.rmw(),
            };
            /* *ISC abx */ 0xff => {
                self.abx();
                self.isc();
                self.rmw(),
            };
        }
    }



    fn izx(&mut self) -> () {
        self.PC+=1;
        let mut a = (self.read(&self.PC) + self.X) & 0xff;
        self.addr = self.read(&((a + 1) << 8)) | self.read(&a);
        self.cycles += 6;
    }

    fn izy(&mut self) -> () {
        self.PC+=1;
        let a = self.read(&self.PC);
        let paddr = (self.read(&((a + 1) & 0xff)) << 8) | self.read(&a);
        self.addr = paddr + self.Y;
        if (paddr & 0x100) != (self.addr & 0x100) {
            self.cycles += 6;
        } else {
            self.cycles += 5;
        }
    }

    fn nd(&self) -> () {
        let a = self.read(&self.PC);
        a |= self.read(&((self.PC & 0xff00) | ((self.PC + 1) & 0xff))) << 8;
        self.addr = self.read(&a);
        self.addr |= self.read(&(a + 1)) << 8;
        self.cycles += 5;
    }

    fn zp(&self) -> () {
        self.PC+=1;
        self.addr = self.read(&self.PC);
        self.cycles += 3;
    }

    fn zpx(&self) -> () {
        self.PC+=1;
        self.addr = (self.read(&self.PC) + self.X) & 0xff;
        self.cycles += 4;
    }

    fn zpy(&self) -> () {
        self.PC+=1;
        self.addr = (self.read(&self.PC) + self.Y) & 0xff;
        self.cycles += 4;
    }

    fn imp(&self) -> () {
        self.cycles += 2;
    }

    fn imm(&self) -> () {
        self.PC+=1;
        self.addr = self.PC;
        self.cycles += 2;
    }

    fn abs(&self) -> () {
        self.PC+=1;
        self.addr = self.read(&self.PC);
        self.addr |= self.read(&self.PC) << 8;
        self.cycles += 4;
    }

    fn abx(&self) -> () {
        self.PC+=1;
        let paddr = self.read(&self.PC);
        self.PC+=1;
        paddr |= self.read(&self.PC) << 8;
        self.addr = paddr + self.X;
        if (paddr & 0x100) != (self.addr & 0x100) {
            self.cycles += 5;
        } else {
            self.cycles += 4;
        }
    }

    fn aby(&self) -> () {
        self.PC+=1;
        let paddr = self.read(&self.PC);
        self.PC+=1;
        paddr |= self.read(&self.PC) << 8;
        self.addr = paddr + self.Y;
        if (paddr & 0x100) != (self.addr & 0x100) {
            self.cycles += 5;
        } else {
            self.cycles += 4;
        }
    }

    fn rel(&self) -> () {
        self.PC+=1;
        self.addr = self.read(&self.PC);
        if self.addr & 0x80 != 0 {
            self.addr -= 0x100;
        }
        self.addr += self.PC;
        self.cycles += 2;
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn rmw(&self) -> () {
        self.write(&self.addr, &(self.tmp & 0xff));
        self.cycles += 2;
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn fnz(&self, v: u8) -> () {
        self.Z = if (v & 0xff) == 0 {1} else {0};
        self.N = if (v & 0x80) != 0 {1} else {0};
    }

    // Borrow
    fn fnzb(&self, v: u8) -> () {
        self.Z = if (v & 0xff) == 0 {1} else {0};
        self.N = if (v & 0x80) != 0 {1} else {0};
        self.C = if (v & 0x100) != 0 {0} else {1};
    }

    // Carry
    fn fnzc(&self, v: u8) -> () {
        self.Z = if (v & 0xff) == 0  {1} else {0};
        self.N = if (v & 0x80) != 0  {1} else {0};
        self.C = if (v & 0x100) != 0  {1} else {0};
    }

    fn branch(&self, v: u8) -> () {
        if (self.addr & 0x100) != (self.PC & 0x100) {
            self.cycles += 2;
        } else {
            self.cycles += 1;
        }
        self.PC = self.addr;
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Subroutines - instructions
    ////////////////////////////////////////////////////////////////////////////////
    fn adc(&self) -> () {
        let v = self.read(&self.addr);
        let c = self.C;
        let r = self.A + v + c;
        if self.D != 0 {
            let mut al = (self.A & 0x0f) + (v & 0x0f) + c;
            if al > 9 {al += 6};
            let mut ah = (self.A >> 4) + (v >> 4) + (if al > 15  {1} else {0});
            self.Z = if (r & 0xff) == 0 {1} else {0};
            self.N = if (ah & 8) != 0 {1} else {0};
            self.V = if (!(self.A ^ v) & (self.A ^ (ah << 4)) & 0x80) != 0 {1} else {0};
            if ah > 9 {ah += 6};
            self.C = if ah > 15 {1} else {0};
            self.A = ((ah << 4) | (al & 15)) & 0xff;
        } else {
            self.Z = if (r & 0xff) == 0 {1} else {0};
            self.N = if (r & 0x80) != 0 {1} else {0};
            self.V = if (!(self.A ^ v) & (self.A ^ r) & 0x80) != 0 {1} else {0};
            self.C = if (r & 0x100) != 0 {1} else {0};
            self.A = r & 0xff;
        }
    }

    fn and(&self) -> () {
        self.A &= self.read(&self.addr);
        self.fnz(self.A);
    }

    fn asl(&self) -> () {
        self.tmp = self.read(&self.addr) << 1;
        self.fnzc(self.tmp);
        self.tmp &= 0xff;
    }

    fn asla(&self) -> () {
        self.tmp = self.A << 1;
        self.fnzc(self.tmp);
        self.A = self.tmp & 0xff;
    }

    fn fnbit(&self) -> () {
        self.tmp = self.read(&self.addr);
        self.N = if (self.tmp & 0x80) != 0 {1} else {0};
        self.V = if (self.tmp & 0x40) != 0 {1} else {0};
        self.Z = if (self.tmp & self.A) == 0 {1} else {0};
    }

    fn brk(&self) -> () {
        self.PC+=1;
        self.write(&(self.S + 0x100), &(self.PC >> 8));
        self.S = (self.S - 1) & 0xff;
        self.write(&(self.S + 0x100), & (self.PC & 0xff));
        self.S = (self.S - 1) & 0xff;
        let v = self.N << 7;
        v |= self.V << 6;
        v |= 3 << 4;
        v |= self.D << 3;
        v |= self.I << 2;
        v |= self.Z << 1;
        v |= self.C;
        self.write(&(self.S + 0x100), &v);
        self.S = (self.S - 1) & 0xff;
        self.I = 1;
        self.D = 0;
        self.PC = (self.read(&0xffff) << 8) | self.read(&0xfffe);
        self.cycles += 5;
    }

    fn bcc(&self) -> () {
        self.branch(if self.C == 0 {1} else {0});
    }
    fn bcs(&self) -> () {
        self.branch(if self.C == 1 {1} else {0});
    }
    fn beq(&self) -> () {
        self.branch(if self.Z == 1 {1} else {0});
    }
    fn bne(&self) -> () {
        self.branch(if self.Z == 0 {1} else {0});
    }
    fn bmi(&self) -> () {
        self.branch(if self.N == 1 {1} else {0});
    }
    fn bpl(&self) -> () {
        self.branch(if self.N == 0 {1} else {0});
    }
    fn bvc(&self) -> () {
        self.branch(if self.V == 0 {1} else {0});
    }
    fn bvs(&self) -> () {
        self.branch(if self.V == 1 {1} else {0});
    }

    fn clc(&self) -> () {
        self.C = 0;
    }
    fn cld(&self) -> () {
        self.D = 0;
    }
    fn cli(&self) -> () {
        self.I = 0;
    }
    fn clv(&self) -> () {
        self.V = 0;
    }

    fn cmp(&self) -> () {
        self.fnzb(self.A - self.read(&self.addr));
    }

    fn cpx(&self) -> () {
        self.fnzb(self.X - self.read(&self.addr));
    }

    fn cpy(&self) -> () {
        self.fnzb(self.Y - self.read(&self.addr));
    }

    fn dec(&self) -> () {
        self.tmp = (self.read(&self.addr) - 1) & 0xff;
        self.fnz(self.tmp);
    }

    fn dex(&self) -> () {
        self.X = (self.X - 1) & 0xff;
        self.fnz(self.X);
    }

    fn dey(&self) -> () {
        self.Y = (self.Y - 1) & 0xff;
        self.fnz(self.Y);
    }

    fn eor(&self) -> () {
        self.A ^= self.read(& self.addr);
        self.fnz(self.A);
    }

    fn inc(&self) -> () {
        self.tmp = (self.read(& self.addr) + 1) & 0xff;
        self.fnz(self.tmp);
    }

    fn inx(&self) {
        self.X = (self.X + 1) & 0xff;
        self.fnz(self.X);
    }

    fn iny(&self) -> () {
        self.Y = (self.Y + 1) & 0xff;
        self.fnz(self.Y);
    }

    fn jmp(&self) -> () {
        self.PC = self.addr;
        self.cycles-=1;
    }

    fn jsr(&self) -> () {
        self.write(&(self.S + 0x100), &((self.PC - 1) >> 8));
        self.S = (self.S - 1) & 0xff;
        self.write(&(self.S + 0x100), &((self.PC - 1) & 0xff));
        self.S = (self.S - 1) & 0xff;
        self.PC = self.addr;
        self.cycles += 2;
    }

    fn lda(&self) -> () {
        self.A = self.read(&self.addr);
        self.fnz(self.A);
    }

    fn ldx(&self) -> () {
        self.X = self.read(&self.addr);
        self.fnz(self.X);
    }

    fn ldy(&self) -> () {
        self.Y = self.read(&self.addr);
        self.fnz(self.Y);
    }

    fn ora(&self) -> () {
        self.A |= self.read(&self.addr);
        self.fnz(self.A);
    }

    fn rol(&self) -> () {
        self.tmp = (self.read(&self.addr) << 1) | self.C;
        self.fnzc(self.tmp);
        self.tmp &= 0xff;
    }
    fn rola(&self) -> () {
        self.tmp = (self.A << 1) | self.C;
        self.fnzc(self.tmp);
        self.A = self.tmp & 0xff;
    }

    fn ror(&self) -> () {
        self.tmp = self.read(&self.addr);
        self.tmp = ((self.tmp & 1) << 8) | (self.C << 7) | (self.tmp >> 1);
        self.fnzc(self.tmp);
        self.tmp &= 0xff;
    }
    fn rora(&self) -> () {
        self.tmp = ((self.A & 1) << 8) | (self.C << 7) | (self.A >> 1);
        self.fnzc(self.tmp);
        self.A = self.tmp & 0xff;
    }

    fn lsr(&self) -> () {
        self.tmp = self.read(&self.addr);
        self.tmp = ((self.tmp & 1) << 8) | (self.tmp >> 1);
        self.fnzc(self.tmp);
        self.tmp &= 0xff;
    }
    fn lsra(&self) -> () {
        self.tmp = ((self.A & 1) << 8) | (self.A >> 1);
        self.fnzc(self.tmp);
        self.A = self.tmp & 0xff;
    }

    fn nop(&self) -> () {
        return;
    }

    fn pha(&self) -> () {
        self.write(&(self.S + 0x100), &self.A);
        self.S = (self.S - 1) & 0xff;
        self.cycles+=1;
    }

    fn php(&self) -> () {
        let v = self.N << 7;
        v |= self.V << 6;
        v |= 3 << 4;
        v |= self.D << 3;
        v |= self.I << 2;
        v |= self.Z << 1;
        v |= self.C;
        self.write(&(self.S + 0x100), &v);
        self.S = (self.S - 1) & 0xff;
        self.cycles+=1;
    }

    fn pla(&self) -> () {
        self.S = (self.S + 1) & 0xff;
        self.A = self.read(&(self.S + 0x100));
        self.fnz(self.A);
        self.cycles += 2;
    }

    fn plp(&self) -> () {
        self.S = (self.S + 1) & 0xff;
        self.tmp = self.read(&(self.S + 0x100));
        self.N = if self.tmp & 0x80 != 0 {1} else {0};
        self.V = if self.tmp & 0x40 != 0 {1} else {0};
        self.D = if self.tmp & 0x08 != 0 {1} else {0};
        self.I = if self.tmp & 0x04 != 0 {1} else {0};
        self.Z = if self.tmp & 0x02 != 0 {1} else {0};
        self.C = if self.tmp & 0x01 != 0 {1} else {0};
        self.cycles += 2;
    }

    fn rti(&self) -> () {
        self.S = (self.S + 1) & 0xff;
        self.tmp = self.read(&(self.S + 0x100));
        self.N = if self.tmp & 0x80 != 0 {1} else {0};
        self.V = if self.tmp & 0x40 != 0 {1} else {0};
        self.D = if self.tmp & 0x08 != 0 {1} else {0};
        self.I = if self.tmp & 0x04 != 0 {1} else {0};
        self.Z = if self.tmp & 0x02 != 0 {1} else {0};
        self.C = if self.tmp & 0x01 != 0 {1} else {0};
        self.S = (self.S + 1) & 0xff;
        self.PC = self.read(&(self.S + 0x100));
        self.S = (self.S + 1) & 0xff;
        self.PC |= self.read(&(self.S + 0x100)) << 8;
        self.cycles += 4;
    }

    fn rts(&self) -> () {
        self.S = (self.S + 1) & 0xff;
        self.PC = self.read(&(self.S + 0x100));
        self.S = (self.S + 1) & 0xff;
        self.PC |= self.read(&(self.S + 0x100)) << 8;
        self.PC+=1;
        self.cycles += 4;
    }

    fn sbc(&self) -> () {
        let v = self.read(&self.addr);
        let c = 1 - self.C;
        let r = self.A - v - c;
        if self.D != 0 {
            let al = (self.A & 0x0f) - (v & 0x0f) - c;
            if al < 0 {al -= 6};
            let ah = (self.A >> 4) - (v >> 4) - (if al < 0 {1} else {0});
            self.Z = if (r & 0xff) == 0 {1} else {0};
            self.N = if (r & 0x80) != 0 {1} else {0};
            self.V = (self.A ^ v) & (self.A ^ r) & if 0x80 != 0 {1} else {0};
            self.C = if (r & 0x100) != 0 {0} else {1};
            if ah < 0 {ah -= 6};
            self.A = ((ah << 4) | (al & 15)) & 0xff;
        } else {
            self.Z = if (r & 0xff) == 0 {1} else {0};
            self.N = if (r & 0x80) != 0 {1} else {0};
            self.V = (self.A ^ v) & if ((self.A ^ r) & 0x80) != 0 {1} else {0};
            self.C = if (r & 0x100) != 0 {0} else {1};
            self.A = r & 0xff;
        }
    }

   fn sec(&self) -> () {
        self.C = 1;
    }
    fn sed(&self) -> () {
        self.D = 1;
    }
    fn sei(&self) -> () {
        self.I = 1;
    }

    fn slo(&self) -> () {
        self.tmp = self.read(&self.addr) << 1;
        self.tmp |= self.A;
        self.fnzc(self.tmp);
        self.A = self.tmp & 0xff;
    }

    fn sta(&self) -> () {
        self.write(&self.addr, &self.A);
    }

    fn stx(&self) -> () {
        self.write(&self.addr, &self.X);
    }

    fn sty(&self) -> () {
        self.write(&self.addr, &self.Y);
    }

    fn tax(&self) -> () {
        self.X = self.A;
        self.fnz(self.X);
    }

    fn tay(&self) -> () {
        self.Y = self.A;
        self.fnz(self.Y);
    }

    fn tsx(&self) -> () {
        self.X = self.S;
        self.fnz(self.X);
    }

    fn txa(&self) -> () {
        self.A = self.X;
        self.fnz(self.A);
    }

    fn txs(&self) -> () {
        self.S = self.X;
    }

    fn tya(&self) -> () {
        self.A = self.Y;
        self.fnz(self.A);
    }

    // INCOMPLETE IMPLEMENTATION

    fn isc(&self) -> () {
        return;
    }

    fn kil(&self) -> () {
        return;
    }

    fn anc(&self) -> () {
        return;
    }

    fn rla(&self) -> () {
        return;
    }

    fn sre(&self) -> () {
        return;
    }

    fn alr(&self) -> () {
        return;
    }

    fn rra(&self) -> () {
        return;
    }

    fn sax(&self) -> () {
        return;
    }

    fn lax(&self) -> () {
        return;
    }

    fn arr(&self) -> () {
        return;
    }

    fn tas(&self) -> () {
        return;
    }

    fn shy(&self) -> () {
        return;
    }

    fn axs(&self) -> () {
        return;
    }

    fn dcp(&self) -> () {
        return;
    }

    fn las(&self) -> () {
        return;
    }

    fn xaa(&self) -> () {
        return;
    }

    fn ahx(&self) -> () {
        return;
    }

    fn shx(&self) -> () {
        return;
    }

}
