use std::fs;
use clap::Parser;

mod chip;
mod font;
mod display;
mod timer;

use chip::{Chip, ChipType};
use display::window_conf;

#[derive(Parser, Debug)]
#[command(author = "Justin Carruthers", about = "Configurable CHIP-8 (and variants) emulator")]
struct Args {
    rom_name: String,

    #[arg(short, long, default_value_t = 1200)]
    target_instructions_per_second: u128,

    #[arg(short, long, value_enum, default_value_t = ChipType::CHIP8)]
    chip_type: ChipType,
}

fn read_rom(rom_name: &String) -> Vec<u8> {
    fs::read(rom_name).expect("Cant read the rom")
}

#[macroquad::main(window_conf)]
async fn main() {
    let args = Args::parse();

    let rom = read_rom(&args.rom_name);
    let mut chip = Chip::new(args.target_instructions_per_second, args.chip_type);

    chip.load_rom(&rom);
    loop {
        chip.step().await;
    }

}
