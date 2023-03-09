mod chip;

fn main() {
    let mut chip = chip::Chip::new();
    println!("Delay Timer: {:?}", chip.delay_timer.get());
    chip.delay_timer.set(40);
    println!("Delay Timer: {:?}", chip.delay_timer.get());
}
