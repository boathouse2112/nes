use crate::{config::ROM_START, util::Error};

pub struct Memory {
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
