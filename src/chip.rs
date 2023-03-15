use rand::Rng;
use crate::font;
use crate::display::Display;
use crate::timer::Timer;
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::process;

const CHIP8: &str = "CHIP-8";
const SUPERCHIP: &str = "S-CHIP";
const FONT_ADDR: usize = 0x050;
const ROM_ADDR: usize = 0x200;
const TIMER_HZ: u8 = 60;

pub struct DecodedInstruction {
    nibbles: [u8; 4],
    nn: u8,
    nnn: usize
}

pub struct Chip {
    pub memory: [u8; 4096],
    pub registers: [u8; 16],
    pub stack: [usize; 32],
    pub stack_level: usize,
    pub display: Display,
    pub delay_timer: Timer,
    pub sound_timer: Timer,
    pub i: usize,
    pub pc: usize,
    pub chip_type: String
}

impl Chip {
    pub fn new() -> Self {
        let mut chip = Chip {
            memory: [0; 4096],
            stack: [0; 32],
            stack_level: 0,
            display: Display::new(),
            registers: [0; 16],
            delay_timer: Timer::new(TIMER_HZ),
            sound_timer: Timer::new(TIMER_HZ),
            i: 0,
            pc: 0,
            chip_type: SUPERCHIP.to_string()
        };

        let font = font::get_font();
        for (offset, byte) in font.iter().enumerate() {
            chip.memory[FONT_ADDR + offset] = *byte;
        }

        chip
    }

    fn clear_display(&mut self) {
        self.display.clear();
    }

    fn handle_return(&mut self) {
        if self.stack_level == 0 {
            println!("Can't return from empty stack");
            process::exit(1);
        }
        self.stack_level = self.stack_level - 1; // stack_level is set to next empty slot in stack,
                                                 // so go back one level to get the last used slot
        self.pc = self.stack[self.stack_level];
    }

    fn jump(&mut self, address: usize) {
        self.pc = address;
    }

    fn call_at(&mut self, address: usize) {
        self.stack[self.stack_level] = self.pc;
        self.stack_level = self.stack_level + 1;
        self.pc = address;
    }

    fn set_vx_rand(&mut self, x: u8, seed: u8) {
        let rand_number = rand::thread_rng().gen_range(0..255);
        self.registers[x as usize] = rand_number & seed;
    }

    //Draw sprite at coord (x, y) that is 8 pixels wide and the height arg tall
    fn draw(&mut self, x: u8, y:u8, height: u8) {
        let x_index = self.registers[x as usize] as usize;
        let y_index = self.registers[y as usize] as usize;

        self.registers[0xF] = 0; // Clear pixel_flip flag

        // Collect sprite to draw on screen
        let mut sprite: Vec<u8> = vec![];
        for row in 0..height {
            // Each row is a byte in memory, so to get the next row, go to the next memory addr
            sprite.push(self.memory[self.i + row as usize]);
        }

        // Let display actually draw the sprite
        let did_flip_pixel_to_off = self.display.draw_sprite(x_index, y_index, height, sprite);
        self.registers[0xF] = if did_flip_pixel_to_off { 1 } else { 0 };
    }

    fn skip_if_key_press(&mut self, x: u8) {
        let device_state = DeviceState::new();
        let keys: Vec<Keycode> = device_state.get_keys();
        let vx_keycode = self.hex_to_keycode(self.registers[x as usize]);

        if keys.contains(&vx_keycode) {
            self.pc = self.pc + 2;
        }
    }

    fn skip_if_not_key_press(&mut self, x: u8) {
        let device_state = DeviceState::new();
        let keys: Vec<Keycode> = device_state.get_keys();
        let vx_keycode = self.hex_to_keycode(self.registers[x as usize]);

        if !keys.contains(&vx_keycode) {
            self.pc = self.pc + 2;
        }
    }

    fn hex_to_keycode(&self, keypress: u8) -> Keycode {
        match keypress {
            0x1 => Keycode::Key1,
            0x2 => Keycode::Key2,
            0x3 => Keycode::Key3,
            0x4 => Keycode::Q,
            0x5 => Keycode::W,
            0x6 => Keycode::E,
            0x7 => Keycode::A,
            0x8 => Keycode::S,
            0x9 => Keycode::D,
            0xA => Keycode::Z,
            0x0 => Keycode::X,
            0xB => Keycode::C,
            0xC => Keycode::Key4,
            0xD => Keycode::R,
            0xE => Keycode::F,
            0xF => Keycode::V,
            _ => {
                Keycode::D
            }
        }
    }

    fn keycode_to_hex(&self, keypress: &Keycode) -> Option<u8> {
        match keypress {
            Keycode::Key1 => Some(0x1),
            Keycode::Key2 => Some(0x2),
            Keycode::Key3 => Some(0x3),
            Keycode::Q => Some(0x4),
            Keycode::W => Some(0x5),
            Keycode::E => Some(0x6),
            Keycode::A => Some(0x7),
            Keycode::S => Some(0x8),
            Keycode::D => Some(0x9),
            Keycode::Z => Some(0xA),
            Keycode::X => Some(0x0),
            Keycode::C => Some(0xB),
            Keycode::Key4 => Some(0xC),
            Keycode::R => Some(0xD),
            Keycode::F => Some(0xE),
            Keycode::V => Some(0xF),
            _ => {
                None
            }
        }
    }

