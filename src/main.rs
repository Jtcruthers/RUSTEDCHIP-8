use std::fs;
use clap::Parser;

mod chip;
mod font;
mod display;
mod timer;

use chip::{Chip, ChipType};
use display::{DisplayType, window_conf};

#[derive(Parser, Debug)]
#[command(author = "Justin Carruthers", about = "Configurable CHIP-8 (and variants) emulator")]
struct Args {
    rom_name: String,

    #[arg(short, long, default_value_t = 1200)]
    target_instructions_per_second: u128,

    #[arg(short, long, value_enum, default_value_t = ChipType::CHIP8)]
    chip_type: ChipType,

    #[arg(short, long, value_enum, default_value_t = DisplayType::Macroquad)]
    display_type: DisplayType,
}

#[macroquad::main(window_conf)]
async fn main() {
    let args = Args::parse();

    let rom = fs::read(&args.rom_name).expect("Cant read the rom");
    let mut chip = Chip::new(args.target_instructions_per_second, args.chip_type, args.display_type);

    chip.load_rom(&rom);
    loop {
        chip.step().await;
    }

}
