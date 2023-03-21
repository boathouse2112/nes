use crate::rom::{Mirroring, Rom};
use bitflags::bitflags;

const CHR_ROM_START: u16 = 0x0000;
const CHR_ROM_END: u16 = 0x1FFF;
const VRAM_START: u16 = 0x2000;
const VRAM_END: u16 = 0x2FFF;
const PALETTE_START: u16 = 0x3F00;
const PALETTE_END: u16 = 0x3FFF;

const NAMETABLE_SIZE: u16 = 0x400;

const ADDRESS_REGISTER_MIRROR_DOWN_MASK: u16 = 0b0011_1111_1111_1111; // [0x4000, 0xFFFF] -> [0, 0x4000)
const VRAM_MIRROR_DOWN_MASK: u16 = 0b0010_1111_1111_1111; // 0x3xxx -> 0x2xxx

bitflags! {

    // 7  bit  0
    // ---- ----
    // VPHB SINN
    // |||| ||||
    // |||| ||++- Base nametable address
    // |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
    // |||| |+--- VRAM address increment per CPU read/write of PPUDATA
    // |||| |     (0: add 1, going across; 1: add 32, going down)
    // |||| +---- Sprite pattern table address for 8x8 sprites
    // ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
    // |||+------ Background pattern table address (0: $0000; 1: $1000)
    // ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
    // |+-------- PPU master/slave select
    // |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
    // +--------- Generate an NMI at the start of the
    //            vertical blanking interval (0: off; 1: on)

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ControlRegister: u8 {
       const GENERATE_NMI               	= 0b1000_0000;
       const MASTER_SLAVE_SELECT        	= 0b0100_0000;
       const SPRITE_PATTERN_OFFSET          = 0b0010_0000;
       const BACKGROUND_PATTERN_OFFSET 	    = 0b0001_0000;
       const SPRITE_PATTERN_ADDRESS     	= 0b0000_1000;
       const VRAM_ADDRESS_INCREMENT     	= 0b0000_0100;
       const NAMETABLE_1                	= 0b0000_0010;
       const NAMETABLE_2                	= 0b0000_0001;
   }
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::from_bits_retain(0b0000_0000)
    }

    pub fn background_pattern_offset(&self) -> u16 {
        if self.contains(Self::BACKGROUND_PATTERN_OFFSET) {
            0x1000
        } else {
            0
        }
    }

    pub fn vram_address_increment_amount(&self) -> u8 {
        if self.contains(Self::VRAM_ADDRESS_INCREMENT) {
            32
        } else {
            1
        }
    }
}

bitflags! {

    // 7  bit  0
    // ---- ----
    // BGRs bMmG
    // |||| ||||
    // |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
    // |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
    // |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    // |||| +---- 1: Show background
    // |||+------ 1: Show sprites
    // ||+------- Emphasize red (green on PAL/Dendy)
    // |+-------- Emphasize green (red on PAL/Dendy)
    // +--------- Emphasize blue

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct MaskRegister: u8 {
       const EMPHASIZE_BLUE             = 0b1000_0000;
       const EMPHASIZE_GREEN        	= 0b0100_0000;
       const EMPHASIZE_RED              = 0b0010_0000;
       const SHOW_SPRITES 	            = 0b0001_0000;
       const SHOW_BACKGROUND     	    = 0b0000_1000;
       const LEFTMOST_8_SPRITES     	= 0b0000_0100;
       const LEFTMOST_8_BACKGROUND      = 0b0000_0010;
       const GREYSCALE                	= 0b0000_0001;
   }
}

impl MaskRegister {
    pub fn new() -> Self {
        MaskRegister::from_bits_retain(0)
    }
}

bitflags! {

    //     7  bit  0
    // ---- ----
    // VSO. ....
    // |||| ||||
    // |||+-++++- PPU open bus. Returns stale PPU bus contents.
    // ||+------- Sprite overflow. The intent was for this flag to be set
    // ||         whenever more than eight sprites appear on a scanline, but a
    // ||         hardware bug causes the actual behavior to be more complicated
    // ||         and generate false positives as well as false negatives; see
    // ||         PPU sprite evaluation. This flag is set during sprite
    // ||         evaluation and cleared at dot 1 (the second dot) of the
    // ||         pre-render line.
    // |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
    // |          a nonzero background pixel; cleared at dot 1 of the pre-render
    // |          line.  Used for raster timing.
    // +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
    //            Set at dot 1 of line 241 (the line *after* the post-render
    //            line); cleared after reading $2002 and at dot 1 of the
    //            pre-render line.

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct StatusRegister: u8 {
       const VBLANK_STARTED     = 0b1000_0000;
       const B        	        = 0b0100_0000;
       const C                  = 0b0010_0000;
       const D 	                = 0b0001_0000;
       const E     	            = 0b0000_1000;
       const F     	            = 0b0000_0100;
       const G                  = 0b0000_0010;
       const H                	= 0b0000_0001;
   }
}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister::from_bits_retain(0)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ScrollRegister {
    x_scroll: u8,
    y_scroll: u8,
    x_scroll_active: bool,
}

