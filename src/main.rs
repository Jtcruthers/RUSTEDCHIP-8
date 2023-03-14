use std::{env, fs};

mod chip;
mod font;
mod display;

fn read_rom(rom_name: &String) -> Vec<u8> {
    fs::read(rom_name).expect("Cant read the rom")
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut chip = chip::Chip::new();

    let rom_name = &args[1];
    println!("Loading ROM {}", rom_name);
    let rom = read_rom(rom_name);

    chip.run(&rom);
}
