use crate::{
    bus::Bus,
    config::{CPU_FLAG_START_VALUE, CPU_SP_START_VALUE},
    console::Console,
    instruction::{AddressingMode, Instruction},
    util::Error,
};
use bitflags::bitflags;

const ROM_START: u16 = 0xC000;

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

pub fn interrupt_nmi(Console { cpu, bus }: &mut Console) {
    cpu.push_stack_u16(bus, cpu.pc);
    let mut flags = cpu.flags.clone();
    flags = flags.union(Flags::BREAK).difference(Flags::BREAK_2);

    cpu.push_stack_u8(bus, flags.bits());
    cpu.flags.insert(Flags::INTERRUPT_DISABLE);

    bus.tick(2);
    cpu.pc = bus.read_u16(0xFFFA);
}

pub fn step(Console { cpu, bus }: &mut Console, instruction: &Instruction) -> Result<(), Error> {
    // Logs instruction name
    fn read_address(cpu: &mut Cpu, bus: &mut Bus, mode: AddressingMode) -> Result<u16, Error> {
        match mode {
            AddressingMode::Immediate => {
                let address = cpu.pc;
                cpu.pc += 1;

                Ok(address)
            }
            AddressingMode::ZeroPage => {
                let address = bus.read_u8(cpu.pc);
                cpu.pc += 1;

                Ok(address as u16)
            }
            AddressingMode::ZeroPageX => {
                let mut address = bus.read_u8(cpu.pc);
                cpu.pc += 1;

                address = address.wrapping_add(cpu.x);
                Ok(address as u16)
            }
            AddressingMode::ZeroPageY => {
                let mut address = bus.read_u8(cpu.pc);
                cpu.pc += 1;

                address = address.wrapping_add(cpu.y);
                Ok(address as u16)
            }
            AddressingMode::Relative => {
                let address = cpu.pc;
                cpu.pc += 1;

                Ok(address)
            }
            AddressingMode::Absolute => {
                let address = bus.read_u16(cpu.pc);
                cpu.pc += 2;

                Ok(address)
            }
            AddressingMode::AbsoluteX => {
                let mut address = bus.read_u16(cpu.pc);
                cpu.pc += 2;

                address = address.wrapping_add(cpu.x as u16);
                Ok(address)
            }
            AddressingMode::AbsoluteY => {
                let mut address = bus.read_u16(cpu.pc);
                cpu.pc += 2;

                address = address.wrapping_add(cpu.y as u16);
                Ok(address)
            }
            AddressingMode::Indirect => {
                let indirect_address = bus.read_u16(cpu.pc);
                cpu.pc += 2;

                let address = bus.read_u16_wrap_page(indirect_address);
                Ok(address)
            }
            AddressingMode::IndirectX => {
                let mut indirect_address = bus.read_u8(cpu.pc);
                cpu.pc += 1;

                // Read the final address from memory[indirect_address + x]
                indirect_address = indirect_address.wrapping_add(cpu.x);
                let address = bus.read_u16_wrap_page(indirect_address as u16);
                Ok(address)
            }
            AddressingMode::IndirectY => {
                let indirect_address = bus.read_u8(cpu.pc);
                cpu.pc += 1;

                // The final address is (memory[indirect_address]) + y
                let mut address = bus.read_u16_wrap_page(indirect_address as u16);
                address = address.wrapping_add(cpu.y as u16);
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
     * Branch (add offset to cpu.pc) if the condition is true
     *  N Z C I D V
     *  - - - - - -
     */
    fn branch(condition: bool, offset: i8, cpu: &mut Cpu) {
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

    cpu.pc += 1; // Increment for opcode read in main.rs

    match instruction.addressing_mode {
        AddressingMode::None => {
            // Execute immediately.

            match instruction.operation {
                "ASL" => {
                    let value = cpu.a;
                    let result = value << 1;
                    cpu.a = result;

                    let carry = (value & 0b1000_0000) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.flags.set(Flags::CARRY, carry);
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "BRK" => {
                    cpu.push_stack_u16(bus, cpu.pc);
                    cpu.push_stack_u8(bus, cpu.flags.bits());
                    cpu.pc = bus.read_u16(0xFFFE);
                    cpu.flags.set(Flags::BREAK, true);
                }
                "CLC" => {
                    cpu.flags.set(Flags::CARRY, false);
                }
                "CLD" => {
                    cpu.flags.set(Flags::DECIMAL, false);
                }
                "CLI" => {
                    cpu.flags.set(Flags::INTERRUPT_DISABLE, false);
                }
                "CLV" => {
                    cpu.flags.set(Flags::OVERFLOW, false);
                }
                "DEX" => decrement(&mut cpu.x, &mut cpu.flags),
                "DEY" => decrement(&mut cpu.y, &mut cpu.flags),
                "INX" => increment(&mut cpu.x, &mut cpu.flags),
                "INY" => increment(&mut cpu.y, &mut cpu.flags),
                "LSR" => {
                    let value = cpu.a;
                    let result = value >> 1;
                    cpu.a = result;

                    let carry = (value & 1) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.flags.set(Flags::CARRY, carry);
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "NOP" => {}
                "PHA" => {
                    // Push A to stack
                    cpu.push_stack_u8(bus, cpu.a);
                }
                "PHP" => {
                    // Push flags to stack
                    // Pushes with bits 5 and 4 true
                    let flags_to_push = cpu.flags.union(Flags::BREAK | Flags::BREAK_2).bits();
                    cpu.push_stack_u8(bus, flags_to_push);
                }
                "PLA" => {
                    // Pull stack to A
                    let value = cpu.pull_stack_u8(bus)?;
                    cpu.a = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "PLP" => {
                    // Pull stack to flags
                    // Sets bit 5 to 1, bit 4 to 0
                    let pulled_flags = Flags::from_bits_retain(cpu.pull_stack_u8(bus)?);
                    let flags = pulled_flags.union(Flags::BREAK).difference(Flags::BREAK_2);
                    cpu.flags = flags;
                }
                "ROL" => {
                    // Rotate A left
                    // result_carry <- [7..0] <- carry
                    let value = cpu.a;
                    let carry_mask = if cpu.flags.contains(Flags::CARRY) {
                        1
                    } else {
                        0
                    };
                    let result = (value << 1) | carry_mask;
                    cpu.a = result;

                    let carry = (value & 0b1000_0000) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.flags.set(Flags::CARRY, carry);
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "ROR" => {
                    // Rotate A right
                    // carry -> [7..0] -> result_carry
                    let value = cpu.a;
                    let carry_mask = if cpu.flags.contains(Flags::CARRY) {
                        0b1000_0000
                    } else {
                        0
                    };
                    let result = (value >> 1) | carry_mask;
                    cpu.a = result;

                    let carry = (value & 1) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.flags.set(Flags::CARRY, carry);
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "RTI" => {
                    // Sets bit 5 to 1, bit 4 to 0
                    let pulled_flags = Flags::from_bits_retain(cpu.pull_stack_u8(bus)?);
                    let flags = pulled_flags.union(Flags::BREAK).difference(Flags::BREAK_2);
                    cpu.flags = flags;
                    cpu.pc = cpu.pull_stack_u16(bus)?;
                }
                "RTS" => {
                    cpu.pc = cpu.pull_stack_u16(bus)?;
                    cpu.pc += 1;
                }
                "SEC" => {
                    cpu.flags.set(Flags::CARRY, true);
                }
                "SED" => {
                    cpu.flags.set(Flags::DECIMAL, true);
                }
                "SEI" => {
                    cpu.flags.set(Flags::INTERRUPT_DISABLE, true);
                }
                "TAX" => {
                    let value = cpu.a;
                    cpu.x = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "TAY" => {
                    let value = cpu.a;
                    cpu.y = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "TSX" => {
                    let value = cpu.sp;
                    cpu.x = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "TXA" => {
                    let value = cpu.x;
                    cpu.a = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "TXS" => {
                    cpu.sp = cpu.x;
                }
                "TYA" => {
                    let value = cpu.y;
                    cpu.a = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                operation => {
                    todo!("{:?}", operation)
                }
            }
        }
        // Load a value based on the addressing mode, and then execute
        _ => {
            let address = read_address(cpu, bus, instruction.addressing_mode)?;

            match instruction.operation {
                "ADC" => {
                    let acc_value = cpu.a;
                    let memory_value = bus.read_u8(address);
                    let carry = cpu.flags.contains(Flags::CARRY);

                    let (result, result_carry) = acc_value.carrying_add(memory_value, carry);
                    cpu.a = result;

                    let zero = result == 0;
                    let overflow = (acc_value as i8).checked_add(memory_value as i8).is_none();
                    let negative = (result as i8) < 0;
                    cpu.flags.set(Flags::CARRY, result_carry);
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::OVERFLOW, overflow);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "AND" => {
                    let value = bus.read_u8(address);
                    let result = cpu.a & value;
                    cpu.a = result;

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "ASL" => {
                    // Shift bits left 1
                    let value = bus.read_u8(address);
                    let result = value << 1;
                    bus.write_u8(address, result);

                    let carry = (value & 0b1000_0000) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.flags.set(Flags::CARRY, carry);
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "BCC" => {
                    // Branch if carry flag is clear
                    let offset = bus.read_i8(address);
                    branch(!cpu.flags.contains(Flags::CARRY), offset, cpu);
                }
                "BCS" => {
                    // Branch if carry flag is set
                    let offset = bus.read_i8(address);
                    branch(cpu.flags.contains(Flags::CARRY), offset, cpu);
                }
                "BEQ" => {
                    // Branch if zero flag is set
                    let offset = bus.read_i8(address);
                    branch(cpu.flags.contains(Flags::ZERO), offset, cpu);
                }
                "BIT" => {
                    // Set zero flag to (A AND value) == 0
                    let value = bus.read_u8(address);
                    let result = cpu.a & value;

                    let zero = result == 0;
                    let overflow = (value & 0b0100_0000) != 0; // Overflow -> bit 6
                    let negative = (value & 0b1000_0000) != 0; // Negative -> bit 7
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::OVERFLOW, overflow);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "BMI" => {
                    // Branch if negative flag is set
                    let offset = bus.read_i8(address);
                    branch(cpu.flags.contains(Flags::NEGATIVE), offset, cpu);
                }
                "BNE" => {
                    // Branch if zero flag is clear
                    let offset = bus.read_i8(address);
                    branch(!cpu.flags.contains(Flags::ZERO), offset, cpu);
                }
                "BPL" => {
                    // Branch if negative flag is clear
                    let offset = bus.read_i8(address);
                    branch(!cpu.flags.contains(Flags::NEGATIVE), offset, cpu);
                }
                "BVC" => {
                    // Branch if overflow flag is clear
                    let offset = bus.read_i8(address);
                    branch(!cpu.flags.contains(Flags::OVERFLOW), offset, cpu);
                }
                "BVS" => {
                    // Branch if overflow flag is set
                    let offset = bus.read_i8(address);
                    branch(cpu.flags.contains(Flags::OVERFLOW), offset, cpu);
                }
                "CMP" => {
                    // Set flags based on A - M
                    let memory_value = bus.read_u8(address);
                    compare(cpu.a, memory_value, &mut cpu.flags);
                }
                "CPX" => {
                    // Set flags based on X - M
                    let memory_value = bus.read_u8(address);
                    compare(cpu.x, memory_value, &mut cpu.flags);
                }
                "CPY" => {
                    // Set flags based on Y - M
                    let memory_value = bus.read_u8(address);
                    compare(cpu.y, memory_value, &mut cpu.flags);
                }
                "DEC" => {
                    // Decrement memory
                    let value = bus.read_u8(address);
                    let result = value.wrapping_sub(1);
                    bus.write_u8(address, result);

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "EOR" => {
                    // A ^ M
                    let acc = cpu.a;
                    let value = bus.read_u8(address);
                    let result = acc ^ value;
                    cpu.a = result;

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "INC" => {
                    // Increment memory
                    let value = bus.read_u8(address);
                    let result = value.wrapping_add(1);
                    bus.write_u8(address, result);

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "JMP" => {
                    // Jump to location
                    cpu.pc = address;
                }
                "JSR" => {
                    // Jump to subroutine. Push PC to the stack, and jump to address
                    cpu.push_stack_u16(bus, cpu.pc - 1);
                    cpu.pc = address;
                }
                "LDA" => {
                    // Load value to A
                    let value = bus.read_u8(address);
                    load(value, &mut cpu.a, &mut cpu.flags);
                }
                "LDX" => {
                    // Load value to a register
                    let value = bus.read_u8(address);
                    cpu.x = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "LDY" => {
                    // Load value to a register
                    let value = bus.read_u8(address);
                    cpu.y = value;

                    let zero = value == 0;
                    let negative = (value as i8) < 0;
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "LSR" => {
                    let value = bus.read_u8(address);
                    let result = value >> 1;
                    bus.write_u8(address, result);

                    let carry = (value & 1) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.flags.set(Flags::CARRY, carry);
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "ORA" => {
                    let value = bus.read_u8(address);
                    let result = cpu.a | value;
                    cpu.a = result;

                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "ROL" => {
                    // Rotate A left
                    // result_carry <- [7..0] <- carry
                    let value = bus.read_u8(address);
                    let carry_mask = if cpu.flags.contains(Flags::CARRY) {
                        1
                    } else {
                        0
                    };
                    let result = (value << 1) | carry_mask;
                    bus.write_u8(address, result);

                    let carry = (value & 0b1000_0000) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.flags.set(Flags::CARRY, carry);
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "ROR" => {
                    // Rotate A right
                    // carry -> [7..0] -> result_carry
                    let value = bus.read_u8(address);
                    let carry_mask = if cpu.flags.contains(Flags::CARRY) {
                        0b1000_0000
                    } else {
                        0
                    };
                    let result = (value >> 1) | carry_mask;
                    bus.write_u8(address, result);

                    let carry = (value & 1) != 0;
                    let zero = result == 0;
                    let negative = (result as i8) < 0;
                    cpu.flags.set(Flags::CARRY, carry);
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "SBC" => {
                    let acc_value = cpu.a;
                    let memory_value = bus.read_u8(address);
                    let carry = cpu.flags.contains(Flags::CARRY);

                    let (result, borrow) = acc_value.borrowing_sub(memory_value, !carry);
                    cpu.a = result;

                    let zero = result == 0;
                    let (_, overflow) = (acc_value as i8).borrowing_sub(memory_value as i8, !carry);
                    let negative = (result as i8) < 0;
                    cpu.flags.set(Flags::CARRY, !borrow);
                    cpu.flags.set(Flags::ZERO, zero);
                    cpu.flags.set(Flags::OVERFLOW, overflow);
                    cpu.flags.set(Flags::NEGATIVE, negative);
                }
                "STA" => {
                    // Store A to memory
                    bus.write_u8(address, cpu.a);
                }
                "STX" => {
                    // Store X to memory
                    bus.write_u8(address, cpu.x);
                }
                "STY" => {
                    // Store Y to memory
                    bus.write_u8(address, cpu.y);
                }
                operation => {
                    todo!("{:?}", operation)
                }
            }
        }
    }

    Ok(())
}
