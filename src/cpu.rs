use crate::{
    bus,
    config::{CPU_FLAGS_START_VALUE, CPU_SP_START_VALUE},
    console::Console,
    instruction::{AddressingMode, Instruction},
    util::Error,
};
use bitflags::bitflags;

const ROM_START: u16 = 0xC000;
const STACK_PAGE_ADDRESS: u16 = 0x0100;

const RESET_INTERRUPT_VECTOR_ADDRESS: u16 = 0xFFFC;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Flags: u8 {
        const NEGATIVE          = 0b1000_0000;
        const OVERFLOW          = 0b0100_0000;
        const BREAK             = 0b0010_0000;
        const BREAK_2           = 0b0001_0000;
        const DECIMAL           = 0b0000_1000;
        const INTERRUPT_DISABLE = 0b0000_0100;
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
            flags: Flags::from_bits_retain(CPU_FLAGS_START_VALUE),
        }
    }
}

/**
 * Pulls a value from the stack
 */
pub fn pull_stack_u8(console: &mut Console) -> Result<u8, Error> {
    console.cpu.sp += 1;
    let address = STACK_PAGE_ADDRESS + console.cpu.sp as u16;
    let value = bus::read_u8(console, address);
    Ok(value)
}

/**
 * Pulls 2 values from the stack, and returns them as a u16
 */
pub fn pull_stack_u16(console: &mut Console) -> Result<u16, Error> {
    console.cpu.sp += 2;
    let address = STACK_PAGE_ADDRESS + console.cpu.sp as u16 - 1;
    let value = bus::read_u16(console, address);
    Ok(value)
}

/**
 * Pushes the given value to the stack
 */
pub fn push_stack_u8(console: &mut Console, value: u8) {
    let address = STACK_PAGE_ADDRESS + console.cpu.sp as u16;
    bus::write_u8(console, address, value);
    console.cpu.sp -= 1;
}

/**
 * Pushes the given value to the stack as 2 u8's
 */
pub fn push_stack_u16(console: &mut Console, value: u16) {
    let address = (0x0100 | console.cpu.sp as u16) - 1;
    bus::write_u16(console, address, value);
    console.cpu.sp -= 2;
}

// ==== Interrupts ====

pub fn reset_interrupt(console: &mut Console) {
    console.cpu.a = 0;
    console.cpu.x = 0;
    console.cpu.flags = Flags::from_bits_retain(CPU_FLAGS_START_VALUE);

    console.cpu.pc = bus::read_u16(console, RESET_INTERRUPT_VECTOR_ADDRESS);
}

pub fn nmi_interrupt(console: &mut Console) {
    push_stack_u16(console, console.cpu.pc);
    let mut flags = console.cpu.flags.clone();
    flags = flags.union(Flags::BREAK).difference(Flags::BREAK_2);

    push_stack_u8(console, flags.bits());
    console.cpu.flags.insert(Flags::INTERRUPT_DISABLE);

    console.ppu.tick(2 * 3);
    console.cpu.pc = bus::read_u16(console, 0xFFFA);
}

