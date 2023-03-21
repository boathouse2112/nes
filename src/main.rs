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
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};
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
            graphics.render(&mut console.ppu);
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

    // Init SDL2
    // let sdl_context = sdl2::init()?;
    // let video_subsystem = sdl_context.video()?;
    // let window = video_subsystem
    //     .window("NES", (256.0 * 2.0) as u32, (240.0 * 2.0) as u32)
    //     .position_centered()
    //     .build()?;

    // let mut canvas = window.into_canvas().present_vsync().build()?;
    // let mut event_pump = sdl_context.event_pump()?;
    // // canvas.set_scale(40.0, 40.0).unwrap();

    // let creator = canvas.texture_creator();
    // let mut texture =
    //     creator.create_texture_target(PixelFormatEnum::RGB24, (8 * 0x20), (8 * 30))?;

    // // Show tiles
    // // let tile_frame = graphics::show_tiles(&rom.character_rom);
    // // texture.update(None, &tile_frame.data, 256 * 3)?;
    // // canvas.copy(&texture, None, None)?;
    // // canvas.present();

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
