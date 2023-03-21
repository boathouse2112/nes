#![feature(bigint_helper_methods)]
mod bus;
mod config;
mod console;
mod cpu;
mod debug;
mod graphics;
mod instruction;
mod palette;
mod ppu;
mod rom;
mod util;

use cpu::Cpu;
use graphics::Graphics;
use instruction::Instruction;
use simple_logger::SimpleLogger;
use std::fs;
use util::Error;

use crate::{bus::Bus, console::Console, ppu::Ppu, rom::Rom};

fn run_with_callback<F>(
    console: &mut Console,
    graphics: &mut Graphics,
    instructions: &Vec<Instruction>,
    mut callback: F,
) -> Result<(), Error>
where
    F: FnMut(&mut Console, &Instruction),
{
    let mut cpu_cycles: u32 = 0;
    loop {
        let opcode = bus::read_u8(console, console.cpu.pc);
        let instruction = instructions.iter().find(|&instr| instr.opcode == opcode);

        let instruction = if instruction.is_none() {
            todo!("Unimplemented opcode: 0x{:02X}", opcode);
        } else {
            instruction.unwrap()
        };

        if ppu::poll_nmi_status(&mut console.ppu) {
            cpu::interrupt_nmi(console);
            graphics.render(&mut console.ppu)?;
        }

        callback(console, instruction);
        cpu::step(console, instruction)?;
        cpu_cycles += instruction.cycles as u32;
        console.ppu.tick(cpu_cycles * 3);
    }
}

fn main() -> Result<(), Error> {
    // Init logging
    SimpleLogger::new().init().unwrap();

    // Load ROM
    let rom_bytes = fs::read("roms/donkey_kong.nes")?;
    // let rom_bytes = fs::read("roms/nestest.nes")?;
    let rom = Rom::new(&rom_bytes)?;
    let ppu = Ppu::new(&rom);

    // Init console
    let instructions = instruction::instructions();
    let mut console = Console {
        cpu: Cpu::new(),
        bus: Bus::new(),
        ppu,
        rom,
    };

    // Init graphics
    let mut graphics = Graphics::new()?;

    run_with_callback(
        &mut console,
        &mut graphics,
        &instructions,
        move |console, instruction| {
            println!("{}", debug::trace(console, instruction));
        },
    )?;

    Ok(())
}
