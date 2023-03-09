#[derive(Debug)]
pub struct Timer {
    value: u8
}

impl Timer {
    pub fn new() -> Self {
        Self {
            value: 0
        }
    }

    pub fn set(&mut self, time: u8) {
        self.value = time;
    }

    pub fn get(&self) -> u8 {
        self.value
    }
}

const DISPLAY_SIZE: usize = 64 * 32;

#[derive(Debug)]
pub struct Chip {
    pub memory: [u8; 4096],
    pub stack: [u16; 32],
    pub display: [bool; DISPLAY_SIZE],
    pub delay_timer: Timer,
    pub sound_timer: Timer
}

impl Chip {
    pub fn new() -> Self {
        Chip {
            memory: [0; 4096],
            stack: [0; 32],
            display: [false; DISPLAY_SIZE],
            delay_timer: Timer::new(),
            sound_timer: Timer::new()
        }
    }

    fn fetch(&self) { }
    fn decode(&self) { }
    fn execute(&self) { }

    pub fn step(&self) {
        self.fetch();
        self.decode();
        self.execute();
    }

    pub fn run(&self) {
        loop {
            self.step()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_memory_is_4_zeroed_out_kilobytes() {
        let chip = Chip::new();
        assert_eq!(chip.memory, [0 as u8; 4096]);
    }

    #[test]
    fn initial_stack_is_32_zeroed_out_double_bytes() {
        let chip = Chip::new();
        assert_eq!(chip.stack.len(), 32);
        for stack_frame in chip.stack.iter() {
            assert_eq!(*stack_frame, 0x00000000 as u16)
        }
    }

    #[test]
    fn initial_display_is_64_by_32_pixels() {
        let chip = Chip::new();
        assert_eq!(chip.display.len(), 64 * 32);
    }

    #[test]
    fn initial_display_is_all_false() {
        let chip = Chip::new();
        assert_eq!(chip.display.iter().all(|pixel| *pixel == false), true);
    }

    #[test]
    fn timers_can_be_set_to_value() {
        let mut delay_timer = Chip::new().delay_timer;
        assert_eq!(delay_timer.get(), 0);

        delay_timer.set(255);
        assert_eq!(delay_timer.get(), 255);
    }
}
