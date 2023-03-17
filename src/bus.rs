use crate::{config::ROM_START, util::Error};

const RAM_START: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS_START: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

const CPU_VRAM_FIRST_MIRROR_MASK: u16 = 0b0000_0111_1111_1111;
const PPU_REGISTERS_FIRST_MIRROR_MASK: u16 = 0b0010_0000_0000_0111;

pub struct Bus {
    cpu_vram: [u8; 0x2048],
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            cpu_vram: [0; 0x2048],
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.cpu_vram[ROM_START..(ROM_START + rom.len())].copy_from_slice(&rom[..]);
    }

    pub fn read_u8(&self, address: u16) -> u8 {
        match address {
            RAM_START..=RAM_MIRRORS_END => {
                let first_mirror_address = address & CPU_VRAM_FIRST_MIRROR_MASK;
                self.cpu_vram[first_mirror_address as usize]
            }
            PPU_REGISTERS_START..=PPU_REGISTERS_MIRRORS_END => {
                let first_mirror_address = address & PPU_REGISTERS_FIRST_MIRROR_MASK;
                todo!("Support PPU");
            }
            _ => {
                panic!("Invalid memory access at {}", address)
            }
        }
    }

    pub fn read_i8(&self, address: u16) -> i8 {
        self.read_u8(address) as i8
    }

    pub fn read_u16(&self, address: u16) -> u16 {
        match address {
            // address can't cross a memory-map boundary
            RAM_START..=RAM_MIRRORS_END if address < RAM_MIRRORS_END => {
                let first_mirror_address = (address & CPU_VRAM_FIRST_MIRROR_MASK) as usize;
                let value_slice: &[u8] =
                    &self.cpu_vram[first_mirror_address..first_mirror_address + 2];
                let value_bytes_little_endian: [u8; 2] = value_slice.try_into().unwrap();
                u16::from_le_bytes(value_bytes_little_endian)
            }
            PPU_REGISTERS_START..=PPU_REGISTERS_MIRRORS_END
                if address < PPU_REGISTERS_MIRRORS_END =>
            {
                let first_mirror_address = address & PPU_REGISTERS_FIRST_MIRROR_MASK;
                todo!("Support PPU");
            }
            _ => {
                panic!("Invalid memory access at {}", address)
            }
        }
    }

    pub fn write_u8(&mut self, address: u16, value: u8) {
        match address {
            RAM_START..=RAM_MIRRORS_END => {
                let first_mirror_address = address & CPU_VRAM_FIRST_MIRROR_MASK;
                self.cpu_vram[first_mirror_address as usize] = value
            }
            PPU_REGISTERS_START..=PPU_REGISTERS_MIRRORS_END => {
                let first_mirror_address = address & PPU_REGISTERS_FIRST_MIRROR_MASK;
                todo!("Support PPU")
            }
            _ => {
                panic!("Invalid memory access at {}", address)
            }
        }
    }

    pub fn write_i8(&mut self, address: u16, value: i8) {
        self.write_u8(address, value as u8)
    }

    pub fn write_u16(&mut self, address: u16, value: u16) {
        match address {
            // address can't cross a memory-map boundary
            RAM_START..=RAM_MIRRORS_END if address < RAM_MIRRORS_END => {
                let first_mirror_address = (address & CPU_VRAM_FIRST_MIRROR_MASK) as usize;
                let little_endian = value.to_le_bytes();
                self.cpu_vram[first_mirror_address..first_mirror_address + 2]
                    .copy_from_slice(&little_endian);
            }
            PPU_REGISTERS_START..=PPU_REGISTERS_MIRRORS_END
                if address < PPU_REGISTERS_MIRRORS_END =>
            {
                let first_mirror_address = address & PPU_REGISTERS_FIRST_MIRROR_MASK;
                todo!("Support PPU");
            }
            _ => {
                panic!("Invalid memory access at {}", address)
            }
        }
    }
}
