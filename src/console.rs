use crate::{bus::Bus, cpu::Cpu};

pub struct Console {
    pub cpu: Cpu,
    pub bus: Bus,
}
