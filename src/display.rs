use clap::ValueEnum;
use macroquad::prelude::*;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
pub const PIXEL_DIMENSION: f32 = 20.;

pub fn window_conf() -> Conf {
    Conf {
        window_title: "RUSTYCHIP-8".to_owned(),
        window_width: 1280,
        window_height: 640,
        ..Default::default()
    }
}

#[derive(ValueEnum, PartialEq, Clone, Debug)]
pub enum DisplayType {
    Macroquad,
    Terminal
}

pub struct Display {
    pub display: [bool; DISPLAY_SIZE],
    display_type: DisplayType
}

#[async_trait::async_trait]
trait MacroquadDisplay {
    async fn render(&self);
}

trait TerminalDisplay {
    fn render(&self);
}

#[async_trait::async_trait]
impl MacroquadDisplay for Display {
    async fn render(&self) {
        clear_background(BLACK);

        for row in 0..DISPLAY_HEIGHT {
            for column in 0..DISPLAY_WIDTH {
                let pixel = DISPLAY_WIDTH * row + column;
                let pixel_height = PIXEL_DIMENSION * row as f32;
                let pixel_width = PIXEL_DIMENSION * column as f32;

                let pixel_on_color = Color::new(255., 176., 0., 255.);
                let pixel_off_color = BLACK;
                let pixel_color = if self.display[pixel] == true { pixel_on_color } else { pixel_off_color };

                draw_rectangle(pixel_width, pixel_height, PIXEL_DIMENSION, PIXEL_DIMENSION, pixel_color);
            }
        }

        next_frame().await;
    }
}

impl TerminalDisplay for Display {
    fn render(&self) {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");

        let mut str_to_print = String::new();
        for _ in 0..DISPLAY_WIDTH {
            str_to_print.push_str("-");
        }
        str_to_print.push_str("\n");
        for row in 0..DISPLAY_HEIGHT {
            str_to_print.push_str("|");
            for column in 0..DISPLAY_WIDTH {
                let pixel = DISPLAY_WIDTH * row + column;
                let sprite = if self.display[pixel] == true { "X" } else { " " };
                str_to_print.push_str(sprite);
            }
            str_to_print.push_str("|\n");
        }
        for _ in 0..DISPLAY_WIDTH {
            str_to_print.push_str("-");
        }
        str_to_print.push_str("\n");

        println!("{}", str_to_print);
    }
}

impl Display {
    pub fn new(display_type: DisplayType) -> Self {
        Self {
            display: [false; DISPLAY_SIZE],
            display_type
        }
    }

    pub fn clear(&mut self) {
        self.display = [false; DISPLAY_SIZE];
    }

    pub fn set_pixel(&mut self, pixel_index: usize, value: bool) {
        let actual_index = pixel_index % DISPLAY_SIZE;
        self.display[actual_index] = value;
    }

    pub fn get_pixel(&self, pixel_index: usize) -> bool {
        let actual_index = pixel_index % DISPLAY_SIZE;
        self.display[actual_index]
    }

    pub async fn print(&self) {
        if self.display_type == DisplayType::Macroquad {
            MacroquadDisplay::render(self).await;
        } else {
            TerminalDisplay::render(self);
        }
    }

    pub async fn draw_sprite(&mut self, x_index: usize, y_index: usize, height: u8, sprite: Vec<u8>) -> bool {
        let mut starting_index = x_index + y_index * DISPLAY_WIDTH as usize;
        let mut flipped_pixel_to_off = false;

        for row in 0..height {
            // Each row is a byte in memory, so to get the next row, go to the next memory addr
            let pixel_pattern = sprite[row as usize];
            if row as usize + y_index >= DISPLAY_HEIGHT {
                break;
            }

            let max_x_in_row = (starting_index / DISPLAY_WIDTH) * DISPLAY_WIDTH + (DISPLAY_WIDTH - 1);
            for offset in 0..8 {
                let pixel_index = starting_index + offset;
                if pixel_index >= max_x_in_row {
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

        self.print().await;
        flipped_pixel_to_off
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_display_is_64_by_32_pixels_all_empty() {
        let display = Display::new(DisplayType::Terminal);
        assert_eq!(display.display, [false; 64 * 32]);
    }
}

