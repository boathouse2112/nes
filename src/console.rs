use crate::{bus::Bus, cpu::Cpu, memory::Memory};

pub struct Console {
    pub cpu: Cpu,
    pub bus: Bus,
}