impl ScrollRegister {
    pub fn new() -> Self {
        ScrollRegister {
            x_scroll: 0,
            y_scroll: 0,
            x_scroll_active: true,
        }
    }

    /**
     * If x_scroll_active, sets x_scroll to the given value.
     * If not, sets y_scroll to the given value.
     * Toggles x_scroll_active
     */
    pub fn update(&mut self, value: u8) {
        if self.x_scroll_active {
            self.x_scroll = value;
        } else {
            self.y_scroll = value;
        };
        self.x_scroll_active = !self.x_scroll_active;
    }

    pub fn reset_latch(&mut self) {
        self.x_scroll_active = true;
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AddressRegister {
    high_byte: u8,
    low_byte: u8,
    high_byte_active: bool,
}

impl AddressRegister {
    pub fn new() -> Self {
        AddressRegister {
            high_byte: 0,
            low_byte: 0,
            high_byte_active: true,
        }
    }

    /**
     * If high_byte_active, sets high_byte to the given value.
     * If not, sets low_byte to the given value.
     * Mirrors down the u16 address containing the updated byte.
     */
    pub fn update(&mut self, value: u8) {
        if self.high_byte_active {
            self.high_byte = value;
        } else {
            self.low_byte = value;
        }
        self.high_byte_active = !self.high_byte_active;

        self.mirror_down();
    }

    /**
     * Increments the address by the given amount. Wraps u16.
     * Mirrors down the new value.
     */
    pub fn increment(&mut self, amount: u8) {
        let value = self.get().wrapping_add(amount as u16);
        self.set(value);

        self.mirror_down();
    }

    pub fn reset_latch(&mut self) {
        self.high_byte_active = true;
    }

    fn get(&self) -> u16 {
        u16::from_be_bytes([self.high_byte, self.low_byte])
    }

    fn set(&mut self, value: u16) {
        let [high_byte, low_byte] = value.to_be_bytes();
        self.high_byte = high_byte;
        self.low_byte = low_byte;
    }

    /**
     * Set the address to the lowest-mirror possibility.
     */
    fn mirror_down(&mut self) {
        let mirror_down = self.get() & ADDRESS_REGISTER_MIRROR_DOWN_MASK;
        self.set(mirror_down);
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Ppu {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam: [u8; 256],
    pub mirroring: Mirroring,

    pub control: ControlRegister,
    pub mask: MaskRegister,
    pub status: StatusRegister,
    pub oam_address: u8,
    pub scroll: ScrollRegister,
    pub vram_address: AddressRegister,

    pub nmi_interrupt: bool,

    data_buffer: u8,
    cycles: u32,
    scanline: u32,
}

impl Ppu {
    pub fn new(rom: &Rom) -> Self {
        Ppu {
            chr_rom: rom.character_rom.clone(),
            palette_table: [0; 32],
            vram: [0; 2048],
            oam: [0; 256],

            mirroring: rom.mirroring,

            control: ControlRegister::new(),
            mask: MaskRegister::new(),
            status: StatusRegister::new(),
            oam_address: 0,
            scroll: ScrollRegister::new(),
            vram_address: AddressRegister::new(),

            nmi_interrupt: false,

            data_buffer: 0,
            cycles: 0,
            scanline: 0,
        }
    }

    /**
     * Writes to bus::$2000
     */
    pub fn write_to_control(&mut self, value: u8) {
        let nmi_before_write = self.control.contains(ControlRegister::GENERATE_NMI);
        self.control = ControlRegister::from_bits_retain(value);
        let nmi_after_write = self.control.contains(ControlRegister::GENERATE_NMI);
        if !nmi_before_write
            && nmi_after_write
            && self.status.contains(StatusRegister::VBLANK_STARTED)
        {
            self.nmi_interrupt = true;
        }
    }

    /**
     * Writes to bus::$2001
     */
    pub fn write_to_mask(&mut self, value: u8) {
        self.mask = MaskRegister::from_bits_retain(value);
    }

    pub fn write_to_vram_address(&mut self, value: u8) {
        self.vram_address.update(value);
    }

    /**
     * Writes to bus::$2003
     */
    pub fn write_to_oam_address(&mut self, value: u8) {
        self.oam_address = value;
    }

    /**
     * Writes to bus::$2004
     * Increments oam_address
     */
    pub fn write_to_oam_data(&mut self, value: u8) {
        self.oam[self.oam_address as usize] = value;
        self.oam_address = self.oam_address.wrapping_add(1);
    }

    /**
     * Writes to bus::$2005
     */
    pub fn write_to_scroll(&mut self, value: u8) {
        self.scroll.update(value);
    }

    /**
     * Writes to bus::$2007
     * Increments vram based on bit 2 of bus::$2000
     */
    pub fn write_to_data(&mut self, value: u8) {
        let address = self.vram_address.get();
        self.vram[address as usize] = value;
        self.increment_address();
    }

    /**
     * Writes to bus::$4014
     */
    pub fn write_to_oam_dma(&mut self, value: u8, cpu_ram: &[u8; 2048]) {
        let cpu_page_start: usize = (value as usize) << 8;
        let cpu_page_end: usize = ((value as usize) << 8) & 0xFF;
        let cpu_ram_slice = &cpu_ram[cpu_page_start..=cpu_page_end];
        self.oam.copy_from_slice(cpu_ram_slice);
    }

    /**
     * Reads data from bus::$2002
     */
    pub fn read_from_status(&mut self) -> u8 {
        self.status.bits()
    }

    /**
     * Reads from bus::$2004
     */
    pub fn read_from_oam_data(&self) -> u8 {
        self.oam[self.oam_address as usize]
    }

    /**
     * Reads data from bus::$2007
     * Increments vram based on bit 2 of bus::$2000
     */
    pub fn read_from_data(&mut self) -> u8 {
        let address = self.vram_address.get();
        self.increment_address();

        match address {
            CHR_ROM_START..=CHR_ROM_END => {
                let result = self.data_buffer;
                self.data_buffer = self.chr_rom[address as usize];
                result
            }
            VRAM_START..=VRAM_END => {
                let result = self.data_buffer;
                let mirror_down_vram_address = self.mirror_down_vram(address);
                self.data_buffer = self.vram[mirror_down_vram_address as usize];
                result
            }
            0x3000..=0x3EFF => panic!("Attempt to read from unused PPU address: {:04X}", address),
            0x3F00..=0x3FFF => self.palette_table[(address - 0x3f00) as usize],
            _ => panic!("Attempt to read from mirrored PPU address: {:04X}", address),
        }
    }

    pub fn tick(&mut self, cycles: u32) -> bool {
        self.cycles += cycles;
        if self.cycles >= 341 {
            self.cycles -= 341;
            self.scanline += 1;
        }

        if self.scanline == 241 {
            if self.control.contains(ControlRegister::GENERATE_NMI) {
                self.status.set(StatusRegister::VBLANK_STARTED, true);
                self.nmi_interrupt = true;
                todo!("Trigger NMI interrupt");
            }
        }

        if self.scanline >= 262 {
            self.scanline = 0;
            self.status.set(StatusRegister::VBLANK_STARTED, false);
            return true;
        }

        false
    }

    fn increment_address(&mut self) {
        self.vram_address
            .increment(self.control.vram_address_increment_amount());
    }

    fn mirror_down_vram(&self, address: u16) -> u16 {
        let vram_index = address - VRAM_START;
        let nametable_index = vram_index / NAMETABLE_SIZE;
        let nametable_offset = address % NAMETABLE_SIZE;
        let nametable_start = match (self.mirroring, nametable_index) {
            (Mirroring::Horizontal, 0 | 1) => 0,
            (Mirroring::Horizontal, 2 | 3) => NAMETABLE_SIZE,
            (Mirroring::Vertical, 0 | 3) => 0,
            (Mirroring::Vertical, 1 | 4) => NAMETABLE_SIZE,
            _ => panic!("Nametable index >3: {:}", nametable_index),
        };
        nametable_start + nametable_offset
    }
}

pub fn poll_nmi_status(ppu: &mut Ppu) -> bool {
    if ppu.nmi_interrupt {
        ppu.nmi_interrupt = false;
        true
    } else {
        false
    }
}
