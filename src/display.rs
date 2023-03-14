pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

pub struct Display {
    pub display: [bool; DISPLAY_SIZE],
}

impl Display {
    pub fn new() -> Self {
        Self {
            display: [false; DISPLAY_SIZE]
        }
    }

    pub fn clear(&mut self) {
        self.display = [false; DISPLAY_SIZE];
    }

    pub fn set_pixel(&mut self, pixel_index: usize, value: bool) {
        self.display[pixel_index] = value;
    }

    pub fn get_pixel(&self, pixel_index: usize) -> bool {
        self.display[pixel_index]
    }

    pub fn print(&self) {
        for _ in 0..DISPLAY_WIDTH {
            print!("-");
        }
        println!("");
        for row in 0..DISPLAY_HEIGHT {
            print!("|");
            for column in 0..DISPLAY_WIDTH {
                let pixel = DISPLAY_WIDTH * row + column;
                let sprite = if self.display[pixel] == true { "X" } else { " "};
                print!("{}", sprite);
            }
            println!("|");
        }
        for _ in 0..DISPLAY_WIDTH {
             print!("-");
        }
        println!("");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_display_is_64_by_32_pixels_all_empty() {
        let display = Display::new();
        assert_eq!(display.display, [false; 64 * 32]);
    }
}

