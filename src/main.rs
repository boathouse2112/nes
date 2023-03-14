#![feature(bigint_helper_methods)]
use std::{error, ops::Add};

use simple_logger::SimpleLogger;

const ROM_START: usize = 0x8000;

struct Cpu {
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
            sp: 0,
            a: 0,
            x: 0,
            y: 0,
            flags: 0,
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

    pub fn pull_stack_u8(&mut self, memory: &mut Memory) -> Result<u8, Error> {
        let value = memory.read_u8(self.sp as u16)?;
        self.sp -= 1;
        Ok(value)
    }

    pub fn pull_stack_u16(&mut self, memory: &mut Memory) -> Result<u16, Error> {
        let value = memory.read_u16(self.sp as u16)?;
        self.sp -= 2;
        Ok(value)
    }

    pub fn push_stack_u8(&mut self, memory: &mut Memory, value: u8) {
        memory.write_u8(self.sp as u16, value);
        self.sp += 1;
    }

    pub fn push_stack_u16(&mut self, memory: &mut Memory, value: u16) {
        memory.write_u16(self.sp as u16, value);
        self.sp += 2;
    }
}

type Error = Box<dyn error::Error>;

struct Memory {
    memory: [u8; 0xFFFF],
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            memory: [0; 0xFFFF],
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.memory[ROM_START..(ROM_START + rom.len())].copy_from_slice(&rom[..]);
    }

    pub fn read_u8(&self, address: u16) -> Result<u8, Error> {
        self.memory
            .get(address as usize)
            .copied()
            .ok_or("Invalid address".into())
    }

    pub fn read_i8(&self, address: u16) -> Result<i8, Error> {
        self.memory
            .get(address as usize)
            .copied()
            .and_then(|n| Some(n as i8))
            .ok_or("Invalid address".into())
    }

    pub fn read_u16(&self, address: u16) -> Result<u16, Error> {
        let address = address as usize;
        let value_slice = self
            .memory
            .get(address..address + 2)
            .ok_or::<Error>("Invalid address".into())?;
        let value_le_bytes: [u8; 2] = value_slice.try_into()?;
        Ok(u16::from_le_bytes(value_le_bytes))
    }

    pub fn write_u8(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    pub fn write_i8(&mut self, address: u16, value: i8) {
        self.memory[address as usize] = value as u8;
    }

    pub fn write_u16(&mut self, address: u16, value: u16) {
        let address = address as usize;
        let little_endian = value.to_le_bytes();
        self.memory[address..address + 2].copy_from_slice(&little_endian);
    }
}

pub struct Console {
    cpu: Cpu,
    memory: Memory,
}

#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    None,
}

pub struct Instruction {
    opcode: u8,
    operation: &'static str,
    addressing_mode: AddressingMode,
}

impl Instruction {
    pub fn new(opcode: u8, operation: &'static str, addressing_mode: AddressingMode) -> Self {
        Instruction {
            opcode,
            operation,
            addressing_mode,
        }
    }
}

