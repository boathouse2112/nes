use crate::{
    bus::Bus,
    config::{CPU_FLAG_START_VALUE, CPU_SP_START_VALUE, ROM_START},
    util::Error,
};
use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Flags: u8 {
        const NEGATIVE          = 0b1000_0000;
        const OVERFLOW          = 0b0100_0000;
        const BREAK             = 0b0010_0000;
        const BREAK_2           = 0b0001_0000;
        const DECIMAL           = 0b0000_1000;
        const INTERRUPT         = 0b0000_0100;
        const ZERO              = 0b0000_0010;
        const CARRY             = 0b0000_0001;
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Cpu {
    pub pc: u16,
    pub sp: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub flags: Flags,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            pc: ROM_START as u16,
            sp: CPU_SP_START_VALUE,
            a: 0,
            x: 0,
            y: 0,
            flags: Flags::from_bits_retain(CPU_FLAG_START_VALUE),
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
