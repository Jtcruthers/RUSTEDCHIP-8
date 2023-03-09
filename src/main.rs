mod chip;

fn main() {
    let chip = chip::Chip::new();
    println!("Mem length: {:?}", chip.memory.len());
}
