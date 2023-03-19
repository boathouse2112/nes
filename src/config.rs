pub const RAM_START: u16 = 0x0000;
pub const RAM_MIRRORS_END: u16 = 0x1FFF;
pub const PPU_REGISTERS_START: u16 = 0x2000;
pub const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
pub const ROM_START: u16 = 0xC000;
pub const ROM_END: u16 = 0xFFFF;

pub const CPU_PAGE_SIZE: u16 = 256;
pub const PROGRAM_ROM_PAGE_SIZE: u16 = 1024 * 16;
pub const CHARACTER_ROM_PAGE_SIZE: u16 = 1024 * 8;

pub const CPU_SP_START_VALUE: u8 = 0xFD;
pub const CPU_FLAG_START_VALUE: u8 = 0b0010_0100;
