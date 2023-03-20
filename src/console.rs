use crate::{bus::Bus, cpu::Cpu, ppu::Ppu, rom::Rom};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Console {
    pub cpu: Cpu,
    pub bus: Bus,
    pub ppu: Ppu,
    pub rom: Rom,
}
