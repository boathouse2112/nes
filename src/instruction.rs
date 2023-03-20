use bitflags::BitFlags;

use crate::{
    bus::Bus,
    console::Console,
    cpu::{Cpu, Flags},
    util::Error,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Instruction {
    pub opcode: u8,
    pub operation: &'static str,
    pub addressing_mode: AddressingMode,
    pub bytes: u8,
    pub cycles: u8,
}

impl Instruction {
    pub fn new(
        opcode: u8,
        operation: &'static str,
        addressing_mode: AddressingMode,
        bytes: u8,
        cycles: u8,
    ) -> Self {
        Instruction {
            opcode,
            operation,
            addressing_mode,
            bytes,
            cycles,
        }
    }
}

pub fn instructions() -> Vec<Instruction> {
    vec![
        // Implied addressing mode
        Instruction::new(0x0A, "ASL", AddressingMode::None, 1, 2),
        Instruction::new(0x00, "BRK", AddressingMode::None, 1, 7),
        Instruction::new(0x18, "CLC", AddressingMode::None, 1, 2),
        Instruction::new(0xD8, "CLD", AddressingMode::None, 1, 2),
        Instruction::new(0x58, "CLI", AddressingMode::None, 1, 2),
        Instruction::new(0xB8, "CLV", AddressingMode::None, 1, 2),
        Instruction::new(0xCA, "DEX", AddressingMode::None, 1, 2),
        Instruction::new(0x88, "DEY", AddressingMode::None, 1, 2),
        Instruction::new(0xE8, "INX", AddressingMode::None, 1, 2),
        Instruction::new(0xC8, "INY", AddressingMode::None, 1, 2),
        Instruction::new(0x4A, "LSR", AddressingMode::None, 1, 2),
        Instruction::new(0xEA, "NOP", AddressingMode::None, 1, 2),
        Instruction::new(0x48, "PHA", AddressingMode::None, 1, 3),
        Instruction::new(0x08, "PHP", AddressingMode::None, 1, 3),
        Instruction::new(0x68, "PLA", AddressingMode::None, 1, 4),
        Instruction::new(0x28, "PLP", AddressingMode::None, 1, 4),
        Instruction::new(0x2A, "ROL", AddressingMode::None, 1, 2),
        Instruction::new(0x6A, "ROR", AddressingMode::None, 1, 2),
        Instruction::new(0x40, "RTI", AddressingMode::None, 1, 6),
        Instruction::new(0x60, "RTS", AddressingMode::None, 1, 6),
        Instruction::new(0x38, "SEC", AddressingMode::None, 1, 2),
        Instruction::new(0xF8, "SED", AddressingMode::None, 1, 2),
        Instruction::new(0x78, "SEI", AddressingMode::None, 1, 2),
        Instruction::new(0xAA, "TAX", AddressingMode::None, 1, 2),
        Instruction::new(0xA8, "TAY", AddressingMode::None, 1, 2),
        Instruction::new(0xBA, "TSX", AddressingMode::None, 1, 2),
        Instruction::new(0x8A, "TXA", AddressingMode::None, 1, 2),
        Instruction::new(0x9A, "TXS", AddressingMode::None, 1, 2),
        Instruction::new(0x98, "TYA", AddressingMode::None, 1, 2),
        // Other addressing modes
        //      ADC
        Instruction::new(0x69, "ADC", AddressingMode::Immediate, 2, 2),
        Instruction::new(0x65, "ADC", AddressingMode::ZeroPage, 2, 3),
        Instruction::new(0x75, "ADC", AddressingMode::ZeroPageX, 2, 4),
        Instruction::new(0x6D, "ADC", AddressingMode::Absolute, 3, 4),
        Instruction::new(0x7D, "ADC", AddressingMode::AbsoluteX, 3, 4),
        Instruction::new(0x79, "ADC", AddressingMode::AbsoluteY, 3, 4),
        Instruction::new(0x61, "ADC", AddressingMode::IndirectX, 2, 6),
        Instruction::new(0x71, "ADC", AddressingMode::IndirectY, 2, 5),
        //      AND
        Instruction::new(0x29, "AND", AddressingMode::Immediate, 2, 2),
        Instruction::new(0x25, "AND", AddressingMode::ZeroPage, 2, 3),
        Instruction::new(0x35, "AND", AddressingMode::ZeroPageX, 2, 4),
        Instruction::new(0x2D, "AND", AddressingMode::Absolute, 3, 4),
        Instruction::new(0x3D, "AND", AddressingMode::AbsoluteX, 3, 4),
        Instruction::new(0x39, "AND", AddressingMode::AbsoluteY, 3, 4),
        Instruction::new(0x21, "AND", AddressingMode::IndirectX, 2, 6),
        Instruction::new(0x31, "AND", AddressingMode::IndirectY, 2, 5),
        //      ASL
        Instruction::new(0x06, "ASL", AddressingMode::ZeroPage, 2, 5),
        Instruction::new(0x16, "ASL", AddressingMode::ZeroPageX, 2, 6),
        Instruction::new(0x0E, "ASL", AddressingMode::Absolute, 3, 6),
        Instruction::new(0x1E, "ASL", AddressingMode::AbsoluteX, 3, 7),
        //      BCC
        Instruction::new(0x90, "BCC", AddressingMode::Relative, 2, 2),
        //      BCS
        Instruction::new(0xB0, "BCS", AddressingMode::Relative, 2, 2),
        //      BEQ
        Instruction::new(0xF0, "BEQ", AddressingMode::Relative, 2, 2),
        //      BIT
        Instruction::new(0x24, "BIT", AddressingMode::ZeroPage, 2, 3),
        Instruction::new(0x2C, "BIT", AddressingMode::Absolute, 3, 4),
        //      BMI
        Instruction::new(0x30, "BMI", AddressingMode::Relative, 2, 2),
        //      BNE
        Instruction::new(0xD0, "BNE", AddressingMode::Relative, 2, 2),
        //      BPL
        Instruction::new(0x10, "BPL", AddressingMode::Relative, 2, 2),
        //      BVC
        Instruction::new(0x50, "BVC", AddressingMode::Relative, 2, 2),
        //      BVS
        Instruction::new(0x70, "BVS", AddressingMode::Relative, 2, 2),
        //      CMP
        Instruction::new(0xC9, "CMP", AddressingMode::Immediate, 2, 2),
        Instruction::new(0xC5, "CMP", AddressingMode::ZeroPage, 2, 3),
        Instruction::new(0xD5, "CMP", AddressingMode::ZeroPageX, 2, 4),
        Instruction::new(0xCD, "CMP", AddressingMode::Absolute, 3, 4),
        Instruction::new(0xDD, "CMP", AddressingMode::AbsoluteX, 3, 4),
        Instruction::new(0xD9, "CMP", AddressingMode::AbsoluteY, 3, 4),
        Instruction::new(0xC1, "CMP", AddressingMode::IndirectX, 2, 6),
        Instruction::new(0xD1, "CMP", AddressingMode::IndirectY, 2, 5),
        //      CPX
        Instruction::new(0xE0, "CPX", AddressingMode::Immediate, 2, 2),
        Instruction::new(0xE4, "CPX", AddressingMode::ZeroPage, 2, 3),
        Instruction::new(0xEC, "CPX", AddressingMode::Absolute, 3, 4),
        //      CPY
        Instruction::new(0xC0, "CPY", AddressingMode::Immediate, 2, 2),
        Instruction::new(0xC4, "CPY", AddressingMode::ZeroPage, 2, 3),
        Instruction::new(0xCC, "CPY", AddressingMode::Absolute, 3, 4),
        //      DEC
        Instruction::new(0xC6, "DEC", AddressingMode::ZeroPage, 2, 5),
        Instruction::new(0xD6, "DEC", AddressingMode::ZeroPageX, 2, 6),
        Instruction::new(0xCE, "DEC", AddressingMode::Absolute, 3, 6),
        Instruction::new(0xDE, "DEC", AddressingMode::AbsoluteX, 3, 7),
        //      EOR
        Instruction::new(0x49, "EOR", AddressingMode::Immediate, 2, 2),
        Instruction::new(0x45, "EOR", AddressingMode::ZeroPage, 2, 3),
        Instruction::new(0x55, "EOR", AddressingMode::ZeroPageX, 2, 4),
        Instruction::new(0x4D, "EOR", AddressingMode::Absolute, 3, 4),
        Instruction::new(0x5D, "EOR", AddressingMode::AbsoluteX, 3, 4),
        Instruction::new(0x59, "EOR", AddressingMode::AbsoluteY, 3, 4),
        Instruction::new(0x41, "EOR", AddressingMode::IndirectX, 2, 6),
        Instruction::new(0x51, "EOR", AddressingMode::IndirectY, 2, 5),
        //      INC
        Instruction::new(0xE6, "INC", AddressingMode::ZeroPage, 2, 5),
        Instruction::new(0xF6, "INC", AddressingMode::ZeroPageX, 2, 6),
        Instruction::new(0xEE, "INC", AddressingMode::Absolute, 3, 6),
        Instruction::new(0xFE, "INC", AddressingMode::AbsoluteX, 3, 7),
        //      JMP
        Instruction::new(0x4C, "JMP", AddressingMode::Absolute, 3, 3),
        Instruction::new(0x6C, "JMP", AddressingMode::Indirect, 3, 5),
        //      JSR
        Instruction::new(0x20, "JSR", AddressingMode::Absolute, 3, 6),
        //      LDA
        Instruction::new(0xA9, "LDA", AddressingMode::Immediate, 2, 2),
        Instruction::new(0xA5, "LDA", AddressingMode::ZeroPage, 2, 3),
        Instruction::new(0xB5, "LDA", AddressingMode::ZeroPageX, 2, 4),
        Instruction::new(0xAD, "LDA", AddressingMode::Absolute, 3, 4),
        Instruction::new(0xBD, "LDA", AddressingMode::AbsoluteX, 3, 4),
        Instruction::new(0xB9, "LDA", AddressingMode::AbsoluteY, 3, 4),
        Instruction::new(0xA1, "LDA", AddressingMode::IndirectX, 2, 6),
        Instruction::new(0xB1, "LDA", AddressingMode::IndirectY, 2, 5),
        //      LDX
        Instruction::new(0xA2, "LDX", AddressingMode::Immediate, 2, 2),
        Instruction::new(0xA6, "LDX", AddressingMode::ZeroPage, 2, 3),
        Instruction::new(0xB6, "LDX", AddressingMode::ZeroPageY, 2, 4),
        Instruction::new(0xAE, "LDX", AddressingMode::Absolute, 3, 4),
        Instruction::new(0xBE, "LDX", AddressingMode::AbsoluteY, 3, 4),
        //      LDY
        Instruction::new(0xA0, "LDY", AddressingMode::Immediate, 2, 2),
        Instruction::new(0xA4, "LDY", AddressingMode::ZeroPage, 2, 3),
        Instruction::new(0xB4, "LDY", AddressingMode::ZeroPageX, 2, 4),
        Instruction::new(0xAC, "LDY", AddressingMode::Absolute, 3, 4),
        Instruction::new(0xBC, "LDY", AddressingMode::AbsoluteX, 3, 4),
        //      LSR
        Instruction::new(0x46, "LSR", AddressingMode::ZeroPage, 2, 5),
        Instruction::new(0x56, "LSR", AddressingMode::ZeroPageX, 2, 6),
        Instruction::new(0x4E, "LSR", AddressingMode::Absolute, 3, 6),
        Instruction::new(0x5E, "LSR", AddressingMode::AbsoluteX, 3, 7),
        //      ORA
        Instruction::new(0x09, "ORA", AddressingMode::Immediate, 2, 2),
        Instruction::new(0x05, "ORA", AddressingMode::ZeroPage, 2, 3),
        Instruction::new(0x15, "ORA", AddressingMode::ZeroPageX, 2, 4),
        Instruction::new(0x0D, "ORA", AddressingMode::Absolute, 3, 4),
        Instruction::new(0x1D, "ORA", AddressingMode::AbsoluteX, 3, 4),
        Instruction::new(0x19, "ORA", AddressingMode::AbsoluteY, 3, 4),
        Instruction::new(0x01, "ORA", AddressingMode::IndirectX, 2, 6),
        Instruction::new(0x11, "ORA", AddressingMode::IndirectY, 2, 5),
        //      ROL
        Instruction::new(0x26, "ROL", AddressingMode::ZeroPage, 2, 5),
        Instruction::new(0x36, "ROL", AddressingMode::ZeroPageX, 2, 6),
        Instruction::new(0x2E, "ROL", AddressingMode::Absolute, 3, 6),
        Instruction::new(0x3E, "ROL", AddressingMode::AbsoluteX, 3, 7),
        //      ROR
        Instruction::new(0x66, "ROR", AddressingMode::ZeroPage, 2, 5),
        Instruction::new(0x76, "ROR", AddressingMode::ZeroPageX, 2, 6),
        Instruction::new(0x6E, "ROR", AddressingMode::Absolute, 3, 6),
        Instruction::new(0x7E, "ROR", AddressingMode::AbsoluteX, 3, 7),
        //      SBC
        Instruction::new(0xE9, "SBC", AddressingMode::Immediate, 2, 2),
        Instruction::new(0xE5, "SBC", AddressingMode::ZeroPage, 2, 3),
        Instruction::new(0xF5, "SBC", AddressingMode::ZeroPageX, 2, 4),
        Instruction::new(0xED, "SBC", AddressingMode::Absolute, 3, 4),
        Instruction::new(0xFD, "SBC", AddressingMode::AbsoluteX, 3, 4),
        Instruction::new(0xF9, "SBC", AddressingMode::AbsoluteY, 3, 4),
        Instruction::new(0xE1, "SBC", AddressingMode::IndirectX, 2, 6),
        Instruction::new(0xF1, "SBC", AddressingMode::IndirectY, 2, 5),
        //      STA
        Instruction::new(0x85, "STA", AddressingMode::ZeroPage, 2, 3),
        Instruction::new(0x95, "STA", AddressingMode::ZeroPageX, 2, 4),
        Instruction::new(0x8D, "STA", AddressingMode::Absolute, 3, 4),
        Instruction::new(0x9D, "STA", AddressingMode::AbsoluteX, 3, 5),
        Instruction::new(0x99, "STA", AddressingMode::AbsoluteY, 3, 5),
        Instruction::new(0x81, "STA", AddressingMode::IndirectX, 2, 6),
        Instruction::new(0x91, "STA", AddressingMode::IndirectY, 2, 6),
        //      STX
        Instruction::new(0x86, "STX", AddressingMode::ZeroPage, 2, 3),
        Instruction::new(0x96, "STX", AddressingMode::ZeroPageY, 2, 4),
        Instruction::new(0x8E, "STX", AddressingMode::Absolute, 3, 4),
        //      STY
        Instruction::new(0x84, "STY", AddressingMode::ZeroPage, 2, 3),
        Instruction::new(0x94, "STY", AddressingMode::ZeroPageX, 2, 4),
        Instruction::new(0x8C, "STY", AddressingMode::Absolute, 3, 4),
    ]
}

// #[cfg(test)]
// mod instruction_tests {
//     use std::fs;

//     use crate::{
//         bus::Bus,
//         console::Console,
//         cpu::{Cpu, Flags},
//         instruction::{instructions, step},
//         ppu::Ppu,
//         rom::Rom,
//         util::Error,
//     };

//     fn load_rom(rom: Rom, rom_data: Vec<u8>) {
//         rom.program_rom[0..rom_data.len()].copy_from_slice(&rom_data[..]);
//     }

//     #[test]
//     fn lda_immediate_loads_immediate_value() -> Result<(), Error> {
//         let rom_bytes = fs::read("roms/snake.nes")?;
//         let rom = Rom::new(&rom_bytes)?;
//         let ppu = Ppu::new(&rom);
//         let mut console = Console {
//             cpu: Cpu::new(),
//             bus: Bus::new(ppu, &rom),
//         };
//         let instructions = instructions();
//         let program = vec![
//             0xA9, // LDA #$FF     $8000
//             0xFF,
//         ];

//         // Running 1 step should move PC forward 2, and load the immediate value.
//         console.bus.load_rom(program);
//         step(&mut console, &instructions)?;

//         assert_eq!(console.cpu.pc, 0x8002);
//         assert_eq!(console.cpu.a, 0xFF);
//         Ok(())
//     }

//     #[test]
//     fn lda_zero_page_loads_from_memory() -> Result<(), Error> {
//         let rom_bytes = fs::read("roms/snake.nes")?;
//         let rom = Rom::new(&rom_bytes)?;
//         let opcodes = instructions();
//         let program = vec![
//             0xA5, // LDA $FF     ;$8000
//             0xFF,
//         ];
//         load_rom(rom, program);
//         let ppu = Ppu::new(&rom);
//         let mut console = Console {
//             cpu: Cpu::new(),
//             bus: Bus::new(ppu, &rom),
//         };

//         console.bus.write_u8(0xFF, 0xAB); // Set the value to read from $FF

//         step(&mut console, &opcodes)?;

//         assert_eq!(console.cpu.pc, 0x8002);
//         assert_eq!(console.cpu.a, 0xAB);
//         Ok(())
//     }

//     #[test]
//     fn lda_zero_page_x_loads_from_memory() -> Result<(), Error> {
//         let rom_bytes = fs::read("roms/snake.nes")?;
//         let rom = Rom::new(&rom_bytes)?;
//         let ppu = Ppu::new(&rom);
//         let mut console = Console {
//             cpu: Cpu::new(),
//             bus: Bus::new(ppu, &rom),
//         };
//         let opcodes = instructions();
//         let program = vec![
//             0xB5, // LDA $F0,X     ;$8000
//             0xF0,
//         ];

//         console.cpu.x = 0x08; // Set X offset
//         console.bus.write_u8(0xF8, 0xAB); // Set the value to read from $F0 + x
//         console.bus.load_rom(program);

//         step(&mut console, &opcodes)?;

//         assert_eq!(console.cpu.pc, 0x8002);
//         assert_eq!(console.cpu.a, 0xAB);
//         Ok(())
//     }

//     #[test]
//     fn lda_zero_page_x_wraps_if_greater_than_0xff() -> Result<(), Error> {
//         let rom_bytes = fs::read("roms/snake.nes")?;
//         let rom = Rom::new(&rom_bytes)?;
//         let ppu = Ppu::new(&rom);
//         let mut console = Console {
//             cpu: Cpu::new(),
//             bus: Bus::new(ppu, &rom),
//         };
//         let opcodes = instructions();
//         let program = vec![
//             0xB5, // LDA $F0,X     ;$8000
//             0xFF,
//         ];

//         console.cpu.x = 0x10; // Set X offset
//         console.bus.write_u8(0x0F, 0xAB); // Set the value to read from $FF + x
//         console.bus.load_rom(program);

//         step(&mut console, &opcodes)?;

//         assert_eq!(console.cpu.pc, 0x8002);
//         assert_eq!(console.cpu.a, 0xAB);
//         Ok(())
//     }

//     #[test]
//     fn lda_absolute_loads_from_memory() -> Result<(), Error> {
//         let rom_bytes = fs::read("roms/snake.nes")?;
//         let rom = Rom::new(&rom_bytes)?;
//         let ppu = Ppu::new(&rom);
//         let mut console = Console {
//             cpu: Cpu::new(),
//             bus: Bus::new(ppu, &rom),
//         };
//         let opcodes = instructions();
//         let program = vec![
//             0xAD, // LDA $0BCD     ;$8000
//             0xCD, //
//             0x0B, //
//         ];

//         console.bus.write_u8(0x0BCD, 0x11); // Set the value to read from $ABCD
//         console.bus.load_rom(program);

//         step(&mut console, &opcodes)?;

//         assert_eq!(console.cpu.pc, 0x8003);
//         assert_eq!(console.cpu.a, 0x11);
//         Ok(())
//     }

//     #[test]
//     fn lda_absolute_x_loads_from_memory() -> Result<(), Error> {
//         let rom_bytes = fs::read("roms/snake.nes")?;
//         let rom = Rom::new(&rom_bytes)?;
//         let ppu = Ppu::new(&rom);
//         let mut console = Console {
//             cpu: Cpu::new(),
//             bus: Bus::new(ppu, &rom),
//         };
//         let opcodes = instructions();
//         let program = vec![
//             0xBD, // LDA $0F00,X     ;$8000
//             0x00, //
//             0x0F, //
//         ];

//         console.cpu.x = 0x22;
//         console.bus.write_u8(0x0F22, 0xAB); // Set the value to read from $F000 + 0x22
//         console.bus.load_rom(program);

//         step(&mut console, &opcodes)?;

//         assert_eq!(console.cpu.pc, 0x8003);
//         assert_eq!(console.cpu.a, 0xAB);
//         Ok(())
//     }

//     #[test]
//     fn lda_absolute_y_loads_from_memory() -> Result<(), Error> {
//         let rom_bytes = fs::read("roms/snake.nes")?;
//         let rom = Rom::new(&rom_bytes)?;
//         let ppu = Ppu::new(&rom);
//         let mut console = Console {
//             cpu: Cpu::new(),
//             bus: Bus::new(ppu, &rom),
//         };
//         let opcodes = instructions();
//         let program = vec![
//             0xB9, // LDA $0F00,Y     ;$8000
//             0x00, //
//             0x0F, //
//         ];

//         console.cpu.y = 0x22;
//         console.bus.write_u8(0x0F22, 0xAB); // Set the value to read from $F000 + 0x22
//         console.bus.load_rom(program);

//         step(&mut console, &opcodes)?;

//         assert_eq!(console.cpu.pc, 0x8003);
//         assert_eq!(console.cpu.a, 0xAB);
//         Ok(())
//     }

//     #[test]
//     fn lda_indirect_x_loads_from_memory() -> Result<(), Error> {
//         let rom_bytes = fs::read("roms/snake.nes")?;
//         let rom = Rom::new(&rom_bytes)?;
//         let ppu = Ppu::new(&rom);
//         let mut console = Console {
//             cpu: Cpu::new(),
//             bus: Bus::new(ppu, &rom),
//         };
//         let opcodes = instructions();
//         let program = vec![
//             0xA1, // LDA ($20,X)     ;$8000
//             0x20,
//         ];

//         console.cpu.x = 0x08;
//         console.bus.write_u8(0x28, 0xAB); // Set the destination to read from ($20 + 0x08)
//         console.bus.write_u8(0xAB, 0xCD);
//         console.bus.load_rom(program);

//         step(&mut console, &opcodes)?;

//         assert_eq!(console.cpu.pc, 0x8002);
//         assert_eq!(console.cpu.a, 0xCD);
//         Ok(())
//     }

//     #[test]
//     fn lda_indirect_y_loads_from_memory() -> Result<(), Error> {
//         let rom_bytes = fs::read("roms/snake.nes")?;
//         let rom = Rom::new(&rom_bytes)?;
//         let ppu = Ppu::new(&rom);
//         let mut console = Console {
//             cpu: Cpu::new(),
//             bus: Bus::new(ppu, &rom),
//         };
//         let opcodes = instructions();
//         let program = vec![
//             0xB1, // LDA ($20),Y     ;$8000
//             0x20,
//         ];

//         console.cpu.y = 0x08;
//         console.bus.write_u8(0x20, 0xF0); // Set the destination to read from $20
//         console.bus.write_u8(0xF8, 0xCD); // Set the value to read from $F0 + 0x08
//         console.bus.load_rom(program);

//         step(&mut console, &opcodes)?;

//         assert_eq!(console.cpu.pc, 0x8002);
//         assert_eq!(console.cpu.a, 0xCD);
//         Ok(())
//     }

//     #[test]
//     fn lda_sets_flags_correctly() -> Result<(), Error> {
//         let rom_bytes = fs::read("roms/snake.nes")?;
//         let rom = Rom::new(&rom_bytes)?;
//         let ppu = Ppu::new(&rom);
//         let mut console = Console {
//             cpu: Cpu::new(),
//             bus: Bus::new(ppu, &rom),
//         };
//         let opcodes = instructions();
//         let program = vec![
//             0xA9, // LDA #$20
//             0x20, //
//             0xA9, // LDA #0
//             0x00, //
//             0xA9, // LDA #$(-10 as u8)
//             ((-10 as i8) as u8),
//         ];

//         console.bus.load_rom(program);

//         step(&mut console, &opcodes)?;
//         assert_eq!(console.cpu.pc, 0x8002);
//         assert_eq!(console.cpu.a, 0x20);
//         assert!(!console.cpu.flags.contains(Flags::ZERO)); // Not zero
//         assert!(!console.cpu.flags.contains(Flags::NEGATIVE)); // Not negative

//         step(&mut console, &opcodes)?;
//         assert_eq!(console.cpu.pc, 0x8004);
//         assert_eq!(console.cpu.a, 0x00);
//         assert!(console.cpu.flags.contains(Flags::ZERO)); // Is zero
//         assert!(!console.cpu.flags.contains(Flags::NEGATIVE)); // Not negative

//         step(&mut console, &opcodes)?;
//         assert_eq!(console.cpu.pc, 0x8006);
//         assert_eq!(console.cpu.a, ((-10 as i8) as u8));
//         assert!(!console.cpu.flags.contains(Flags::ZERO)); // Not Zero
//         assert!(console.cpu.flags.contains(Flags::NEGATIVE)); // Is negative

//         Ok(())
//     }

//     #[test]
//     fn asl_works() -> Result<(), Error> {
//         let rom_bytes = fs::read("roms/snake.nes")?;
//         let rom = Rom::new(&rom_bytes)?;
//         let ppu = Ppu::new(&rom);
//         let mut console = Console {
//             cpu: Cpu::new(),
//             bus: Bus::new(ppu, &rom),
//         };
//         let opcodes = instructions();
//         let program = vec![
//             0x0A, // ASL A
//             0x06, // ASL $20
//             0x20, //
//         ];

//         console.cpu.a = 0b1000_0000;
//         console.bus.write_u8(0x20, 0b0100_0000);
//         console.bus.load_rom(program);

//         step(&mut console, &opcodes)?;
//         assert_eq!(console.cpu.a, 0);
//         assert!(console.cpu.flags.contains(Flags::CARRY));
//         assert!(console.cpu.flags.contains(Flags::ZERO));
//         assert!(!console.cpu.flags.contains(Flags::NEGATIVE));

//         step(&mut console, &opcodes)?;
//         let result = console.bus.read_u8(0x20);
//         assert_eq!(result, 0b1000_0000);
//         assert!(!console.cpu.flags.contains(Flags::CARRY));
//         assert!(!console.cpu.flags.contains(Flags::ZERO));
//         assert!(console.cpu.flags.contains(Flags::NEGATIVE));
//         Ok(())
//     }
// }
