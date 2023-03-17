#![feature(bigint_helper_methods)]
mod bus;
mod config;
mod console;
mod cpu;
mod instruction;
mod rom;
mod util;

use cpu::Cpu;
use simple_logger::SimpleLogger;
use std::fs;
use util::Error;

use crate::{bus::Bus, console::Console, rom::Rom};

fn main() -> Result<(), Error> {
    SimpleLogger::new().init().unwrap();

    // Load ROM
    let rom_bytes = fs::read("roms/snake.nes")?;
    let rom = Rom::new(&rom_bytes)?;

    let instructions = instruction::instructions();
    let mut console = Console {
        cpu: Cpu::new(),
        bus: Bus::new(rom),
    };

    // console.memory.load_rom();
    instruction::step(&mut console, &instructions)?;

    println!("Hello, world!");

    Ok(())
}