pub fn opcodes() -> Vec<Instruction> {
    vec![
        // Implied addressing mode
        Instruction::new(0x0A, "ASL", AddressingMode::None),
        Instruction::new(0x00, "BRK", AddressingMode::None),
        Instruction::new(0x18, "CLC", AddressingMode::None),
        Instruction::new(0xD8, "CLD", AddressingMode::None),
        Instruction::new(0x58, "CLI", AddressingMode::None),
        Instruction::new(0xB8, "CLV", AddressingMode::None),
        Instruction::new(0xCA, "DEX", AddressingMode::None),
        Instruction::new(0x88, "DEY", AddressingMode::None),
        Instruction::new(0xE8, "INX", AddressingMode::None),
        Instruction::new(0xC8, "INY", AddressingMode::None),
        Instruction::new(0xAA, "TAX", AddressingMode::None),
        Instruction::new(0x4A, "LSR", AddressingMode::None),
        Instruction::new(0xEA, "NOP", AddressingMode::None),
        Instruction::new(0x48, "PHA", AddressingMode::None),
        Instruction::new(0x08, "PHP", AddressingMode::None),
        Instruction::new(0x68, "PLA", AddressingMode::None),
        Instruction::new(0x28, "PLP", AddressingMode::None),
        Instruction::new(0x2A, "ROL", AddressingMode::None),
        Instruction::new(0x6A, "ROR", AddressingMode::None),
        Instruction::new(0x40, "RTI", AddressingMode::None),
        Instruction::new(0x60, "RTS", AddressingMode::None),
        Instruction::new(0x38, "SEC", AddressingMode::None),
        Instruction::new(0xF8, "SED", AddressingMode::None),
        Instruction::new(0x78, "SEI", AddressingMode::None),
        Instruction::new(0xAA, "TAX", AddressingMode::None),
        Instruction::new(0xA8, "TAY", AddressingMode::None),
        Instruction::new(0xBA, "TSX", AddressingMode::None),
        Instruction::new(0x8A, "TXA", AddressingMode::None),
        Instruction::new(0x9A, "TXS", AddressingMode::None),
        Instruction::new(0x98, "TYA", AddressingMode::None),
        // Other addressing modes
        //      ADC
        Instruction::new(0x69, "ADC", AddressingMode::Immediate),
        Instruction::new(0x65, "ADC", AddressingMode::ZeroPage),
        Instruction::new(0x75, "ADC", AddressingMode::ZeroPageX),
        Instruction::new(0x6D, "ADC", AddressingMode::Absolute),
        Instruction::new(0x7D, "ADC", AddressingMode::AbsoluteX),
        Instruction::new(0x79, "ADC", AddressingMode::AbsoluteY),
        Instruction::new(0x61, "ADC", AddressingMode::IndirectX),
        Instruction::new(0x71, "ADC", AddressingMode::IndirectY),
        //      ASL
        Instruction::new(0x06, "ASL", AddressingMode::ZeroPage),
        Instruction::new(0x16, "ASL", AddressingMode::ZeroPageX),
        Instruction::new(0x0E, "ASL", AddressingMode::Absolute),
        Instruction::new(0x1E, "ASL", AddressingMode::AbsoluteX),
        //      AND
        Instruction::new(0x29, "AND", AddressingMode::Immediate),
        Instruction::new(0x25, "AND", AddressingMode::ZeroPage),
        Instruction::new(0x35, "AND", AddressingMode::ZeroPageX),
        Instruction::new(0x2D, "AND", AddressingMode::Absolute),
        Instruction::new(0x3D, "AND", AddressingMode::AbsoluteX),
        Instruction::new(0x39, "AND", AddressingMode::AbsoluteY),
        Instruction::new(0x21, "AND", AddressingMode::IndirectX),
        Instruction::new(0x31, "AND", AddressingMode::IndirectY),
        //      BCC
        Instruction::new(0x90, "BCC", AddressingMode::Relative),
        //      BCS
        Instruction::new(0xB0, "BCS", AddressingMode::Relative),
        //      BEQ
        Instruction::new(0xF0, "BEQ", AddressingMode::Relative),
        //      BIT
        Instruction::new(0x24, "BIT", AddressingMode::ZeroPage),
        Instruction::new(0x2C, "BIT", AddressingMode::Absolute),
        //      BMI
        Instruction::new(0x30, "BMI", AddressingMode::Relative),
        //      BNE
        Instruction::new(0xD0, "BEQ", AddressingMode::Relative),
        //      BPL
        Instruction::new(0x10, "BPL", AddressingMode::Relative),
        //      BVC
        Instruction::new(0x50, "BVC", AddressingMode::Relative),
        //      BVS
        Instruction::new(0x70, "BVS", AddressingMode::Relative),
        //      CMP
        Instruction::new(0xC9, "CMP", AddressingMode::Immediate),
        Instruction::new(0xC5, "CMP", AddressingMode::ZeroPage),
        Instruction::new(0xD5, "CMP", AddressingMode::ZeroPageX),
        Instruction::new(0xCD, "CMP", AddressingMode::Absolute),
        Instruction::new(0xDD, "CMP", AddressingMode::AbsoluteX),
        Instruction::new(0xD9, "CMP", AddressingMode::AbsoluteY),
        Instruction::new(0xC1, "CMP", AddressingMode::IndirectX),
        Instruction::new(0xD1, "CMP", AddressingMode::IndirectY),
        //      CPX
        Instruction::new(0xE0, "CPX", AddressingMode::Immediate),
        Instruction::new(0xE4, "CPX", AddressingMode::ZeroPage),
        Instruction::new(0xEC, "CPX", AddressingMode::Absolute),
        //      CPY
        Instruction::new(0xC0, "CPY", AddressingMode::Immediate),
        Instruction::new(0xC4, "CPY", AddressingMode::ZeroPage),
        Instruction::new(0xCC, "CPY", AddressingMode::Absolute),
        //      DEC
        Instruction::new(0xC0, "DEC", AddressingMode::ZeroPage),
        Instruction::new(0xC4, "DEC", AddressingMode::ZeroPageX),
        Instruction::new(0xCC, "DEC", AddressingMode::Absolute),
        Instruction::new(0xCC, "DEC", AddressingMode::AbsoluteX),
        //      EOR
        Instruction::new(0x49, "EOR", AddressingMode::Immediate),
        Instruction::new(0x45, "EOR", AddressingMode::ZeroPage),
        Instruction::new(0x55, "EOR", AddressingMode::ZeroPageX),
        Instruction::new(0x4D, "EOR", AddressingMode::Absolute),
        Instruction::new(0x5D, "EOR", AddressingMode::AbsoluteX),
        Instruction::new(0x59, "EOR", AddressingMode::AbsoluteY),
        Instruction::new(0x41, "EOR", AddressingMode::IndirectX),
        Instruction::new(0x51, "EOR", AddressingMode::IndirectY),
        //      INC
        Instruction::new(0xE6, "INC", AddressingMode::ZeroPage),
        Instruction::new(0xF6, "INC", AddressingMode::ZeroPageX),
        Instruction::new(0xEE, "INC", AddressingMode::Absolute),
        Instruction::new(0xFE, "INC", AddressingMode::AbsoluteX),
        //      JMP
        Instruction::new(0x4C, "JMP", AddressingMode::Absolute),
        Instruction::new(0x6C, "JMP", AddressingMode::Indirect),
        //      JSR
        Instruction::new(0x20, "JSR", AddressingMode::Absolute),
        //      LDA
        Instruction::new(0xA9, "LDA", AddressingMode::Immediate),
        Instruction::new(0xA5, "LDA", AddressingMode::ZeroPage),
        Instruction::new(0xB5, "LDA", AddressingMode::ZeroPageX),
        Instruction::new(0xAD, "LDA", AddressingMode::Absolute),
        Instruction::new(0xBD, "LDA", AddressingMode::AbsoluteX),
        Instruction::new(0xB9, "LDA", AddressingMode::AbsoluteY),
        Instruction::new(0xA1, "LDA", AddressingMode::IndirectX),
        Instruction::new(0xB1, "LDA", AddressingMode::IndirectY),
        //      LDX
        Instruction::new(0xA2, "LDX", AddressingMode::Immediate),
        Instruction::new(0xA6, "LDX", AddressingMode::ZeroPage),
        Instruction::new(0xB6, "LDX", AddressingMode::ZeroPageY),
        Instruction::new(0xAE, "LDX", AddressingMode::Absolute),
        Instruction::new(0xBE, "LDX", AddressingMode::AbsoluteY),
        //      LDY
        Instruction::new(0xA0, "LDY", AddressingMode::Immediate),
        Instruction::new(0xA4, "LDY", AddressingMode::ZeroPage),
        Instruction::new(0xB4, "LDY", AddressingMode::ZeroPageX),
        Instruction::new(0xAC, "LDY", AddressingMode::Absolute),
        Instruction::new(0xBC, "LDY", AddressingMode::AbsoluteX),
        //      LDY
        Instruction::new(0xA4, "LDY", AddressingMode::ZeroPage),
        Instruction::new(0xB4, "LDY", AddressingMode::ZeroPageX),
        Instruction::new(0xAC, "LDY", AddressingMode::Absolute),
        Instruction::new(0xBC, "LDY", AddressingMode::AbsoluteX),
        //      LSR
        Instruction::new(0x46, "LSR", AddressingMode::ZeroPage),
        Instruction::new(0x56, "LSR", AddressingMode::ZeroPageX),
        Instruction::new(0x4E, "LSR", AddressingMode::Absolute),
        Instruction::new(0x5E, "LSR", AddressingMode::AbsoluteX),
        //      ORA
        Instruction::new(0x09, "ORA", AddressingMode::Immediate),
        Instruction::new(0x05, "ORA", AddressingMode::ZeroPage),
        Instruction::new(0x15, "ORA", AddressingMode::ZeroPageX),
        Instruction::new(0x0D, "ORA", AddressingMode::Absolute),
        Instruction::new(0x1D, "ORA", AddressingMode::AbsoluteX),
        Instruction::new(0x19, "ORA", AddressingMode::AbsoluteY),
        Instruction::new(0x01, "ORA", AddressingMode::IndirectX),
        Instruction::new(0x11, "ORA", AddressingMode::IndirectY),
        //      ROL
        Instruction::new(0x26, "ROL", AddressingMode::ZeroPage),
        Instruction::new(0x36, "ROL", AddressingMode::ZeroPageX),
        Instruction::new(0x2E, "ROL", AddressingMode::Absolute),
        Instruction::new(0x3E, "ROL", AddressingMode::AbsoluteX),
        //      ROR
        Instruction::new(0x66, "ROR", AddressingMode::ZeroPage),
        Instruction::new(0x76, "ROR", AddressingMode::ZeroPageX),
        Instruction::new(0x6E, "ROR", AddressingMode::Absolute),
        Instruction::new(0x7E, "ROR", AddressingMode::AbsoluteX),
        //      SBC
        Instruction::new(0xE9, "SBC", AddressingMode::Immediate),
        Instruction::new(0xE5, "SBC", AddressingMode::ZeroPage),
        Instruction::new(0xF5, "SBC", AddressingMode::ZeroPageX),
        Instruction::new(0xED, "SBC", AddressingMode::Absolute),
        Instruction::new(0xFD, "SBC", AddressingMode::AbsoluteX),
        Instruction::new(0xF9, "SBC", AddressingMode::AbsoluteY),
        Instruction::new(0xE1, "SBC", AddressingMode::IndirectX),
        Instruction::new(0xF1, "SBC", AddressingMode::IndirectY),
        //      STA
        Instruction::new(0x85, "STA", AddressingMode::ZeroPage),
        Instruction::new(0x95, "STA", AddressingMode::ZeroPageX),
        Instruction::new(0x8D, "STA", AddressingMode::Absolute),
        Instruction::new(0x9D, "STA", AddressingMode::AbsoluteX),
        Instruction::new(0x99, "STA", AddressingMode::AbsoluteY),
        Instruction::new(0x81, "STA", AddressingMode::IndirectX),
        Instruction::new(0x91, "STA", AddressingMode::IndirectY),
        //      STX
        Instruction::new(0x86, "STX", AddressingMode::ZeroPage),
        Instruction::new(0x96, "STX", AddressingMode::ZeroPageY),
        Instruction::new(0x8E, "STX", AddressingMode::Absolute),
        //      STY
        Instruction::new(0x84, "STY", AddressingMode::ZeroPage),
        Instruction::new(0x94, "STY", AddressingMode::ZeroPageX),
        Instruction::new(0x8C, "STY", AddressingMode::Absolute),
    ]
}

