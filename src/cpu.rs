use crate::{
    bus::Bus,
    config::{CPU_FLAG_START_VALUE, CPU_SP_START_VALUE, ROM_START},
    util::Error,
};

pub struct Cpu {
    pub pc: u16,
    pub sp: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub flags: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            pc: ROM_START as u16,
            sp: CPU_SP_START_VALUE,
            a: 0,
            x: 0,
            y: 0,
            flags: CPU_FLAG_START_VALUE,
        }
    }

    pub fn c(&self) -> bool {
        (self.flags & 0b0000_0001) != 0
    }

    pub fn z(&self) -> bool {
        (self.flags & 0b0000_0010) != 0
    }

    pub fn i(&self) -> bool {
        (self.flags & 0b0000_0100) != 0
    }

    pub fn d(&self) -> bool {
        (self.flags & 0b0000_1000) != 0
    }

    pub fn b(&self) -> bool {
        (self.flags & 0b0001_0000) != 0
    }

    pub fn v(&self) -> bool {
        (self.flags & 0b0100_0000) != 0
    }

    pub fn n(&self) -> bool {
        (self.flags & 0b1000_0000) != 0
    }

    pub fn set_c(&mut self, c: bool) {
        let flags = self.flags;
        self.flags = if c {
            flags | 0b0000_0001
        } else {
            flags & 0b1111_1110
        }
    }

    pub fn set_z(&mut self, z: bool) {
        let flags = self.flags;
        self.flags = if z {
            flags | 0b0000_0010
        } else {
            flags & 0b1111_1101
        }
    }

    pub fn set_i(&mut self, i: bool) {
        let flags = self.flags;
        self.flags = if i {
            flags | 0b0000_0100
        } else {
            flags & 0b1111_1011
        }
    }

    pub fn set_d(&mut self, d: bool) {
        let flags = self.flags;
        self.flags = if d {
            flags | 0b0000_1000
        } else {
            flags & 0b1111_0111
        }
    }

    pub fn set_b(&mut self, b: bool) {
        let flags = self.flags;
        self.flags = if b {
            flags | 0b0001_0000
        } else {
            flags & 0b1110_1111
        }
    }

    pub fn set_v(&mut self, v: bool) {
        let flags = self.flags;
        self.flags = if v {
            flags | 0b0100_0000
        } else {
            flags & 0b1011_1111
        }
    }

    pub fn set_n(&mut self, n: bool) {
        let flags = self.flags;
        self.flags = if n {
            flags | 0b1000_0000
        } else {
            flags & 0b0111_1111
        }
    }

    pub fn pull_stack_u8(&mut self, bus: &mut Bus) -> Result<u8, Error> {
        self.sp += 1;
        let address = 0x0100 | self.sp as u16;
        let value = bus.read_u8(address);
        Ok(value)
    }

    /*
    SP=FD

    0x01FD
    0x01FE  BC
    0x01FF  AB

    pull_stack(SP)
    -> ABCD

    SP=FF

    0x01FD
    0x01FE
    0x01FF
    */
    pub fn pull_stack_u16(&mut self, bus: &mut Bus) -> Result<u16, Error> {
        self.sp += 2;
        let address = (0x0100 | self.sp as u16) - 1;
        let value = bus.read_u16(address);
        Ok(value)
    }

    pub fn push_stack_u8(&mut self, bus: &mut Bus, value: u8) {
        let address = 0x0100 | self.sp as u16;
        bus.write_u8(address, value);
        self.sp -= 1;
    }

    /*
    SP=FF

    0x01FD
    0x01FE
    0x01FF

    push_stack(SP, 0xABCD)
    ->

    SP=FD

    0x01FD
    0x01FE  CD
    0x01FF  AB
    */
    pub fn push_stack_u16(&mut self, bus: &mut Bus, value: u16) {
        let address = (0x0100 | self.sp as u16) - 1;
        bus.write_u16(address, value);
        self.sp -= 2;
    }
}

#[cfg(test)]
mod cpu_tests {
    use crate::Cpu;
    #[test]
    fn read_flag_functions_read_flags_correctly() {
        let mut cpu = Cpu::new();

        // Flags are initialized to false
        assert!(!cpu.c());
        assert!(!cpu.z());
        assert!(!cpu.i());
        assert!(!cpu.d());
        assert!(!cpu.b());
        assert!(!cpu.v());
        assert!(!cpu.n());

        cpu.flags = 0b1010_1010;

        assert!(!cpu.c());
        assert!(cpu.z());
        assert!(!cpu.i());
        assert!(cpu.d());
        assert!(!cpu.b());
        assert!(!cpu.v());
        assert!(cpu.n());
    }

    #[test]
    fn set_flag_functions_set_flags_correctly() {
        let mut cpu = Cpu::new();

        // Flags are initialized to false
        assert!(!cpu.c());
        assert!(!cpu.z());
        assert!(!cpu.i());
        assert!(!cpu.d());
        assert!(!cpu.b());
        assert!(!cpu.v());
        assert!(!cpu.n());

        cpu.set_c(true);
        cpu.set_z(true);
        cpu.set_i(true);
        cpu.set_d(true);
        cpu.set_b(true);
        cpu.set_v(true);
        cpu.set_n(true);

        assert!(cpu.c());
        assert!(cpu.z());
        assert!(cpu.i());
        assert!(cpu.d());
        assert!(cpu.b());
        assert!(cpu.v());
        assert!(cpu.n());
    }
}
