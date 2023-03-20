use crate::{
    console::Console,
    instruction::{AddressingMode, Instruction},
};

pub fn trace(Console { cpu, bus }: &mut Console, instruction: &Instruction) -> String {
    let instruction_bytes: Vec<u8> = (0..instruction.bytes as u16)
        .map(|i| bus.read_u8(cpu.pc + i))
        .collect();
    let instruction_bytes_string = instruction_bytes
        .iter()
        .map(|byte| format!("{:02X}", byte))
        .fold(String::new(), |a, b| a + &b + " ");

    let mut instruction_assembly: String = match instruction.addressing_mode {
        AddressingMode::Immediate => {
            format!("{} #${:02X}", instruction.operation, instruction_bytes[1])
        }
        AddressingMode::ZeroPage => {
            format!("{} ${:02X}", instruction.operation, instruction_bytes[1])
        }
        AddressingMode::ZeroPageX => {
            format!("{} ${:02X},X", instruction.operation, instruction_bytes[1])
        }
        AddressingMode::ZeroPageY => {
            format!("{} ${:02X},Y", instruction.operation, instruction_bytes[1])
        }
        AddressingMode::Relative => {
            let offset = bus.read_i8(cpu.pc + 1);
            let address = cpu.pc as i32 + 2 + offset as i32; // PC is incremented +2 during read
            format!("{} ${:02X}", instruction.operation, address)
        }
        AddressingMode::Absolute => format!(
            "{} ${:02X}{:02X}",
            instruction.operation, instruction_bytes[2], instruction_bytes[1]
        ),
        AddressingMode::AbsoluteX => format!(
            "{} ${:02X}{:02X},X",
            instruction.operation, instruction_bytes[2], instruction_bytes[1]
        ),
        AddressingMode::AbsoluteY => format!(
            "{} ${:02X}{:02X},Y",
            instruction.operation, instruction_bytes[2], instruction_bytes[1]
        ),
        AddressingMode::Indirect => format!(
            "{} (${:02X}{:02X})",
            instruction.operation, instruction_bytes[2], instruction_bytes[1]
        ),
        AddressingMode::IndirectX => format!(
            "{} (${:02X},X)",
            instruction.operation, instruction_bytes[1]
        ),
        AddressingMode::IndirectY => format!(
            "{} (${:02X}),Y",
            instruction.operation, instruction_bytes[1]
        ),
        AddressingMode::None => instruction.operation.to_string(),
    };

    instruction_assembly = instruction_assembly
        + &match instruction.addressing_mode {
            AddressingMode::None => match instruction.operation {
                "ASL" | "LSR" | "ROL" | "ROR" => " A".to_string(),
                _ => "".to_string(),
            },
            AddressingMode::ZeroPage => {
                let address = instruction_bytes[1] as u16;
                let value = bus.read_u8(address);
                format!(" = {:02X}", value)
            }
            AddressingMode::ZeroPageX => {
                let address = instruction_bytes[1];
                let address_x = address.wrapping_add(cpu.x);
                let value = bus.read_u8(address_x as u16);
                format!(" @ {:02X} = {:02X}", address_x, value)
            }
            AddressingMode::ZeroPageY => {
                let address = instruction_bytes[1];
                let address_y = address.wrapping_add(cpu.y);
                let value = bus.read_u8(address_y as u16);
                format!(" @ {:02X} = {:02X}", address_y, value)
            }
            AddressingMode::Absolute => match instruction.operation {
                "JMP" | "JSR" => "".to_string(),
                _ => {
                    let address = u16::from_le_bytes([instruction_bytes[1], instruction_bytes[2]]);
                    let value = bus.read_u8(address);
                    format!(" = {:02X}", value)
                }
            },
            AddressingMode::AbsoluteX => {
                let address = u16::from_le_bytes([instruction_bytes[1], instruction_bytes[2]]);
                let address_x = address.wrapping_add(cpu.x as u16);
                let value = bus.read_u8(address_x);
                format!(" @ {:04X} = {:02X}", address_x, value)
            }
            AddressingMode::AbsoluteY => {
                let address = u16::from_le_bytes([instruction_bytes[1], instruction_bytes[2]]);
                let address_y = address.wrapping_add(cpu.y as u16);
                let value = bus.read_u8(address_y);
                format!(" @ {:04X} = {:02X}", address_y, value)
            }
            AddressingMode::Indirect => {
                let indirect_address =
                    u16::from_le_bytes([instruction_bytes[1], instruction_bytes[2]]);
                let address = bus.read_u16_wrap_page(indirect_address);
                format!(" = {:04X}", address)
            }
            AddressingMode::IndirectX => {
                let mut indirect_address = instruction_bytes[1];
                indirect_address = indirect_address.wrapping_add(cpu.x);
                let address = bus.read_u16_wrap_page(indirect_address as u16);
                let value = bus.read_u8(address);
                format!(
                    " @ {:02X} = {:04X} = {:02X}",
                    indirect_address, address, value
                )
            }
            AddressingMode::IndirectY => {
                let indirect_address = instruction_bytes[1];
                let address = bus.read_u16_wrap_page(indirect_address as u16);
                let address_y = address.wrapping_add(cpu.y as u16);
                let value = bus.read_u8(address_y);
                format!(" = {:04X} @ {:04X} = {:02X}", address, address_y, value)
            }
            _ => "".to_string(),
        };

    format!(
        "{:04X}  {:10}{:32}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
        cpu.pc,
        instruction_bytes_string,
        instruction_assembly,
        cpu.a,
        cpu.x,
        cpu.y,
        cpu.flags,
        cpu.sp
    )
}