    fn await_then_store_keypress(&mut self, x: u8) {
        let device_state = DeviceState::new();
        loop {
            let keys = device_state.get_keys();
            let mut keys_in_hex = keys.iter().filter_map(|k| self.keycode_to_hex(k));
            if let Some(key) = keys_in_hex.next() {
                self.registers[x as usize] = key;
                break;
            }
        }
    }

    fn fetch(&mut self) -> u16 {
        let first_byte = self.memory[self.pc] as u16;
        let second_byte = self.memory[1 + self.pc] as u16;

        let shifted_first_byte = first_byte << 8; // 0xAB becomes 0xAB00
        let combined_bytes = shifted_first_byte + second_byte;

        self.pc = self.pc + 2;

        combined_bytes
    }

    fn decode(&self, instruction: u16) -> DecodedInstruction {
        let first_nibble = (instruction >> 12) as u8;
        let second_nibble = ((instruction & 0x0F00) >> 8) as u8;
        let third_nibble = ((instruction & 0x00F0) >> 4) as u8;
        let fourth_nibble = (instruction & 0x000F) as u8;
        let nibbles = [first_nibble, second_nibble, third_nibble, fourth_nibble];

        let nn = (instruction & 0x00FF) as u8;
        let nnn = (instruction & 0x0FFF) as usize;

        DecodedInstruction { nibbles, nn, nnn }
    }

    fn execute(&mut self, decoded_instruction: DecodedInstruction) {
        match decoded_instruction.nibbles {
            [0, 0, 0x0, 0x0] => process::exit(1),
            [0, 0, 0xE, 0x0] => self.clear_display(),
            [0, 0, 0xE, 0xE] => self.handle_return(),
            [0, _, _, _] => { },
            [1, _, _, _] => self.jump(decoded_instruction.nnn),
            [2, _, _, _] => self.call_at(decoded_instruction.nnn as usize),
            [3, x, _, _] => {
                if self.registers[x as usize] == decoded_instruction.nn {
                    self.pc += 2
                }
            },
            [4, x, _, _] => {
                if self.registers[x as usize] != decoded_instruction.nn {
                    self.pc += 2
                }
            },
            [5, x, y, 0x0] => {
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.pc += 2
                }
            },
            [6, x, _, _] => self.registers[x as usize] = decoded_instruction.nn,
            [7, x, _, _] => self.registers[x as usize] = {
                let sum = self.registers[x as usize] as u16 + decoded_instruction.nn as u16;
                if sum > 255 {
                    (sum - 255 - 1) as u8
                } else {
                    sum as u8
                }
            },
            [8, x, y, 0] => self.registers[x as usize] = self.registers[y as usize],
            [8, x, y, 1] => {
                self.registers[x as usize] = self.registers[x as usize] | self.registers[y as usize];

                if self.chip_type == CHIP8 {
                    self.registers[0xF] = 0;
                }
            },
            [8, x, y, 2] => {
                self.registers[x as usize] = self.registers[x as usize] & self.registers[y as usize];

                if self.chip_type == CHIP8 {
                    self.registers[0xF] = 0;
                }
            },
            [8, x, y, 3] => {
                self.registers[x as usize] = self.registers[x as usize] ^ self.registers[y as usize];

                if self.chip_type == CHIP8 {
                    self.registers[0xF] = 0;
                }
            },
            [8, x, y, 4] => self.registers[x as usize] = {
                let mut sum = self.registers[x as usize] as u16 + self.registers[y as usize] as u16;
                if sum > 255 {
                    sum = sum - 255 - 1;
                    self.registers[0xF] = 1;
                    sum as u8
                } else {
                    self.registers[0xF] = 0;
                    sum as u8
                }
            },
            [8, x, y, 5] => self.registers[x as usize] = {
                if self.registers[x as usize] >= self.registers[y as usize] {
                    let diff = self.registers[x as usize] - self.registers[y as usize];
                    self.registers[0xF] = 1;

                    diff
                } else {
                    let positive_diff = self.registers[y as usize] - self.registers[x as usize];
                    let diff = 0xFF - positive_diff + 1;
                    self.registers[0xF] = 0;

                    diff
                }
            },
            [8, x, y, 6] => {
                if self.chip_type == CHIP8 {
                    self.registers[x as usize] = self.registers[y as usize];
                }
                let vx = self.registers[x as usize];
                let lsb = vx & 1;

                self.registers[x as usize] = vx >> 1;
                self.registers[0xF] = lsb;
            },
            [8, x, y, 7] => self.registers[x as usize] = {
                if self.registers[y as usize] >= self.registers[x as usize] {
                    let diff = self.registers[y as usize] - self.registers[x as usize];
                    self.registers[0xF] = 1;
                    diff
                } else {
                    let positive_diff = self.registers[x as usize] - self.registers[y as usize];
                    self.registers[0xF] = 0;
                    0xFF - positive_diff + 1
                }
            },
            [8, x, y, 0xE] => {
                if self.chip_type == CHIP8 {
                    self.registers[x as usize] = self.registers[y as usize];
                }
                let vx = self.registers[x as usize];
                let msb = (vx >> 7) & 1;

                self.registers[x as usize] = vx << 1;
                self.registers[0xF] = msb;
            },
            [9, x, y, 0] => {
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.pc += 2
                }
            },
            [0xA, _, _, _] => self.i = decoded_instruction.nnn,
            [0xB, x, _, _] => {
                let register_index = if self.chip_type == SUPERCHIP { x as usize } else { 0 };
                self.pc = self.registers[register_index] as usize + decoded_instruction.nnn
            },
            [0xC, x, _, _] => self.set_vx_rand(x, decoded_instruction.nn),
            [0xD, x, y, n] => self.draw(x, y, n),
            [0xE, x, 0x9, 0xE] => self.skip_if_key_press(x),
            [0xE, x, 0xA, 0x1] => self.skip_if_not_key_press(x),
            [0xF, x, 0x0, 0x7] => self.registers[x as usize] = self.delay_timer.get(),
            [0xF, x, 0x0, 0xA] => self.await_then_store_keypress(x),
            [0xF, x, 0x1, 0x5] => self.delay_timer.set(self.registers[x as usize]),
            [0xF, x, 0x1, 0x8] => self.sound_timer.set(self.registers[x as usize]),
            [0xF, x, 0x1, 0xE] => self.i += self.registers[x as usize] as usize,
            [0xF, x, 0x2, 0x9] => self.i = FONT_ADDR + (self.registers[x as usize] as usize * font::FONT_SIZE),
            [0xF, x, 0x3, 0x3] => {
                let number = self.registers[x as usize];
                let ones = number % 10;
                let tens = number / 10 % 10;
                let hundreds = number / 10 / 10 % 10;

                self.memory[self.i] = hundreds;
                self.memory[1 + self.i] = tens;
                self.memory[2 + self.i] = ones;
            },
            [0xF, x, 0x5, 0x5] => {
                for register in 0..=x as usize {
                    let address = self.i + register;
                    self.memory[address] = self.registers[register];
                }

                // CHIP-8 updates I to the end of the stored registers
                if self.chip_type == "CHIP-8" {
                    self.i = self.i + x as usize;
                }
            },
            [0xF, x, 0x6, 0x5] => {
                for register in 0..=x as usize {
                    let address = self.i + register;
                    self.registers[register] = self.memory[address]
                }

                if self.chip_type == "CHIP-8" {
                    self.i = self.i + x as usize;
                }
            },
            _ => ()
        }
    }

    fn check_timers(&mut self) {
        self.delay_timer.check_decrement();
        self.sound_timer.check_decrement();
    }

    pub fn step(&mut self) {
        let instruction = self.fetch();
        let decoded_instruction = self.decode(instruction);
        self.execute(decoded_instruction);
        self.check_timers();
    }

    pub fn load_rom(&mut self, rom: &Vec<u8>) {
        for (offset, byte) in rom.iter().enumerate() {
            self.memory[ROM_ADDR + offset] = *byte;
            self.pc = ROM_ADDR;
        }
    }
}

