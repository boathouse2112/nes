use crate::{bus::Bus, cpu::Cpu};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Console {
    pub cpu: Cpu,
    pub bus: Bus,
}
