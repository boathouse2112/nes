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
use instruction::Instruction;
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};
use simple_logger::SimpleLogger;
use std::fs;
use util::Error;

use crate::{bus::Bus, console::Console, ppu::Ppu, rom::Rom};

fn run_with_callback<F>(
    console: &mut Console,
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

    // Init SDL2
    // let sdl_context = sdl2::init().unwrap();
    // let video_subsystem = sdl_context.video().unwrap();
    // let window = video_subsystem
    //     .window("Snake game", (32.0 * 10.0) as u32, (32.0 * 10.0) as u32)
    //     .position_centered()
    //     .build()
    //     .unwrap();

    // let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    // let mut event_pump = sdl_context.event_pump().unwrap();
    // canvas.set_scale(10.0, 10.0).unwrap();

    // let creator = canvas.texture_creator();
    // let mut texture = creator
    //     .create_texture_target(PixelFormatEnum::RGB24, (8 * 0x20), (8 * 30))
    //     .unwrap();

    // Load ROM
    // let rom_bytes = fs::read("roms/donkey_kong.nes")?;
    let rom_bytes = fs::read("roms/nestest.nes")?;
    let rom = Rom::new(&rom_bytes)?;
    let ppu = Ppu::new(&rom);

    // let tile_frame = graphics::show_tiles(&rom.character_rom);
    // texture.update(None, &tile_frame.data, 256 * 3).unwrap();
    // canvas.copy(&texture, None, None).unwrap();
    // canvas.present();

    // loop {
    //     for event in event_pump.poll_iter() {
    //         match event {
    //             Event::Quit { .. }
    //             | Event::KeyDown {
    //                 keycode: Some(Keycode::Escape),
    //                 ..
    //             } => std::process::exit(0),
    //             _ => { /* do nothing */ }
    //         }
    //     }
    // }

    let instructions = instruction::instructions();
    let mut console = Console {
        cpu: Cpu::new(),
        bus: Bus::new(),
        ppu,
        rom,
    };

    // console.memory.load_rom();
    run_with_callback(&mut console, &instructions, move |console, instruction| {
        println!("{}", debug::trace(console, instruction));
    })?;

    Ok(())
}
