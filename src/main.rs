use std::{env, fs};

mod chip;
mod font;
mod display;
mod timer;

fn read_rom(rom_name: &String) -> Vec<u8> {
    fs::read(rom_name).expect("Cant read the rom")
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let rom_name = &args[1];
    let rom = read_rom(rom_name);
    let mut chip = chip::Chip::new();

    chip.load_rom(&rom);
    loop {
        chip.step();
    }
}
