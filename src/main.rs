use std::fs;

mod chip;
mod font;

fn get_rom() -> Vec<u8> {
    let filename =  "./ibm_logo.ch8";

    fs::read(filename).expect("Cannot read rom")
}

fn main() {
    let mut chip = chip::Chip::new();

    let rom = get_rom();

    chip.run(&rom);
}
