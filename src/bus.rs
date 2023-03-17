use crate::{
    config::{
        PPU_REGISTERS_MIRRORS_END, PPU_REGISTERS_START, PROGRAM_ROM_PAGE_SIZE, RAM_MIRRORS_END,
        RAM_START, ROM_END, ROM_START,
    },
    rom::Rom,
};

const CPU_VRAM_FIRST_MIRROR_MASK: u16 = 0b0000_0111_1111_1111;
const PPU_REGISTERS_FIRST_MIRROR_MASK: u16 = 0b0010_0000_0000_0111;

pub struct Bus {
    cpu_vram: [u8; 0x2048],
    rom: Rom,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        Bus {
            cpu_vram: [0; 0x2048],
            rom,
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.rom.program_rom[0..rom.len()].copy_from_slice(&rom[..]);
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
            ROM_START..=ROM_END => {
                let rom_address = address - ROM_START;
                let single_page_program_rom =
                    self.rom.program_rom.len() as u16 == PROGRAM_ROM_PAGE_SIZE;

                let first_mirror_rom_address =
                    if single_page_program_rom && rom_address >= PROGRAM_ROM_PAGE_SIZE {
                        rom_address % PROGRAM_ROM_PAGE_SIZE
                    } else {
                        rom_address
                    };

                self.rom.program_rom[first_mirror_rom_address as usize]
            }
            _ => {
                panic!("Invalid memory access at {:X}", address)
            }
        }
    }

    pub fn read_i8(&self, address: u16) -> i8 {
        self.read_u8(address) as i8
    }

    pub fn read_u16(&self, address: u16) -> u16 {
        let low_byte = self.read_u8(address);
        let high_byte = self.read_u8(address + 1);
        u16::from_le_bytes([low_byte, high_byte])
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
            ROM_START..=ROM_END => {
                panic!("Invalid attempt to write to ROM at {:X}", address)
            }
            _ => {
                panic!("Invalid memory access at {:X}", address)
            }
        }
    }

    pub fn write_i8(&mut self, address: u16, value: i8) {
        self.write_u8(address, value as u8)
    }

    pub fn write_u16(&mut self, address: u16, value: u16) {
        let [byte_1, byte_2] = value.to_le_bytes();
        self.write_u8(address, byte_1);
        self.write_u8(address + 1, byte_2);
    }
}
