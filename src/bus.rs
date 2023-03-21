use crate::{
    config::{CPU_PAGE_SIZE, PROGRAM_ROM_PAGE_SIZE},
    console::Console,
};

const RAM_START: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS_START: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const ROM_START: u16 = 0xC000;
const ROM_END: u16 = 0xFFFF;

const CPU_RAM_MIRROR_DOWN_MASK: u16 = 0b0000_0111_1111_1111;
const PPU_MIRROR_DOWN_MASK: u16 = 0b0010_0000_0000_0111;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Bus {
    cpu_ram: [u8; 2048],
}

impl Bus {
    pub fn new() -> Self {
        Bus { cpu_ram: [0; 2048] }
    }
}

pub fn read_u8(console: &mut Console, address: u16) -> u8 {
    match address {
        RAM_START..=RAM_MIRRORS_END => {
            let mirrored_down = address & CPU_RAM_MIRROR_DOWN_MASK;
            console.bus.cpu_ram[mirrored_down as usize]
        }
        PPU_REGISTERS_START..=PPU_REGISTERS_MIRRORS_END => {
            let mirrored_down = address & PPU_MIRROR_DOWN_MASK;

            match mirrored_down {
                    0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                        panic!("Attempt to read from write-only PPU address: {:04X}, mirrored down to : {:04X}", address, mirrored_down)
                    }
                    0x2002 => console.ppu.read_from_status(),
                    0x2004 => console.ppu.read_from_oam_data(),
                    0x2007 => console.ppu.read_from_data(),
                    _ => panic!(
                        "Attempt to read from invalid address in PPU range: {:04X}, mirrored-down to: {:04X}",
                        address,
                        mirrored_down
                    ),
                }
        }
        ROM_START..=ROM_END => {
            let rom_address = address - ROM_START;
            let single_page_program_rom =
                console.rom.program_rom.len() as u16 == PROGRAM_ROM_PAGE_SIZE;

            let first_mirror_rom_address =
                if single_page_program_rom && rom_address >= PROGRAM_ROM_PAGE_SIZE {
                    rom_address % PROGRAM_ROM_PAGE_SIZE
                } else {
                    rom_address
                };

            console.rom.program_rom[first_mirror_rom_address as usize]
        }
        _ => {
            panic!("Invalid attempt to read at {:X}", address)
        }
    }
}

pub fn read_i8(console: &mut Console, address: u16) -> i8 {
    read_u8(console, address) as i8
}

pub fn read_u16(console: &mut Console, address: u16) -> u16 {
    let low_byte = read_u8(console, address);
    let high_byte = read_u8(console, address + 1);
    u16::from_le_bytes([low_byte, high_byte])
}

pub fn read_u16_wrap_page(console: &mut Console, address: u16) -> u16 {
    let low_byte = read_u8(console, address);
    let page_start = (address / CPU_PAGE_SIZE) * CPU_PAGE_SIZE;
    let high_byte_address = page_start + ((address + 1) % CPU_PAGE_SIZE);
    let high_byte = read_u8(console, high_byte_address);
    u16::from_le_bytes([low_byte, high_byte])
}

pub fn write_u8(console: &mut Console, address: u16, value: u8) {
    match address {
        RAM_START..=RAM_MIRRORS_END => {
            let mirrored_down = address & CPU_RAM_MIRROR_DOWN_MASK;
            console.bus.cpu_ram[mirrored_down as usize] = value
        }
        PPU_REGISTERS_START..=PPU_REGISTERS_MIRRORS_END => {
            let mirrored_down = address & PPU_MIRROR_DOWN_MASK;
            match mirrored_down {
                    0x2002 => panic!("Attempt to write to read-only ppu address: {:40X}, mirrored-down to: {:40X}", address, mirrored_down),
                    0x2000 => console.ppu.write_to_control(value),
                    0x2001 => console.ppu.write_to_mask(value),
                    0x2003 => console.ppu.write_to_oam_address(value),
                    0x2004 => console.ppu.write_to_oam_data(value),
                    0x2005 => console.ppu.write_to_scroll(value),
                    0x2006 => console.ppu.write_to_vram_address(value),
                    0x2007 => console.ppu.write_to_data(value),
                    0x4014 => console.ppu.write_to_oam_dma(value, &console.bus.cpu_ram),
                    _ => panic!("Attempt to write to invalid address in ppu range: {:40X}, mirrored-down to: {:40X}", address, mirrored_down)
                }
        }
        ROM_START..=ROM_END => {
            panic!("Invalid attempt to write to ROM at {:X}", address)
        }
        _ => {
            panic!("Invalid attempt to write at {:X}", address)
        }
    }
}

pub fn write_u16(console: &mut Console, address: u16, value: u16) {
    let [low_byte, high_byte] = value.to_le_bytes();
    write_u8(console, address, low_byte);
    write_u8(console, address + 1, high_byte);
}
