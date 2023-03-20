use crate::config::{CHARACTER_ROM_PAGE_SIZE, PROGRAM_ROM_PAGE_SIZE};

const I_NES_IDENTIFIER_BYTES: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mapper {
    Zero,
}

impl TryFrom<u8> for Mapper {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Mapper::Zero),
            _ => Err("Unsupported mapper: {}".replace("{}", &value.to_string())),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Rom {
    pub program_rom: Vec<u8>,
    pub character_rom: Vec<u8>,
    pub mirroring: Mirroring,
    pub mapper: Mapper,
}

impl Rom {
    pub fn new(rom_bytes: &Vec<u8>) -> Result<Rom, String> {
        let i_nes_identifier_bytes = &rom_bytes[0..4];
        let program_rom_banks = rom_bytes[4];
        let character_rom_banks = rom_bytes[5];
        let control_byte_1 = rom_bytes[6];
        let control_byte_2 = rom_bytes[7];

        if i_nes_identifier_bytes != I_NES_IDENTIFIER_BYTES {
            return Err("Rom file is not an iNES file".to_string());
        }

        let mapper_byte = (control_byte_2 & 0xF0) | (control_byte_1 >> 4);
        let mapper = Mapper::try_from(mapper_byte)?;

        let i_nes_version = control_byte_2 & 0x0F;
        if i_nes_version != 0 {
            return Err("Only iNES v1.0 files are supported".to_string());
        }

        let four_screen_mirroring = ((control_byte_1 & 0x0F) >> 4) != 0;
        let vertical_mirroring = (control_byte_1 & 1) != 0;
        let mirroring = match (four_screen_mirroring, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let program_rom_size = program_rom_banks as usize * PROGRAM_ROM_PAGE_SIZE as usize;
        let character_rom_size = character_rom_banks as usize * CHARACTER_ROM_PAGE_SIZE as usize;

        let has_trainer = ((control_byte_1 & 0b0000_0100) >> 3) != 0;

        let program_rom_start = 16 + if has_trainer { 500 } else { 0 };
        let character_rom_start = program_rom_start + program_rom_size;

        Ok(Rom {
            program_rom: rom_bytes[program_rom_start..(program_rom_start + program_rom_size)]
                .to_vec(),
            character_rom: rom_bytes
                [character_rom_start..(character_rom_start + character_rom_size)]
                .to_vec(),
            mirroring,
            mapper,
        })
    }
}