#[cfg(test)]
use crate::display::DISPLAY_SIZE;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_memory_has_font_at_0x050() {
        let chip = Chip::new();
        let font = font::get_font();

        for (i, byte) in font.iter().enumerate() {
            assert_eq!(*byte, chip.memory[0x050 + i]);
        }
    }

    #[test]
    fn initial_stack_is_32_zeroed_out_double_bytes() {
        let chip = Chip::new();
        assert_eq!(chip.stack.len(), 32);
        for stack_frame in chip.stack.iter() {
            assert_eq!(*stack_frame, 0x00000000)
        }
    }

    #[test]
    fn timers_can_be_set_to_value() {
        let mut delay_timer = Chip::new().delay_timer;
        assert_eq!(delay_timer.get(), 0);

        delay_timer.set(255);
        assert_eq!(delay_timer.get(), 255);
    }

    #[test]
    fn fetch_gets_two_byte_instruction_and_increments_pc() {
        let mut chip = Chip::new();
        chip.pc = 30;

        chip.memory[30] = 0xAB;
        chip.memory[31] = 0xCD;

        let instruction = chip.fetch();

        assert_eq!(instruction, 0xABCD);
        assert_eq!(chip.pc, 32);
    }

    #[test]
    fn decode_parses_instruction() {
        let chip = Chip::new();
        let instruction = 0xABCD;

        let decoded_instruction = chip.decode(instruction);

        assert_eq!(decoded_instruction.nibbles[0], 0xA);
        assert_eq!(decoded_instruction.nibbles[1], 0xB);
        assert_eq!(decoded_instruction.nibbles[2], 0xC);
        assert_eq!(decoded_instruction.nibbles[3], 0xD);
        assert_eq!(decoded_instruction.nn, 0xCD);
        assert_eq!(decoded_instruction.nnn, 0xBCD);
    }

    #[test]
    fn calling_and_returning_from_functions_works() {
        let mut chip = Chip::new();
        chip.pc = 0xFFF;

        chip.call_at(0x200);
        assert_eq!(chip.stack[0], 0xFFF);
        assert_eq!(chip.pc, 0x200);

        chip.call_at(0x300);
        assert_eq!(chip.stack[1], 0x200);
        assert_eq!(chip.pc, 0x300);

        chip.call_at(0x400);
        assert_eq!(chip.stack[2], 0x300);
        assert_eq!(chip.pc, 0x400);
        assert_eq!(chip.stack_level, 3);

        // Now Return
        chip.handle_return();
        assert_eq!(chip.pc, 0x300);
        assert_eq!(chip.stack[2], 0x300); // We dont clear the stack, just overwrite it

        chip.handle_return();
        assert_eq!(chip.pc, 0x200);

        chip.handle_return();
        assert_eq!(chip.pc, 0xFFF);
        assert_eq!(chip.stack_level, 0);
    }

    #[test]
    fn load_rom() {
        let mut chip = Chip::new();
        let rom: Vec<u8> = vec![0xD, 0xE, 0xA, 0xD, 0xB, 0xE, 0xE, 0xF];

        chip.load_rom(&rom);

        assert_eq!(chip.memory[ROM_ADDR + 0], 0xD);
        assert_eq!(chip.memory[ROM_ADDR + 1], 0xE);
        assert_eq!(chip.memory[ROM_ADDR + 2], 0xA);
        assert_eq!(chip.memory[ROM_ADDR + 3], 0xD);
        assert_eq!(chip.memory[ROM_ADDR + 4], 0xB);
        assert_eq!(chip.memory[ROM_ADDR + 5], 0xE);
        assert_eq!(chip.memory[ROM_ADDR + 6], 0xE);
        assert_eq!(chip.memory[ROM_ADDR + 7], 0xF);
    }

    #[test]
    fn test_00e0_clear_display() {
        let mut chip = Chip::new();
        chip.display.display = [true; DISPLAY_SIZE];

        let decoded_instruction = chip.decode(0x00E0);
        chip.execute(decoded_instruction);

        assert_eq!(chip.display.display, [false; DISPLAY_SIZE]);
    }

    #[test]
    fn test_00ee_return_from_subroutine() {
        let mut chip = Chip::new();
        chip.pc = 0x500;
        chip.stack[0] = 0x250;
        chip.stack_level = 1;

        let decoded_instruction = chip.decode(0x00EE);
        chip.execute(decoded_instruction);

        assert_eq!(chip.pc, 0x250);
        assert_eq!(chip.stack_level, 0);
    }

    #[test]
    fn test_1nnn_jump() {
        let mut chip = Chip::new();
        chip.pc = 0x250;

        let decoded_instruction = chip.decode(0x1ABC);
        chip.execute(decoded_instruction);

        assert_eq!(chip.pc, 0xABC);
    }

    #[test]
    fn test_2nnn_call_subroutine_at_nnn() {
        let mut chip = Chip::new();
        chip.pc = 0x250;

        let decoded_instruction = chip.decode(0x2ABC);
        chip.execute(decoded_instruction);

        assert_eq!(chip.pc, 0xABC);
        assert_eq!(chip.stack[0], 0x250);
    }

    #[test]
    fn test_3xnn_skip_if_vx_equal_nn_dont_skip() {
        let mut chip = Chip::new();
        chip.pc = 0x250;
        chip.registers[0xA] = 0xAA;

        let decoded_instruction = chip.decode(0x3A00);
        chip.execute(decoded_instruction);

        assert_eq!(chip.pc, 0x250);
    }

    #[test]
    fn test_3xnn_skip_if_vx_equal_nn_skip() {
        let mut chip = Chip::new();
        chip.pc = 0x250;
        chip.registers[0xA] = 0xAA;

        let decoded_instruction = chip.decode(0x3AAA);
        chip.execute(decoded_instruction);

        assert_eq!(chip.pc, 0x252);
    }

    #[test]
    fn test_4xnn_skip_if_vx_not_equal_nn_dont_skip() {
        let mut chip = Chip::new();
        chip.pc = 0x250;
        chip.registers[0xA] = 0xAA;

        let decoded_instruction = chip.decode(0x4AAA);
        chip.execute(decoded_instruction);

        assert_eq!(chip.pc, 0x250);
    }

    #[test]
    fn test_4xnn_skip_if_vx_not_equal_nn_skip() {
        let mut chip = Chip::new();
        chip.pc = 0x250;
        chip.registers[0xA] = 0xAA;

        let decoded_instruction = chip.decode(0x4A00);
        chip.execute(decoded_instruction);

        assert_eq!(chip.pc, 0x252);
    }

    #[test]
    fn test_5xy0_skip_if_vx_equal_vy_dont_skip() {
        let mut chip = Chip::new();
        chip.pc = 0x250;
        chip.registers[0xA] = 0xAA;
        chip.registers[0xB] = 0xBB;

        let decoded_instruction = chip.decode(0x5AB0);
        chip.execute(decoded_instruction);

        assert_eq!(chip.pc, 0x250);
    }

    #[test]
    fn test_5xy0_skip_if_vx_equal_vy_skip() {
        let mut chip = Chip::new();
        chip.pc = 0x250;
        chip.registers[0xA] = 0xAA;
        chip.registers[0xB] = 0xAA;

        let decoded_instruction = chip.decode(0x5AB0);
        chip.execute(decoded_instruction);

        assert_eq!(chip.pc, 0x252);
    }

    #[test]
    fn test_6xnn_set_vx_to_nn_00() {
        let mut chip = Chip::new();
        chip.registers[0xA] = 0x0;

        let decoded_instruction = chip.decode(0x6A00);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[0xA], 0x00);
    }

    #[test]
    fn test_6xnn_set_vx_to_nn_ff() {
        let mut chip = Chip::new();
        chip.registers[0xA] = 0x0;

        let decoded_instruction = chip.decode(0x6AFF);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[0xA], 0xFF);
    }

    #[test]
    fn test_7xnn_add_vx_and_nn() {
        let vx: usize = 0x3;
        let vf: usize = 0xF;
        let mut chip = Chip::new();
        chip.registers[vx] = 0x10; // vx = 0x10
        chip.registers[vf] = 0x01;

        let decoded_instruction = chip.decode(0x730F); // n = 0x0F
        chip.execute(decoded_instruction);

        // 0x10 + 0x0F = 0x1F
        assert_eq!(chip.registers[vx], 0x1F);
        assert_eq!(chip.registers[vf], 0x1); // Carry flag is unchanged
    }

    #[test]
    fn test_7xnn_add_vx_and_nn_overflow() {
        let vx: usize = 0x3;
        let vf: usize = 0xF;
        let mut chip = Chip::new();
        chip.registers[vx] = 0xF0; // vx = 0xF0
        chip.registers[vf] = 0x00;

        let decoded_instruction = chip.decode(0x73F0); // nn = 0xF0
        chip.execute(decoded_instruction);

        // 0xF0 + 0xF0 = 0x1E0 --> only 8 bits, so its just 0xE0
        assert_eq!(chip.registers[vx], 0xE0);
        assert_eq!(chip.registers[vf], 0x0); // Carry flag is unchanged
    }

    #[test]
    fn test_8xy0_set_vx_to_value_of_vy() {
        let vx: usize = 0xA;
        let vy: usize = 0xB;
        let mut chip = Chip::new();
        chip.registers[vx] = 0xF0;
        chip.registers[vy] = 0x0F;

        let decoded_instruction = chip.decode(0x8AB0);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx], 0x0F);
        assert_eq!(chip.registers[vy], 0x0F); // VY is unchanged
    }

    #[test]
    fn test_8xy1_set_vx_to_vx_bitwise_or_vy_none() {
        let vx: usize = 0xA;
        let vy: usize = 0xB;
        let mut chip = Chip::new();
        chip.registers[vx] = 0x00;
        chip.registers[vy] = 0x00;

        let decoded_instruction = chip.decode(0x8AB1);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx], 0x00);
        assert_eq!(chip.registers[vy], 0x00); // VY is unchanged
    }

    #[test]
    fn test_8xy1_set_vx_to_vx_bitwise_or_vy_all() {
        let vx: usize = 0xA;
        let vy: usize = 0xB;
        let mut chip = Chip::new();
        chip.registers[vx] = 0xF0;
        chip.registers[vy] = 0x0F;

        let decoded_instruction = chip.decode(0x8AB1);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx], 0xFF);
        assert_eq!(chip.registers[vy], 0x0F); // VY is unchanged
    }

    #[test]
    fn test_8xy2_set_vx_to_vx_bitwise_and_vy_none() {
        let vx: usize = 0xA;
        let vy: usize = 0xB;
        let mut chip = Chip::new();
        chip.registers[vx] = 0xF0;
        chip.registers[vy] = 0x0F;

        let decoded_instruction = chip.decode(0x8AB2);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx], 0x00);
        assert_eq!(chip.registers[vy], 0x0F); // VY is unchanged
    }

    #[test]
    fn test_8xy2_set_vx_to_vx_bitwise_and_vy_some() {
        let vx: usize = 0xA;
        let vy: usize = 0xB;
        let mut chip = Chip::new();
        chip.registers[vx] = 0xF2;
        chip.registers[vy] = 0x18;

        let decoded_instruction = chip.decode(0x8AB2);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx], 0x10);
        assert_eq!(chip.registers[vy], 0x18); // VY is unchanged
    }

    #[test]
    fn test_8xy2_set_vx_to_vx_bitwise_and_vy_all() {
        let vx: usize = 0xA;
        let vy: usize = 0xB;
        let mut chip = Chip::new();
        chip.registers[vx] = 0xFF;
        chip.registers[vy] = 0xFF;

        let decoded_instruction = chip.decode(0x8AB2);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx], 0xFF);
        assert_eq!(chip.registers[vy], 0xFF); // VY is unchanged
    }

    #[test]
    fn test_8xy3_set_vx_to_vx_bitwise_xor_vy_none() {
        let vx: usize = 0xA;
        let vy: usize = 0xB;
        let mut chip = Chip::new();
        chip.registers[vx] = 0xFF;
        chip.registers[vy] = 0xFF;

        let decoded_instruction = chip.decode(0x8AB3);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx], 0x00);
        assert_eq!(chip.registers[vy], 0xFF); // VY is unchanged
    }

    #[test]
    fn test_8xy3_set_vx_to_vx_bitwise_xor_vy_some() {
        let vx: usize = 0xA;
        let vy: usize = 0xB;
        let mut chip = Chip::new();
        chip.registers[vx] = 0xF2;
        chip.registers[vy] = 0x18;

        let decoded_instruction = chip.decode(0x8AB3);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx], 0xEA);
        assert_eq!(chip.registers[vy], 0x18); // VY is unchanged
    }

    #[test]
    fn test_8xy3_set_vx_to_vx_bitwise_xor_vy_all() {
        let vx: usize = 0xA;
        let vy: usize = 0xB;
        let mut chip = Chip::new();
        chip.registers[vx] = 0xF0;
        chip.registers[vy] = 0x0F;

        let decoded_instruction = chip.decode(0x8AB3);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx], 0xFF);
        assert_eq!(chip.registers[vy], 0x0F); // VY is unchanged
    }

    #[test]
    fn test_8xy4_add_vx_and_vy() {
        let vx: usize = 0xA;
        let vy: usize = 0xB;
        let vf: usize = 0xF;
        let mut chip = Chip::new();
        chip.registers[vx] = 0xF0;
        chip.registers[vy] = 0x0F;

        let decoded_instruction = chip.decode(0x8AB4);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx], 0xFF);
        assert_eq!(chip.registers[vy], 0x0F); // VY is unchanged
        assert_eq!(chip.registers[vf], 0x0); // Carry flag is not set
    }

    #[test]
    fn test_8xy4_add_vx_and_vy_overflow() {
        let vx: usize = 0xA;
        let vy: usize = 0xB;
        let vf: usize = 0xF;
        let mut chip = Chip::new();
        chip.registers[vx] = 0xFF;
        chip.registers[vy] = 0x0F;

        let decoded_instruction = chip.decode(0x8AB4);
        chip.execute(decoded_instruction);

        // 0xFF + 0x0F = 0x10E --> u8 only has 8 bits, so it's 0xOE
        assert_eq!(chip.registers[vx], 0x0E);
        assert_eq!(chip.registers[vy], 0x0F); // VY is unchanged
        assert_eq!(chip.registers[vf], 0x1); // Carry flag is set
    }

    #[test]
    fn test_8xy5_subtract_vy_from_vx() {
        let vx = 0xA;
        let vy = 0xB;
        let vf = 0xF;
        let mut chip = Chip::new();
        chip.registers[vx as usize] = 0xFF;
        chip.registers[vy as usize] = 0x0F;

        let decoded_instruction = chip.decode(0x8AB5);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx as usize], 0xF0);
        assert_eq!(chip.registers[vy as usize], 0x0F); // VY is unchanged
        assert_eq!(chip.registers[vf as usize], 0x1); // Carry flag is set since no borrow
    }

    #[test]
    fn test_8xy5_subtract_vy_from_vx_underflow() {
        let vx = 0xA;
        let vy = 0xB;
        let vf = 0xF;
        let mut chip = Chip::new();
        chip.registers[vx as usize] = 0x0F;
        chip.registers[vy as usize] = 0xFF;

        let decoded_instruction = chip.decode(0x8AB5);
        chip.execute(decoded_instruction);

        // 0x10F - 0xFF = 0x010 ---> 0x0F - 0xFF is the same, but have to carry.
        assert_eq!(chip.registers[vx as usize], 0x10);
        assert_eq!(chip.registers[vy as usize], 0xFF); // VY is unchanged
        assert_eq!(chip.registers[vf as usize], 0x0); // Carry flag no longer set due to the borrow
    }

    #[test]
    fn test_8xy6_store_vx_least_sig_bit_into_vf_1() {
        let vx: usize = 0xA;
        let vf: usize = 0xF;
        let mut chip = Chip::new();
        chip.registers[vx] = 0b11111101;
        chip.registers[vf] = 0x00;

        let decoded_instruction = chip.decode(0x8AB6);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx], 0b01111110);
        assert_eq!(chip.registers[vf], 1);
    }

    #[test]
    fn test_8xy6_store_vx_least_sig_bit_into_vf_0() {
        let vx: usize = 0xA;
        let vf: usize = 0xF;
        let mut chip = Chip::new();
        chip.registers[vx] = 0b10000010;
        chip.registers[vf] = 0x00;

        let decoded_instruction = chip.decode(0x8AB6);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx], 0b01000001);
        assert_eq!(chip.registers[vf], 0);
    }

    #[test]
    fn test_8xy7_set_vx_to_vy_minux_vx() {
        let vx = 0xA;
        let vy = 0xB;
        let vf = 0xF;
        let mut chip = Chip::new();
        chip.registers[vx as usize] = 0x0F;
        chip.registers[vy as usize] = 0xFF;

        let decoded_instruction = chip.decode(0x8AB7);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx as usize], 0xF0);
        assert_eq!(chip.registers[vy as usize], 0xFF); // VY is unchanged
        assert_eq!(chip.registers[vf as usize], 0x1); // Carry flag set due to no borrow
    }

    #[test]
    fn test_8xy7_set_vx_to_vy_minux_vx_underflow() {
        let vx = 0xA;
        let vy = 0xB;
        let vf = 0xF;
        let mut chip = Chip::new();
        chip.registers[vx as usize] = 0xFF;
        chip.registers[vy as usize] = 0x0F;

        let decoded_instruction = chip.decode(0x8AB7);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx as usize], 0x10);
        assert_eq!(chip.registers[vy as usize], 0x0F); // VY is unchanged
        assert_eq!(chip.registers[vf as usize], 0x0); // Carry flag not set due to the borrow
    }

    #[test]
    fn test_8xye_store_vx_most_sig_bit_into_vf_1() {
        let vx: usize = 0xA;
        let vf: usize = 0xF;
        let mut chip = Chip::new();
        chip.registers[vx] = 0b10000001;
        chip.registers[vf] = 0x00;

        let decoded_instruction = chip.decode(0x8ABE);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx], 0b00000010);
        assert_eq!(chip.registers[vf], 0x1); // Carry flag is unchanged
                                             //TODO - This is the super8 impl. Need to also do
                                             //chip-8 impl only
    }

    #[test]
    fn test_8xye_store_vx_most_sig_bit_into_vf_0() {
        let vx: usize = 0xA;
        let vf: usize = 0xF;
        let mut chip = Chip::new();
        chip.registers[vx] = 0b01111111;
        chip.registers[vf] = 0x00;

        let decoded_instruction = chip.decode(0x8ABE);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx], 0b11111110);
        assert_eq!(chip.registers[vf], 0x0);
    }

    #[test]
    fn test_9xy0_skip_if_vx_not_equal_vy_skip() {
        let mut chip = Chip::new();
        chip.pc = 0x250;
        chip.registers[0xA] = 0xAA;
        chip.registers[0xB] = 0x00;

        let decoded_instruction = chip.decode(0x9AB0);
        chip.execute(decoded_instruction);

        assert_eq!(chip.pc, 0x252);
    }

    #[test]
    fn test_annn_set_i_to_nnn() {
        let mut chip = Chip::new();
        chip.i = 0;

        let decoded_instruction = chip.decode(0xABED);
        chip.execute(decoded_instruction);

        assert_eq!(chip.i, 0xBED);
    }

    #[test]
    fn test_bnnn_jump_to_nnn_plus_v0_chip8() {
        let mut chip = Chip::new();
        chip.chip_type = CHIP8.to_string();
        chip.pc = 0x200;
        chip.registers[0] = 0xF;

        let decoded_instruction = chip.decode(0xBABC);
        chip.execute(decoded_instruction);

        assert_eq!(chip.pc, 0xABC + 0xF);
    }

    #[test]
    fn test_bnnn_jump_to_nnn_plus_v0_superchip() {
        let mut chip = Chip::new();
        chip.chip_type = SUPERCHIP.to_string();
        chip.pc = 0x200;
        chip.registers[0x0] = 0x0;
        chip.registers[0xA] = 0xF;
        //SUPERCHIP uses VX instead of V0 to add to NNN

        let decoded_instruction = chip.decode(0xBABC);
        chip.execute(decoded_instruction);

        assert_eq!(chip.pc, 0xABC + 0xF);
    }

    #[test]
    fn test_ex9e_skip_if_vx_key_is_pressed() {
        let mut chip = Chip::new();
        chip.pc = 0x200;

        let decoded_instruction = chip.decode(0xEA9E);
        chip.execute(decoded_instruction);

        assert_eq!(chip.pc, 0x200);
    }

    #[test]
    fn test_fx07_set_vx_to_delay_timers_value() {
        let mut chip = Chip::new();
        let vx = 0xA;
        chip.registers[vx] = 0;
        chip.delay_timer.set(30);

        let decoded_instruction = chip.decode(0xFA07);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[vx], 30);
    }

    /* Commenting out since you have to type, but it did allow me to test manually
     * There has to a better way to test this, or even yet a better way to get the input
    #[test]
    fn test_fx0a_await_then_store_keypress_in_vx() {
        let mut chip = Chip::new();
        let vx = 0xA;

        let decoded_instruction = chip.decode(0xFA0A);
        chip.execute(decoded_instruction); // Have to press E and enter

        assert_eq!(chip.registers[vx], 0xE)
    }
    *
    */

    #[test]
    fn test_fx15_set_delay_timer_to_vx() {
        let mut chip = Chip::new();
        let vx = 0xA;
        chip.registers[vx] = 30;
        chip.delay_timer.set(0);

        let decoded_instruction = chip.decode(0xFA15);
        chip.execute(decoded_instruction);

        assert_eq!(chip.delay_timer.get(), 30);
    }

    #[test]
    fn test_fx18_set_sound_timer_to_vx() {
        let mut chip = Chip::new();
        let vx = 0xA;
        chip.registers[vx] = 30;
        chip.sound_timer.set(0);

        let decoded_instruction = chip.decode(0xFA18);
        chip.execute(decoded_instruction);

        assert_eq!(chip.sound_timer.get(), 30);
    }

    #[test]
    fn test_fx1e_add_vx_to_i() {
        let mut chip = Chip::new();
        chip.i = 0x0F;
        chip.registers[0xA] = 0xF0;
        chip.registers[0xF] = 9;

        let decoded_instruction = chip.decode(0xFA1E);
        chip.execute(decoded_instruction);

        assert_eq!(chip.i, 0xFF);
        assert_eq!(chip.registers[0xF], 0x9); // VF unaffected
    }

    #[test]
    fn test_fx1e_add_vx_to_i_overflow() {
        let mut chip = Chip::new();
        chip.i = 4096;
        chip.registers[0xA] = 255;
        chip.registers[0xF] = 9;

        let decoded_instruction = chip.decode(0xFA1E);
        chip.execute(decoded_instruction);

        // Not an overflow, and we don't throw any error. This is the rom's responsibility to ensure
        assert_eq!(chip.i, 4351);
        assert_eq!(chip.registers[0xF], 0x9); // VF unaffected
    }

    #[test]
    fn test_fx29_set_i_to_sprite_for_vx() {
        let mut chip = Chip::new();
        chip.registers[0xA] = 0xF;

        let decoded_instruction = chip.decode(0xFA29);
        chip.execute(decoded_instruction);

        assert_eq!(FONT_ADDR, 0x050);
        assert_eq!(chip.i, 0x09B); // FONT_ADDR + skipping 15 5 byte characters(0x4B) to get to 0x9B
        assert_eq!(chip.memory[chip.i], 0xF0);
        assert_eq!(chip.memory[chip.i + 1], 0x80);
        assert_eq!(chip.memory[chip.i + 2], 0xF0);
        assert_eq!(chip.memory[chip.i + 3], 0x80);
        assert_eq!(chip.memory[chip.i + 4], 0x80);
    }

    #[test]
    fn test_fx33_store_binary_at_i_000() {
        let vx = 0xA;
        let mut chip = Chip::new();
        chip.registers[vx] = 0;
        chip.i = 0;

        let decoded_instruction = chip.decode(0xFA33);
        chip.execute(decoded_instruction);

        assert_eq!(chip.memory[0], 0x0);
        assert_eq!(chip.memory[1], 0x0);
        assert_eq!(chip.memory[2], 0x0);
    }

    #[test]
    fn test_fx33_store_binary_at_i_255() {
        let vx = 0xA;
        let mut chip = Chip::new();
        chip.registers[vx] = 255;
        chip.i = 0;

        let decoded_instruction = chip.decode(0xFA33);
        chip.execute(decoded_instruction);

        assert_eq!(chip.memory[0], 0x2);
        assert_eq!(chip.memory[1], 0x5);
        assert_eq!(chip.memory[2], 0x5);
    }

    #[test]
    fn test_fx33_store_binary_at_i_123() {
        let vx = 0xA;
        let mut chip = Chip::new();
        chip.registers[vx] = 123;
        chip.i = 0;

        let decoded_instruction = chip.decode(0xFA33);
        chip.execute(decoded_instruction);

        assert_eq!(chip.memory[0], 0x1);
        assert_eq!(chip.memory[1], 0x2);
        assert_eq!(chip.memory[2], 0x3);
    }

    #[test]
    fn test_fx55_store_registers_at_i() {
        let mut chip = Chip::new();
        chip.i = 0x500;
        chip.registers[0x0] = 123;
        chip.registers[0x5] = 23;
        chip.registers[0xA] = 3;
        chip.registers[0xB] = 32;

        let decoded_instruction = chip.decode(0xFA55);
        chip.execute(decoded_instruction);

        assert_eq!(chip.memory[0x500], 123);
        assert_eq!(chip.memory[0x505], 23);
        assert_eq!(chip.memory[0x50A], 3);
        assert_eq!(chip.memory[0x50B], 0); // Not included, since VX is 0xA
    }

    #[test]
    fn test_fx65_load_registers_from_i() {
        let mut chip = Chip::new();
        let i = 0x500;
        chip.i = i;
        chip.memory[i + 0] = 123;
        chip.memory[i + 1] = 23;
        chip.memory[i + 2] = 3;
        chip.memory[i + 3] = 32;
        chip.memory[i + 4] = 33;

        let decoded_instruction = chip.decode(0xF365);
        chip.execute(decoded_instruction);

        assert_eq!(chip.registers[0], 123);
        assert_eq!(chip.registers[1], 23);
        assert_eq!(chip.registers[2], 3);
        assert_eq!(chip.registers[3], 32); // Not included, since VX is 0x3
        assert_eq!(chip.registers[4], 0); // Not included, since VX is 0x3
    }
}