fn step(Console { cpu, memory }: &mut Console, opcodes: &Vec<Instruction>) -> Result<(), Error> {
    let opcode = memory.read_u8(cpu.pc)?;
    cpu.pc += 1;

    // Logs instruction name
    fn read_address(
        cpu: &mut Cpu,
        memory: &mut Memory,
        mode: AddressingMode,
        operation: &str,
    ) -> Result<u16, Error> {
        match mode {
            AddressingMode::Immediate => {
                let address = cpu.pc;
                let value = memory.read_u8(address)?;
                cpu.pc += 1;
                log::info!("{} #{:X}", operation, value);

                Ok(address)
            }
            AddressingMode::ZeroPage => {
                let address = memory.read_u8(cpu.pc)?;
                cpu.pc += 1;
                log::info!("{} ${:X}", operation, address);

                Ok(address as u16)
            }
            AddressingMode::ZeroPageX => {
                let mut address = memory.read_u8(cpu.pc)?;
                cpu.pc += 1;
                log::info!("{} ${:X},X", operation, address);

                address = address.wrapping_add(cpu.x);
                Ok(address as u16)
            }
            AddressingMode::ZeroPageY => {
                let mut address = memory.read_u8(cpu.pc)?;
                cpu.pc += 1;
                log::info!("{} ${:X},Y", operation, address);

                address = address.wrapping_add(cpu.y);
                Ok(address as u16)
            }
            AddressingMode::Relative => {
                let address = cpu.pc;
                let value = memory.read_i8(address)?;
                cpu.pc += 1;
                log::info!("{} #{:+X}", operation, value);

                Ok(address)
            }
            AddressingMode::Absolute => {
                let address = memory.read_u16(cpu.pc)?;
                cpu.pc += 2;
                log::info!("{} ${:X}", operation, address);

                Ok(address)
            }
            AddressingMode::AbsoluteX => {
                let mut address = memory.read_u16(cpu.pc)?;
                cpu.pc += 2;
                log::info!("{} ${:X},X", operation, address);

                address = address.wrapping_add(cpu.x as u16);
                Ok(address)
            }
            AddressingMode::AbsoluteY => {
                let mut address = memory.read_u16(cpu.pc)?;
                cpu.pc += 2;
                log::info!("{} ${:X},Y", operation, address);

                address = address.wrapping_add(cpu.y as u16);
                Ok(address)
            }
            AddressingMode::Indirect => {
                let indirect_address = memory.read_u16(cpu.pc)?;
                cpu.pc += 2;
                log::info!("{} (${:X})", operation, indirect_address);

                let address = memory.read_u16(indirect_address)?;
                Ok(address)
            }
            AddressingMode::IndirectX => {
                let mut indirect_address = memory.read_u8(cpu.pc)?;
                cpu.pc += 1;
                log::info!("{} (${:X},X)", operation, indirect_address);

                // Read the final address from memory[indirect_address + x]
                indirect_address = indirect_address.wrapping_add(cpu.x);
                let address = memory.read_u16(indirect_address as u16)?;
                Ok(address)
            }
            AddressingMode::IndirectY => {
                let indirect_address = memory.read_u8(cpu.pc)?;
                cpu.pc += 1;
                log::info!("{} (${:X}),Y", operation, indirect_address);

                // The final address is (memory[indirect_address]) + y
                let mut address = memory.read_u16(indirect_address as u16)?;
                address = address.wrapping_add(cpu.y as u16);
                Ok(address)
            }
            _ => {
                panic!()
            }
        }
    }

    let instruction = opcodes
        .iter()
        .find(|&instruction| instruction.opcode == opcode)
        .unwrap();

    match instruction.addressing_mode {
        AddressingMode::None => {
            // Execute immediately.
            log::info!("{}", instruction.operation);

            match instruction.operation {
                "ASL" => {
                    let value = cpu.a;
                    let result = value << 1;
                    cpu.a = result;

                    let carry = (value & 0b1000_0000) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_c(carry);
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "BRK" => {
                    todo!("PC and processor status are pushed to stack?");
                    // cpu.set_b(true);
                }
                "CLC" => {
                    // Clear carry flag
                    cpu.set_c(false);
                }
                "CLD" => {
                    // Clear carry flag
                    cpu.set_d(false);
                }
                "CLI" => {
                    // Clear carry flag
                    cpu.set_i(false);
                }
                "CLV" => {
                    // Clear carry flag
                    cpu.set_v(false);
                }
                "DEX" => {
                    // Decrement X
                    let value = cpu.x;
                    let result = value - 1;
                    cpu.x = result;

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "DEY" => {
                    // Decrement Y
                    let value = cpu.y;
                    let result = value - 1;
                    cpu.y = result;

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "INX" => {
                    let result = cpu.x + 1;
                    cpu.x = result;

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "INY" => {
                    let result = cpu.y + 1;
                    cpu.y = result;

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "LSR" => {
                    let value = cpu.a;
                    let result = value >> 1;
                    cpu.a = result;

                    let carry = (value & 1) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_c(carry);
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "PHA" => {
                    // Push A to stack
                    cpu.push_stack_u8(memory, cpu.a);
                }
                "PHP" => {
                    // Push flags to stack
                    cpu.push_stack_u8(memory, cpu.flags);
                }
                "PLA" => {
                    // Pull stack to A
                    let value = cpu.pull_stack_u8(memory)?;
                    cpu.a = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "PLP" => {
                    // Pull stack to flags
                    cpu.flags = cpu.pull_stack_u8(memory)?;
                }
                "ROL" => {
                    // Rotate A left
                    // result_carry <- [7..0] <- carry
                    let value = cpu.a;
                    let carry_mask = if cpu.c() { 1 } else { 0 };
                    let result = (value << 1) | carry_mask;
                    cpu.a = result;

                    let carry = (value & 0b1000_0000) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_c(carry);
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "ROR" => {
                    // Rotate A right
                    // carry -> [7..0] -> result_carry
                    let value = cpu.a;
                    let carry_mask = if cpu.c() { 0b1000_0000 } else { 0 };
                    let result = (value >> 1) | carry_mask;
                    cpu.a = result;

                    let carry = (value & 1) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_c(carry);
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "RTI" => {
                    cpu.flags = cpu.pull_stack_u8(memory)?;
                    cpu.pc = cpu.pull_stack_u16(memory)?;
                }
                "RTS" => {
                    cpu.pc = cpu.pull_stack_u16(memory)?;
                }
                "SEC" => {
                    cpu.set_c(true);
                }
                "SED" => {
                    cpu.set_d(true);
                }
                "SEI" => {
                    cpu.set_i(true);
                }
                "TAX" => {
                    let value = cpu.a;
                    cpu.x = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "TAY" => {
                    let value = cpu.a;
                    cpu.y = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "TSX" => {
                    let value = cpu.sp;
                    cpu.x = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "TXA" => {
                    let value = cpu.x;
                    cpu.a = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "TXS" => {
                    cpu.sp = cpu.x;
                }
                "TYA" => {
                    let value = cpu.y;
                    cpu.a = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                operation => {
                    todo!("{:?}", operation)
                }
            }
        }
        // Load a value based on the addressing mode, and then execute
        _ => {
            let address = read_address(
                cpu,
                memory,
                instruction.addressing_mode,
                instruction.operation,
            )?;

            match instruction.operation {
                "ADC" => {
                    let acc_value = cpu.a;
                    let memory_value = memory.read_u8(address)?;
                    let carry = cpu.c();

                    let (result, result_carry) = acc_value.carrying_add(memory_value, carry);
                    cpu.a = result;

                    let should_be_negative = (acc_value as i32)
                        .carrying_add(memory_value as i32, carry)
                        .0
                        < 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    let overflow = negative != should_be_negative;
                    cpu.set_c(result_carry);
                    cpu.set_z(zero);
                    cpu.set_v(overflow);
                    cpu.set_n(negative);
                }
                "AND" => {
                    let value = memory.read_u8(address)?;
                    let acc = cpu.a;
                    let result = acc & value;
                    cpu.a = result;

                    let zero = result == 0;
                    let negative = (value as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "ASL" => {
                    // Shift bits left 1
                    let value = memory.read_u8(address)?;
                    let result = value << 1;
                    memory.write_u8(address, result);

                    let carry = (value & 0b1000_0000) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_c(carry);
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "BCC" => {
                    // Branch (add offset to pc) if carry flag is clear
                    let offset = memory.read_i8(address)?;
                    if !cpu.c() {
                        cpu.pc = (cpu.pc as i16 + offset as i16) as u16
                    }
                }
                "BCS" => {
                    // Branch if carry flag is set
                    let offset = memory.read_i8(address)?;
                    if cpu.c() {
                        cpu.pc = (cpu.pc as i16 + offset as i16) as u16
                    }
                }
                "BEQ" => {
                    // Branch if zero flag is set
                    let offset = memory.read_i8(address)?;
                    if cpu.z() {
                        cpu.pc = (cpu.pc as i16 + offset as i16) as u16
                    }
                }
                "BIT" => {
                    // Set zero flag to (A AND value) == 0
                    let value = memory.read_u8(address)?;
                    let result = cpu.a & value;

                    let zero = result == 0;
                    let overflow = (result & 0b0100_0000) != 0; // Yeah, overflow is set to bit 6. Idk.
                    let negative = (result & 0b1000_0000) != 0;
                    cpu.set_z(zero);
                    cpu.set_v(overflow);
                    cpu.set_n(negative);
                }
                "BMI" => {
                    // Branch if negative flag is set
                    let offset = memory.read_i8(address)?;
                    if cpu.n() {
                        cpu.pc = (cpu.pc as i16 + offset as i16) as u16
                    }
                }
                "BNE" => {
                    // Branch if zero flag is clear
                    let offset = memory.read_i8(address)?;
                    if !cpu.z() {
                        cpu.pc = (cpu.pc as i16 + offset as i16) as u16
                    }
                }
                "BPL" => {
                    // Branch if negative flag is clear
                    let offset = memory.read_i8(address)?;
                    if !cpu.n() {
                        cpu.pc = (cpu.pc as i16 + offset as i16) as u16
                    }
                }
                "BVC" => {
                    // Branch if overflow flag is clear
                    let offset = memory.read_i8(address)?;
                    if !cpu.v() {
                        cpu.pc = (cpu.pc as i16 + offset as i16) as u16
                    }
                }
                "BVS" => {
                    // Branch if overflow flag is set
                    let offset = memory.read_i8(address)?;
                    if cpu.n() {
                        cpu.pc = (cpu.pc as i16 + offset as i16) as u16
                    }
                }
                "CMP" => {
                    // Set flags based on A - M
                    let acc = cpu.a;
                    let value = memory.read_u8(address)?;
                    let result = acc - value;

                    let carry = acc > value;
                    let zero = acc == value;
                    let negative = (result as i8) < 0;
                    cpu.set_c(carry);
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "CPX" => {
                    // Set flags based on A - M
                    let x = cpu.x;
                    let value = memory.read_u8(address)?;
                    let result = x - value;

                    let carry = x > value;
                    let zero = x == value;
                    let negative = (result as i8) < 0;
                    cpu.set_c(carry);
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "CPY" => {
                    // Set flags based on A - M
                    let y = cpu.y;
                    let value = memory.read_u8(address)?;
                    let result = y - value;

                    let carry = y > value;
                    let zero = y == value;
                    let negative = (result as i8) < 0;
                    cpu.set_c(carry);
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "DEC" => {
                    // Decrement memory
                    let value = memory.read_u8(address)?;
                    let result = value - 1;
                    memory.write_u8(address, result);

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "EOR" => {
                    // A ^ M
                    let acc = cpu.a;
                    let value = memory.read_u8(address)?;
                    let result = acc ^ value;
                    cpu.a = result;

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "INC" => {
                    // Increment memory
                    let value = memory.read_u8(address)?;
                    let result = value + 1;
                    memory.write_u8(address, result);

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "JMP" => {
                    // Jump to location
                    let value = memory.read_u16(address)?;
                    cpu.pc = value;
                }
                "JSR" => {
                    // Jump to subroutine. Push PC to the stack, and jump to address
                    cpu.push_stack_u16(memory, cpu.pc);
                    cpu.pc = address;
                }
                "LDA" => {
                    // Load value to a register
                    let value = memory.read_u8(address)?;
                    cpu.a = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "LDX" => {
                    // Load value to a register
                    let value = memory.read_u8(address)?;
                    cpu.x = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "LDY" => {
                    // Load value to a register
                    let value = memory.read_u8(address)?;
                    cpu.y = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "LSR" => {
                    let value = memory.read_u8(address)?;
                    let result = value >> 1;
                    memory.write_u8(address, result);

                    let carry = (value & 1) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_c(carry);
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "ORA" => {
                    let value = memory.read_u8(address)?;
                    let result = cpu.a | value;

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "ROL" => {
                    // Rotate A left
                    // result_carry <- [7..0] <- carry
                    let value = memory.read_u8(address)?;
                    let carry_mask = if cpu.c() { 1 } else { 0 };
                    let result = (value << 1) | carry_mask;
                    memory.write_u8(address, result);

                    let carry = (value & 0b1000_0000) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_c(carry);
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "ROR" => {
                    // Rotate A right
                    // carry -> [7..0] -> result_carry
                    let value = memory.read_u8(address)?;
                    let carry_mask = if cpu.c() { 0b1000_0000 } else { 0 };
                    let result = (value >> 1) | carry_mask;
                    memory.write_u8(address, result);

                    let carry = (value & 1) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.set_c(carry);
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "SBC" => {
                    let acc_value = cpu.a;
                    let memory_value = memory.read_u8(address)?;
                    let carry = cpu.c();

                    let (result, result_carry) = acc_value.borrowing_sub(memory_value, !carry);
                    cpu.a = result;

                    let should_be_negative = (acc_value as i32)
                        .borrowing_sub(memory_value as i32, !carry)
                        .0
                        < 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    let overflow = should_be_negative != negative;
                    cpu.set_c(result_carry);
                    cpu.set_z(zero);
                    cpu.set_v(overflow);
                    cpu.set_n(negative);
                }
                "STA" => {
                    // Store A to memory
                    memory.write_u8(address, cpu.a);
                }
                "STX" => {
                    // Store X to memory
                    memory.write_u8(address, cpu.x);
                }
                "STY" => {
                    // Store Y to memory
                    memory.write_u8(address, cpu.y);
                }
                operation => {
                    todo!("{:?}", operation)
                }
            }
        }
    }

    Ok(())
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

#[cfg(test)]
mod instruction_tests {
    use crate::{opcodes, step, Console, Cpu, Error, Memory};

    #[test]
    fn lda_immediate_loads_immediate_value() -> Result<(), Error> {
        let mut console = Console {
            cpu: Cpu::new(),
            memory: Memory::new(),
        };
        let opcodes = opcodes();
        let program = vec![
            0xA9, // LDA #$FF     $8000
            0xFF,
        ];

        // Running 1 step should move PC forward 2, and load the immediate value.
        console.memory.load_rom(program);
        step(&mut console, &opcodes)?;

        assert_eq!(console.cpu.pc, 0x8002);
        assert_eq!(console.cpu.a, 0xFF);
        Ok(())
    }

    #[test]
    fn lda_zero_page_loads_from_memory() -> Result<(), Error> {
        let mut console = Console {
            cpu: Cpu::new(),
            memory: Memory::new(),
        };
        let opcodes = opcodes();
        let program = vec![
            0xA5, // LDA $FF     ;$8000
            0xFF,
        ];

        console.memory.write_u8(0xFF, 0xAB); // Set the value to read from $FF
        console.memory.load_rom(program);

        step(&mut console, &opcodes)?;

        assert_eq!(console.cpu.pc, 0x8002);
        assert_eq!(console.cpu.a, 0xAB);
        Ok(())
    }

    #[test]
    fn lda_zero_page_x_loads_from_memory() -> Result<(), Error> {
        let mut console = Console {
            cpu: Cpu::new(),
            memory: Memory::new(),
        };
        let opcodes = opcodes();
        let program = vec![
            0xB5, // LDA $F0,X     ;$8000
            0xF0,
        ];

        console.cpu.x = 0x08; // Set X offset
        console.memory.write_u8(0xF8, 0xAB); // Set the value to read from $F0 + x
        console.memory.load_rom(program);

        step(&mut console, &opcodes)?;

        assert_eq!(console.cpu.pc, 0x8002);
        assert_eq!(console.cpu.a, 0xAB);
        Ok(())
    }

    #[test]
    fn lda_zero_page_x_wraps_if_greater_than_0xff() -> Result<(), Error> {
        let mut console = Console {
            cpu: Cpu::new(),
            memory: Memory::new(),
        };
        let opcodes = opcodes();
        let program = vec![
            0xB5, // LDA $F0,X     ;$8000
            0xFF,
        ];

        console.cpu.x = 0x10; // Set X offset
        console.memory.write_u8(0x0F, 0xAB); // Set the value to read from $FF + x
        console.memory.load_rom(program);

        step(&mut console, &opcodes)?;

        assert_eq!(console.cpu.pc, 0x8002);
        assert_eq!(console.cpu.a, 0xAB);
        Ok(())
    }

    #[test]
    fn lda_absolute_loads_from_memory() -> Result<(), Error> {
        let mut console = Console {
            cpu: Cpu::new(),
            memory: Memory::new(),
        };
        let opcodes = opcodes();
        let program = vec![
            0xAD, // LDA $ABCD     ;$8000
            0xCD, //
            0xAB, //
        ];

        console.memory.write_u8(0xABCD, 0x11); // Set the value to read from $ABCD
        console.memory.load_rom(program);

        step(&mut console, &opcodes)?;

        assert_eq!(console.cpu.pc, 0x8003);
        assert_eq!(console.cpu.a, 0x11);
        Ok(())
    }

    #[test]
    fn lda_absolute_x_loads_from_memory() -> Result<(), Error> {
        let mut console = Console {
            cpu: Cpu::new(),
            memory: Memory::new(),
        };
        let opcodes = opcodes();
        let program = vec![
            0xBD, // LDA $F000,X     ;$8000
            0x00, //
            0xF0, //
        ];

        console.cpu.x = 0x22;
        console.memory.write_u8(0xF022, 0xAB); // Set the value to read from $F000 + 0x22
        console.memory.load_rom(program);

        step(&mut console, &opcodes)?;

        assert_eq!(console.cpu.pc, 0x8003);
        assert_eq!(console.cpu.a, 0xAB);
        Ok(())
    }

    #[test]
    fn lda_absolute_y_loads_from_memory() -> Result<(), Error> {
        let mut console = Console {
            cpu: Cpu::new(),
            memory: Memory::new(),
        };
        let opcodes = opcodes();
        let program = vec![
            0xB9, // LDA $F000,Y     ;$8000
            0x00, //
            0xF0, //
        ];

        console.cpu.y = 0x22;
        console.memory.write_u8(0xF022, 0xAB); // Set the value to read from $F000 + 0x22
        console.memory.load_rom(program);

        step(&mut console, &opcodes)?;

        assert_eq!(console.cpu.pc, 0x8003);
        assert_eq!(console.cpu.a, 0xAB);
        Ok(())
    }

    #[test]
    fn lda_indirect_x_loads_from_memory() -> Result<(), Error> {
        let mut console = Console {
            cpu: Cpu::new(),
            memory: Memory::new(),
        };
        let opcodes = opcodes();
        let program = vec![
            0xA1, // LDA ($20,X)     ;$8000
            0x20,
        ];

        console.cpu.x = 0x08;
        console.memory.write_u8(0x28, 0xAB); // Set the destination to read from ($20 + 0x08)
        console.memory.write_u8(0xAB, 0xCD);
        console.memory.load_rom(program);

        step(&mut console, &opcodes)?;

        assert_eq!(console.cpu.pc, 0x8002);
        assert_eq!(console.cpu.a, 0xCD);
        Ok(())
    }

    #[test]
    fn lda_indirect_y_loads_from_memory() -> Result<(), Error> {
        let mut console = Console {
            cpu: Cpu::new(),
            memory: Memory::new(),
        };
        let opcodes = opcodes();
        let program = vec![
            0xB1, // LDA ($20),Y     ;$8000
            0x20,
        ];

        console.cpu.y = 0x08;
        console.memory.write_u8(0x20, 0xF0); // Set the destination to read from $20
        console.memory.write_u8(0xF8, 0xCD); // Set the value to read from $F0 + 0x08
        console.memory.load_rom(program);

        step(&mut console, &opcodes)?;

        assert_eq!(console.cpu.pc, 0x8002);
        assert_eq!(console.cpu.a, 0xCD);
        Ok(())
    }

    #[test]
    fn lda_sets_flags_correctly() -> Result<(), Error> {
        let mut console = Console {
            cpu: Cpu::new(),
            memory: Memory::new(),
        };
        let opcodes = opcodes();
        let program = vec![
            0xA9, // LDA #$20
            0x20, //
            0xA9, // LDA #$00
            0x00, //
            0xA9, // LDA #$(-10 as u8)
            ((-10 as i8) as u8),
        ];

        console.memory.load_rom(program);

        step(&mut console, &opcodes)?;
        assert_eq!(console.cpu.pc, 0x8002);
        assert_eq!(console.cpu.a, 0x20);
        assert!(!console.cpu.z()); // Not zero
        assert!(!console.cpu.n()); // Not negative

        step(&mut console, &opcodes)?;
        assert_eq!(console.cpu.pc, 0x8004);
        assert_eq!(console.cpu.a, 0x00);
        assert!(console.cpu.z()); // Is zero
        assert!(!console.cpu.n()); // Not negative

        step(&mut console, &opcodes)?;
        assert_eq!(console.cpu.pc, 0x8006);
        assert_eq!(console.cpu.a, ((-10 as i8) as u8));
        assert!(!console.cpu.z()); // Not Zero
        assert!(console.cpu.n()); // Is negative

        Ok(())
    }

    #[test]
    fn asl_works() -> Result<(), Error> {
        let mut console = Console {
            cpu: Cpu::new(),
            memory: Memory::new(),
        };
        let opcodes = opcodes();
        let program = vec![
            0x0A, // ASL A
            0x06, // ASL $20
            0x20, //
        ];

        console.cpu.a = 0b1000_0000;
        console.memory.write_u8(0x20, 0b0100_0000);
        console.memory.load_rom(program);

        step(&mut console, &opcodes)?;
        assert_eq!(console.cpu.a, 0);
        assert!(console.cpu.c());
        assert!(console.cpu.z());
        assert!(!console.cpu.n());

        step(&mut console, &opcodes)?;
        let result = console.memory.read_u8(0x20)?;
        assert_eq!(result, 0b1000_0000);
        assert!(!console.cpu.c());
        assert!(!console.cpu.z());
        assert!(console.cpu.n());
        Ok(())
    }
}

fn main() {
    SimpleLogger::new().init().unwrap();

    let program = vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00];
    let opcodes = opcodes();
    let mut console = Console {
        cpu: Cpu::new(),
        memory: Memory::new(),
    };

    console.memory.load_rom(program);
    step(&mut console, &opcodes);

    println!("Hello, world!");
}
