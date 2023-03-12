use std::{error, ops::Add};

use simple_logger::SimpleLogger;

const ROM_START: usize = 0x8000;

struct Cpu {
    pub pc: u16,
    pub sp: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            pc: ROM_START as u16,
            sp: 0,
            a: 0,
            x: 0,
            y: 0,
            p: 0,
        }
    }

    pub fn c(self) -> u8 {
        self.p & 0b0000_0001
    }

    pub fn z(self) -> u8 {
        self.p & 0b0000_0010
    }

    pub fn i(self) -> u8 {
        self.p & 0b0000_0100
    }

    pub fn d(self) -> u8 {
        self.p & 0b0000_1000
    }

    pub fn b(self) -> u8 {
        self.p & 0b0001_0000
    }

    pub fn v(self) -> u8 {
        self.p & 0b0100_0000
    }

    pub fn n(self) -> u8 {
        self.p & 0b1000_0000
    }

    pub fn set_c(&mut self, c: bool) {
        let flags = self.p;
        self.p = if c {
            flags | 0b0000_0001
        } else {
            flags & 0b1111_1110
        }
    }

    pub fn set_z(&mut self, z: bool) {
        let flags = self.p;
        self.p = if z {
            flags | 0b0000_0010
        } else {
            flags & 0b1111_1101
        }
    }

    pub fn set_i(&mut self, i: bool) {
        let flags = self.p;
        self.p = if i {
            flags | 0b0000_0100
        } else {
            flags & 0b1111_1011
        }
    }

    pub fn set_d(&mut self, d: bool) {
        let flags = self.p;
        self.p = if d {
            flags | 0b0000_1000
        } else {
            flags & 0b1111_0111
        }
    }

    pub fn set_b(&mut self, b: bool) {
        let flags = self.p;
        self.p = if b {
            flags | 0b0001_0000
        } else {
            flags & 0b1110_1111
        }
    }

    pub fn set_v(&mut self, v: bool) {
        let flags = self.p;
        self.p = if v {
            flags | 0b0100_0000
        } else {
            flags & 0b1011_1111
        }
    }

    pub fn set_n(&mut self, n: bool) {
        let flags = self.p;
        self.p = if n {
            flags | 0b1000_0000
        } else {
            flags & 0b0111_1111
        }
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
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Implied,
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
        Instruction::new(0xE8, "INX", AddressingMode::Implied),
        Instruction::new(0xAA, "TAX", AddressingMode::Implied),
        // Other addressing modes
        Instruction::new(0xA9, "LDA", AddressingMode::Immediate),
        Instruction::new(0xA5, "LDA", AddressingMode::ZeroPage),
        Instruction::new(0xB5, "LDA", AddressingMode::ZeroPageX),
        Instruction::new(0xAD, "LDA", AddressingMode::Absolute),
        Instruction::new(0xBD, "LDA", AddressingMode::AbsoluteX),
        Instruction::new(0xB9, "LDA", AddressingMode::AbsoluteY),
        Instruction::new(0xA1, "LDA", AddressingMode::IndirectX),
    ]
}

fn step(Console { cpu, memory }: &mut Console, opcodes: &Vec<Instruction>) -> Result<(), Error> {
    let opcode = memory.read_u8(cpu.pc)?;
    cpu.pc += 1;

    // Logs instruction name
    fn read_value(
        cpu: &mut Cpu,
        memory: &mut Memory,
        mode: AddressingMode,
        operation: &str,
    ) -> Result<u8, Error> {
        match mode {
            AddressingMode::Immediate => {
                let value = memory.read_u8(cpu.pc)?;
                cpu.pc += 1;
                log::info!("{} #{:X}", operation, value);

                Ok(value)
            }
            AddressingMode::ZeroPage => {
                let address = memory.read_u8(cpu.pc)?;
                cpu.pc += 1;
                log::info!("{} ${:X}", operation, address);

                memory.read_u8(address as u16)
            }
            AddressingMode::ZeroPageX => {
                let mut address = memory.read_u8(cpu.pc)?;
                cpu.pc += 1;
                log::info!("{} ${:X},X", operation, address);

                address = address.wrapping_add(cpu.x);
                memory.read_u8(address as u16)
            }
            AddressingMode::ZeroPageY => {
                let mut address = memory.read_u8(cpu.pc)?;
                cpu.pc += 1;
                log::info!("{} ${:X},Y", operation, address);

                address = address.wrapping_add(cpu.y);
                memory.read_u8(address as u16)
            }
            AddressingMode::Absolute => {
                let address = memory.read_u16(cpu.pc)?;
                cpu.pc += 2;
                log::info!("{} ${:X}", operation, address);

                memory.read_u8(address)
            }
            AddressingMode::AbsoluteX => {
                let mut address = memory.read_u16(cpu.pc)?;
                cpu.pc += 2;
                log::info!("{} ${:X},X", operation, address);

                address = address.wrapping_add(cpu.x as u16);
                memory.read_u8(address)
            }
            AddressingMode::AbsoluteY => {
                let mut address = memory.read_u16(cpu.pc)?;
                cpu.pc += 2;
                log::info!("{} ${:X},Y", operation, address);

                address = address.wrapping_add(cpu.y as u16);
                memory.read_u8(address)
            }
            AddressingMode::IndirectX => {
                let mut indirect_address = memory.read_u8(cpu.pc)?;
                cpu.pc += 1;
                log::info!("{} (${:X},X)", operation, indirect_address);

                // Read the final address from memory[indirect_address + x]
                indirect_address = indirect_address.wrapping_add(cpu.x);
                let address = memory.read_u16(indirect_address as u16)?;
                memory.read_u8(address)
            }
            AddressingMode::IndirectY => {
                let mut indirect_address = memory.read_u8(cpu.pc)?;
                cpu.pc += 1;
                log::info!("{} (${:X}),Y", operation, indirect_address);

                // The final address is (memory[indirect_address]) + y
                let mut address = memory.read_u16(indirect_address as u16)?;
                address = address.wrapping_add(cpu.y as u16);
                memory.read_u8(address)
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
        AddressingMode::Implied => {
            // Execute immediately.
            log::info!("{}", instruction.operation);

            match instruction.operation {
                "INX" => {
                    let value = cpu.x + 1;
                    cpu.x = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.set_z(zero);
                    cpu.set_n(negative);
                }
                "TAX" => {
                    let value = cpu.a;
                    cpu.x = value;

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
            let value = read_value(
                cpu,
                memory,
                instruction.addressing_mode,
                instruction.operation,
            )?;

            match instruction.operation {
                "LDA" => {
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
    }

    Ok(())
}

#[cfg(test)]
mod instruction_tests {
    use crate::{opcodes, step, Console, Cpu, Memory};

    #[test]
    fn lda_immediate() {
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
        step(&mut console, &opcodes);

        assert_eq!(console.cpu.pc, 0x8002);
        assert_eq!(console.cpu.a, 0xFF);
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
