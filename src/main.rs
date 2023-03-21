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

fn run_with_callbacks<PostResetFn, CallbackFn>(
    console: &mut Console,
    graphics: &mut Graphics,
    instructions: &Vec<Instruction>,
    mut post_reset: PostResetFn,
    mut callback: CallbackFn,
) -> Result<(), Error>
where
    PostResetFn: FnMut(&mut Console),
    CallbackFn: FnMut(&mut Console, &Instruction),
{
    cpu::reset_interrupt(console);
    post_reset(console);

    loop {
        let opcode = bus::read_u8(console, console.cpu.pc);
        let instruction = instructions.iter().find(|&instr| instr.opcode == opcode);

        let instruction = if instruction.is_none() {
            todo!("Unimplemented opcode: 0x{:02X}", opcode);
        } else {
            instruction.unwrap()
        };

        if ppu::poll_nmi_status(&mut console.ppu) {
            cpu::nmi_interrupt(console);
            graphics.render(&mut console.ppu)?;
        }

        callback(console, instruction);
        cpu::step(console, instruction)?;
        console.ppu.tick(instruction.cycles as u32 * 3);
    }
}

fn main() -> Result<(), Error> {
    // Init logging
    SimpleLogger::new().init().unwrap();

    // Load ROM
    // let rom_bytes = fs::read("roms/donkey_kong.nes")?;
    let rom_bytes = fs::read("roms/nestest.nes")?;
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

    run_with_callbacks(
        &mut console,
        &mut graphics,
        &instructions,
        move |console| {
            // console.cpu.pc = 0xC000
        },
        move |console, instruction| {
            println!("{}", debug::trace(console, instruction));
        },
    )?;

    Ok(())
}