pub fn step(console: &mut Console, instruction: &Instruction) -> Result<(), Error> {
    // Logs instruction name
    fn read_address(console: &mut Console, mode: AddressingMode) -> Result<u16, Error> {
        match mode {
            AddressingMode::Immediate => {
                let address = console.cpu.pc;
                console.cpu.pc += 1;

                Ok(address)
            }
            AddressingMode::ZeroPage => {
                let address = bus::read_u8(console, console.cpu.pc);
                console.cpu.pc += 1;

                Ok(address as u16)
            }
            AddressingMode::ZeroPageX => {
                let mut address = bus::read_u8(console, console.cpu.pc);
                console.cpu.pc += 1;

                address = address.wrapping_add(console.cpu.x);
                Ok(address as u16)
            }
            AddressingMode::ZeroPageY => {
                let mut address = bus::read_u8(console, console.cpu.pc);
                console.cpu.pc += 1;

                address = address.wrapping_add(console.cpu.y);
                Ok(address as u16)
            }
            AddressingMode::Relative => {
                let address = console.cpu.pc;
                console.cpu.pc += 1;

                Ok(address)
            }
            AddressingMode::Absolute => {
                let address = bus::read_u16(console, console.cpu.pc);
                console.cpu.pc += 2;

                Ok(address)
            }
            AddressingMode::AbsoluteX => {
                let mut address = bus::read_u16(console, console.cpu.pc);
                console.cpu.pc += 2;

                address = address.wrapping_add(console.cpu.x as u16);
                Ok(address)
            }
            AddressingMode::AbsoluteY => {
                let mut address = bus::read_u16(console, console.cpu.pc);
                console.cpu.pc += 2;

                address = address.wrapping_add(console.cpu.y as u16);
                Ok(address)
            }
            AddressingMode::Indirect => {
                let indirect_address = bus::read_u16(console, console.cpu.pc);
                console.cpu.pc += 2;

                let address = bus::read_u16_wrap_page(console, indirect_address);
                Ok(address)
            }
            AddressingMode::IndirectX => {
                let mut indirect_address = bus::read_u8(console, console.cpu.pc);
                console.cpu.pc += 1;

                // Read the final address from memory[indirect_address + x]
                indirect_address = indirect_address.wrapping_add(console.cpu.x);
                let address = bus::read_u16_wrap_page(console, indirect_address as u16);
                Ok(address)
            }
            AddressingMode::IndirectY => {
                let indirect_address = bus::read_u8(console, console.cpu.pc);
                console.cpu.pc += 1;

                // The final address is (memory[indirect_address]) + y
                let mut address = bus::read_u16_wrap_page(console, indirect_address as u16);
                address = address.wrapping_add(console.cpu.y as u16);
                Ok(address)
            }
            _ => {
                panic!()
            }
        }
    }

    /**
     * Increment the given register
     *  N Z C I D V
     *  ? ? - - - -
     */
    fn increment(register: &mut u8, flags: &mut Flags) {
        let value = *register;
        let result = value.wrapping_add(1);
        *register = result;

        let zero = result == 0;
        let negative = (result as i8) < 0;
        flags.set(Flags::ZERO, zero);
        flags.set(Flags::NEGATIVE, negative);
    }

    /**
     * Decrement the given register
     *  N Z C I D V
     *  ? ? - - - -
     */
    fn decrement(register: &mut u8, flags: &mut Flags) {
        let value = *register;
        let result = value.wrapping_sub(1);
        *register = result;

        let zero = result == 0;
        let negative = (result as i8) < 0;
        flags.set(Flags::ZERO, zero);
        flags.set(Flags::NEGATIVE, negative);
    }

    /**
     * Set flags based on (lhs - rhs)
     *  N Z C I D V
     *  - - - - - -
     */
    fn compare(lhs: u8, rhs: u8, flags: &mut Flags) {
        let (result, borrow) = lhs.borrowing_sub(rhs, false);

        let zero = lhs == rhs;
        let negative = (result as i8) < 0;
        flags.set(Flags::CARRY, !borrow);
        flags.set(Flags::ZERO, zero);
        flags.set(Flags::NEGATIVE, negative);
    }

    /**
     * Branch (add offset to console.cpu.pc) if the condition is true
     *  N Z C I D V
     *  - - - - - -
     */
    fn branch(cpu: &mut Cpu, condition: bool, offset: i8) {
        if condition {
            cpu.pc = (cpu.pc as i16 + offset as i16) as u16
        }
    }

    /**
     *  Loads the given value to a register
     *  N Z C I D V
     *  ? ? - - - -
     */
    fn load(value: u8, register: &mut u8, flags: &mut Flags) {
        *register = value;

        let zero = value == 0;
        let negative = (value as i8) < 0;
        flags.set(Flags::ZERO, zero);
        flags.set(Flags::NEGATIVE, negative);
    }

    console.cpu.pc += 1; // Increment for opcode read in main.rs

    match instruction.addressing_mode {
        AddressingMode::None => {
            // Execute immediately.

            match instruction.operation {
                "ASL" => {
                    let value = console.cpu.a;
                    let result = value << 1;
                    console.cpu.a = result;

                    let carry = (value & 0b1000_0000) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    console.cpu.flags.set(Flags::CARRY, carry);
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "BRK" => {
                    push_stack_u16(console, console.cpu.pc);
                    push_stack_u8(console, console.cpu.flags.bits());
                    console.cpu.pc = bus::read_u16(console, 0xFFFE);
                    console.cpu.flags.set(Flags::BREAK, true);
                }
                "CLC" => {
                    console.cpu.flags.set(Flags::CARRY, false);
                }
                "CLD" => {
                    console.cpu.flags.set(Flags::DECIMAL, false);
                }
                "CLI" => {
                    console.cpu.flags.set(Flags::INTERRUPT_DISABLE, false);
                }
                "CLV" => {
                    console.cpu.flags.set(Flags::OVERFLOW, false);
                }
                "DEX" => decrement(&mut console.cpu.x, &mut console.cpu.flags),
                "DEY" => decrement(&mut console.cpu.y, &mut console.cpu.flags),
                "INX" => increment(&mut console.cpu.x, &mut console.cpu.flags),
                "INY" => increment(&mut console.cpu.y, &mut console.cpu.flags),
                "LSR" => {
                    let value = console.cpu.a;
                    let result = value >> 1;
                    console.cpu.a = result;

                    let carry = (value & 1) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    console.cpu.flags.set(Flags::CARRY, carry);
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "NOP" => {}
                "PHA" => {
                    // Push A to stack
                    push_stack_u8(console, console.cpu.a);
                }
                "PHP" => {
                    // Push flags to stack
                    // Pushes with bits 5 and 4 true
                    let flags_to_push = console
                        .cpu
                        .flags
                        .union(Flags::BREAK | Flags::BREAK_2)
                        .bits();
                    push_stack_u8(console, flags_to_push);
                }
                "PLA" => {
                    // Pull stack to A
                    let value = pull_stack_u8(console)?;
                    console.cpu.a = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "PLP" => {
                    // Pull stack to flags
                    // Sets bit 5 to 1, bit 4 to 0
                    let pulled_flags = Flags::from_bits_retain(pull_stack_u8(console)?);
                    let flags = pulled_flags.union(Flags::BREAK).difference(Flags::BREAK_2);
                    console.cpu.flags = flags;
                }
                "ROL" => {
                    // Rotate A left
                    // result_carry <- [7..0] <- carry
                    let value = console.cpu.a;
                    let carry_mask = if console.cpu.flags.contains(Flags::CARRY) {
                        1
                    } else {
                        0
                    };
                    let result = (value << 1) | carry_mask;
                    console.cpu.a = result;

                    let carry = (value & 0b1000_0000) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    console.cpu.flags.set(Flags::CARRY, carry);
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "ROR" => {
                    // Rotate A right
                    // carry -> [7..0] -> result_carry
                    let value = console.cpu.a;
                    let carry_mask = if console.cpu.flags.contains(Flags::CARRY) {
                        0b1000_0000
                    } else {
                        0
                    };
                    let result = (value >> 1) | carry_mask;
                    console.cpu.a = result;

                    let carry = (value & 1) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    console.cpu.flags.set(Flags::CARRY, carry);
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "RTI" => {
                    // Sets bit 5 to 1, bit 4 to 0
                    let pulled_flags = Flags::from_bits_retain(pull_stack_u8(console)?);
                    let flags = pulled_flags.union(Flags::BREAK).difference(Flags::BREAK_2);
                    console.cpu.flags = flags;
                    console.cpu.pc = pull_stack_u16(console)?;
                }
                "RTS" => {
                    console.cpu.pc = pull_stack_u16(console)?;
                    console.cpu.pc += 1;
                }
                "SEC" => {
                    console.cpu.flags.set(Flags::CARRY, true);
                }
                "SED" => {
                    console.cpu.flags.set(Flags::DECIMAL, true);
                }
                "SEI" => {
                    console.cpu.flags.set(Flags::INTERRUPT_DISABLE, true);
                }
                "TAX" => {
                    let value = console.cpu.a;
                    console.cpu.x = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "TAY" => {
                    let value = console.cpu.a;
                    console.cpu.y = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "TSX" => {
                    let value = console.cpu.sp;
                    console.cpu.x = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "TXA" => {
                    let value = console.cpu.x;
                    console.cpu.a = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "TXS" => {
                    console.cpu.sp = console.cpu.x;
                }
                "TYA" => {
                    let value = console.cpu.y;
                    console.cpu.a = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                operation => {
                    todo!("{:?}", operation)
                }
            }
        }
        // Load a value based on the addressing mode, and then execute
        _ => {
            let address = read_address(console, instruction.addressing_mode)?;

            match instruction.operation {
                "ADC" => {
                    let acc_value = console.cpu.a;
                    let memory_value = bus::read_u8(console, address);
                    let carry = console.cpu.flags.contains(Flags::CARRY);

                    let (result, result_carry) = acc_value.carrying_add(memory_value, carry);
                    console.cpu.a = result;

                    let zero = result == 0;
                    let overflow = (acc_value as i8).checked_add(memory_value as i8).is_none();
                    let negative = (result as i8) < 0;
                    console.cpu.flags.set(Flags::CARRY, result_carry);
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::OVERFLOW, overflow);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "AND" => {
                    let value = bus::read_u8(console, address);
                    let result = console.cpu.a & value;
                    console.cpu.a = result;

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "ASL" => {
                    // Shift bits left 1
                    let value = bus::read_u8(console, address);
                    let result = value << 1;
                    bus::write_u8(console, address, result);

                    let carry = (value & 0b1000_0000) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    console.cpu.flags.set(Flags::CARRY, carry);
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "BCC" => {
                    // Branch if carry flag is clear
                    let offset = bus::read_i8(console, address);
                    let condition = !console.cpu.flags.contains(Flags::CARRY);
                    branch(&mut console.cpu, condition, offset);
                }
                "BCS" => {
                    // Branch if carry flag is set
                    let offset = bus::read_i8(console, address);
                    let condition = console.cpu.flags.contains(Flags::CARRY);
                    branch(&mut console.cpu, condition, offset);
                }
                "BEQ" => {
                    // Branch if zero flag is set
                    let offset = bus::read_i8(console, address);
                    let condition = console.cpu.flags.contains(Flags::ZERO);
                    branch(&mut console.cpu, condition, offset);
                }
                "BIT" => {
                    // Set zero flag to (A AND value) == 0
                    let value = bus::read_u8(console, address);
                    let result = console.cpu.a & value;

                    let zero = result == 0;
                    let overflow = (value & 0b0100_0000) != 0; // Overflow -> bit 6
                    let negative = (value & 0b1000_0000) != 0; // Negative -> bit 7
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::OVERFLOW, overflow);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "BMI" => {
                    // Branch if negative flag is set
                    let offset = bus::read_i8(console, address);
                    let condition = console.cpu.flags.contains(Flags::NEGATIVE);
                    branch(&mut console.cpu, condition, offset);
                }
                "BNE" => {
                    // Branch if zero flag is clear
                    let offset = bus::read_i8(console, address);
                    let condition = !console.cpu.flags.contains(Flags::ZERO);
                    branch(&mut console.cpu, condition, offset);
                }
                "BPL" => {
                    // Branch if negative flag is clear
                    let offset = bus::read_i8(console, address);
                    let condition = !console.cpu.flags.contains(Flags::NEGATIVE);
                    branch(&mut console.cpu, condition, offset);
                }
                "BVC" => {
                    // Branch if overflow flag is clear
                    let offset = bus::read_i8(console, address);
                    let condition = !console.cpu.flags.contains(Flags::OVERFLOW);
                    branch(&mut console.cpu, condition, offset);
                }
                "BVS" => {
                    // Branch if overflow flag is set
                    let offset = bus::read_i8(console, address);
                    let condition = console.cpu.flags.contains(Flags::OVERFLOW);
                    branch(&mut console.cpu, condition, offset);
                }
                "CMP" => {
                    // Set flags based on A - M
                    let memory_value = bus::read_u8(console, address);
                    compare(console.cpu.a, memory_value, &mut console.cpu.flags);
                }
                "CPX" => {
                    // Set flags based on X - M
                    let memory_value = bus::read_u8(console, address);
                    compare(console.cpu.x, memory_value, &mut console.cpu.flags);
                }
                "CPY" => {
                    // Set flags based on Y - M
                    let memory_value = bus::read_u8(console, address);
                    compare(console.cpu.y, memory_value, &mut console.cpu.flags);
                }
                "DEC" => {
                    // Decrement memory
                    let value = bus::read_u8(console, address);
                    let result = value.wrapping_sub(1);
                    bus::write_u8(console, address, result);

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "EOR" => {
                    // A ^ M
                    let acc = console.cpu.a;
                    let value = bus::read_u8(console, address);
                    let result = acc ^ value;
                    console.cpu.a = result;

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "INC" => {
                    // Increment memory
                    let value = bus::read_u8(console, address);
                    let result = value.wrapping_add(1);
                    bus::write_u8(console, address, result);

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "JMP" => {
                    // Jump to location
                    console.cpu.pc = address;
                }
                "JSR" => {
                    // Jump to subroutine. Push PC to the stack, and jump to address
                    push_stack_u16(console, console.cpu.pc - 1);
                    console.cpu.pc = address;
                }
                "LDA" => {
                    // Load value to A
                    let value = bus::read_u8(console, address);
                    load(value, &mut console.cpu.a, &mut console.cpu.flags);
                }
                "LDX" => {
                    // Load value to a register
                    let value = bus::read_u8(console, address);
                    console.cpu.x = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "LDY" => {
                    // Load value to a register
                    let value = bus::read_u8(console, address);
                    console.cpu.y = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "LSR" => {
                    let value = bus::read_u8(console, address);
                    let result = value >> 1;
                    bus::write_u8(console, address, result);

                    let carry = (value & 1) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    console.cpu.flags.set(Flags::CARRY, carry);
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "ORA" => {
                    let value = bus::read_u8(console, address);
                    let result = console.cpu.a | value;
                    console.cpu.a = result;

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "ROL" => {
                    // Rotate A left
                    // result_carry <- [7..0] <- carry
                    let value = bus::read_u8(console, address);
                    let carry_mask = if console.cpu.flags.contains(Flags::CARRY) {
                        1
                    } else {
                        0
                    };
                    let result = (value << 1) | carry_mask;
                    bus::write_u8(console, address, result);

                    let carry = (value & 0b1000_0000) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    console.cpu.flags.set(Flags::CARRY, carry);
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "ROR" => {
                    // Rotate A right
                    // carry -> [7..0] -> result_carry
                    let value = bus::read_u8(console, address);
                    let carry_mask = if console.cpu.flags.contains(Flags::CARRY) {
                        0b1000_0000
                    } else {
                        0
                    };
                    let result = (value >> 1) | carry_mask;
                    bus::write_u8(console, address, result);

                    let carry = (value & 1) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    console.cpu.flags.set(Flags::CARRY, carry);
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "SBC" => {
                    let acc_value = console.cpu.a;
                    let memory_value = bus::read_u8(console, address);
                    let carry = console.cpu.flags.contains(Flags::CARRY);

                    let (result, borrow) = acc_value.borrowing_sub(memory_value, !carry);
                    console.cpu.a = result;

                    let zero = result == 0;
                    let (_, overflow) = (acc_value as i8).borrowing_sub(memory_value as i8, !carry);
                    let negative = (result as i8) < 0;
                    console.cpu.flags.set(Flags::CARRY, !borrow);
                    console.cpu.flags.set(Flags::ZERO, zero);
                    console.cpu.flags.set(Flags::OVERFLOW, overflow);
                    console.cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "STA" => {
                    // Store A to memory
                    bus::write_u8(console, address, console.cpu.a);
                }
                "STX" => {
                    // Store X to memory
                    bus::write_u8(console, address, console.cpu.x);
                }
                "STY" => {
                    // Store Y to memory
                    bus::write_u8(console, address, console.cpu.y);
                }
                operation => {
                    todo!("{:?}", operation)
                }
            }
        }
    }

    Ok(())
}
