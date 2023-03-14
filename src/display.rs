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

    pub fn draw_sprite(&mut self, x_index: usize, y_index: usize, height: u8, sprite: Vec<u8>) -> bool {
        let mut starting_index = x_index + y_index * DISPLAY_WIDTH as usize;
        let mut flipped_pixel_to_off = false;

        for row in 0..height {
            // Each row is a byte in memory, so to get the next row, go to the next memory addr
            let pixel_pattern = sprite[row as usize];

            for offset in 0..8 {
                let pixel_index = starting_index + offset;
                if pixel_index >= 2048 {
                    break;
                }

                let pixel_bit = (pixel_pattern >> 7 - offset) & 1;
                let display_bit = self.get_pixel(pixel_index);
                let new_value = match (pixel_bit, display_bit) {
                    (0, false) => false,
                    (1, false) => true,
                    (0, true) => true,
                    (1, true) => {
                        flipped_pixel_to_off = true;
                        false
                    },
                    _ => panic!("This wasn't supposed to happen: {} {}", pixel_bit, display_bit)
                };

                self.set_pixel(pixel_index, new_value);
            }

            starting_index += DISPLAY_WIDTH;
        }

        self.print();
        flipped_pixel_to_off
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

