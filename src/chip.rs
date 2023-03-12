use rand::Rng;
use crate::font;
use std::process;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
const FONT_ADDR: usize = 0x050;
const ROM_ADDR: usize = 0x200;

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

#[derive(Debug)]
pub struct DecodedInstruction {
    nibbles: [u8; 4],
    nn: u8,
    nnn: u16
}

#[derive(Debug)]
pub struct Chip {
    pub memory: [u8; 4096],
    pub registers: [u8; 16],
    pub stack: [usize; 32],
    pub stack_level: usize,
    pub display: [bool; DISPLAY_SIZE],
    pub delay_timer: Timer,
    pub sound_timer: Timer,
    pub i: usize,
    pub pc: usize
}

impl Chip {
    pub fn new() -> Self {
        let mut chip = Chip {
            memory: [0; 4096],
            stack: [0; 32],
            stack_level: 0,
            display: [false; DISPLAY_SIZE],
            registers: [0; 16],
            delay_timer: Timer::new(),
            sound_timer: Timer::new(),
            i: 0,
            pc: 0
        };

        let font = font::get_font();
        for (offset, byte) in font.iter().enumerate() {
            chip.memory[FONT_ADDR + offset] = *byte;
        }

        chip
    }

    fn clear_display(&mut self) {
        self.display = [false; DISPLAY_SIZE];
    }

    fn handle_return(&mut self) {
        self.pc = self.stack[self.stack_level];
        self.stack_level = self.stack_level - 1;
    }

    fn jump(&mut self, address: u16) {
        self.pc = address as usize;
    }

    fn call_at(&mut self, address: usize) {
        self.stack_level = self.stack_level + 1;
        self.stack[self.stack_level] = self.pc;
        self.pc = address;
    }

    fn skip_if_eq(&mut self, x: u8, y:u8) {
        if x == y {
            self.pc += 2
        }
    }

    fn skip_if_not_eq(&mut self, x: u8, y:u8) {
        if x != y {
            self.pc += 2
        }
    }

    fn set_vx_rand(&mut self, x: u8, seed: u8) {
        let rand_number = rand::thread_rng().gen_range(0..255);
        self.registers[x as usize] = rand_number & seed;
    }

    fn store_least_sig_vx_bit(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        let lsb = vx & 1;

        self.registers[0xF] = lsb;
        self.registers[x as usize] = vx >> 1;
    }

    fn store_most_sig_vx_bit(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        let msb = (vx >> 7) & 1;

        self.registers[0xF] = msb;
        self.registers[x as usize] = vx << 1;
    }

    fn set_i_location_of_vx_character_sprite(&mut self, x: u8) {
        println!("SETTING LOCATION OF I TO FONT_ADDR + V{}", x);
        self.i = FONT_ADDR + (x as usize * font::FONT_SIZE);
    }

    fn print_screen(&self) {
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

    //Draw sprite at coord (x, y) that is 8 pixels wide and the height arg tall
    fn draw(&mut self, x: u8, y:u8, height: u8) {
        let vx = self.registers[x as usize] as usize;
        let vy = self.registers[y as usize] as usize;
        let mut starting_index = vx + vy * DISPLAY_WIDTH as usize;
        println!("SELF.I: {:#04X}\tX: {}\tY: {}", self.i, vx, vy);

        self.registers[0xF] = 0;

        for row in 0..height {
            // Each row is a byte in memory, so to get the next row, go to the next memory addr
            let pixel_pattern = self.memory[self.i + row as usize];

            for offset in 0..8 {
                let pixel_bit = (pixel_pattern >> 7 - offset) & 1;
                let pixel_index = starting_index + offset;
                if pixel_index >= DISPLAY_WIDTH * (row as usize + 1) {
                    println!("{} {} SKIPPING THIS ONE: {} - {} {}", x, y, pixel_index, row, offset);
                } else {
                    println!("{} {} DRAWING: {} - {} {}", x, y, pixel_index, row, offset);
                }

                // set VF to 1 if any screen pixels are flipped from set to unset when sprite is drawn
                if self.display[pixel_index] == true && pixel_bit == 0 {
                    self.registers[0xF] = 1; 
                }
                self.display[pixel_index] = pixel_bit == 1;
            }

            starting_index += DISPLAY_WIDTH;
        }

            self.print_screen();
    }

    fn skip_if_key_press(&mut self, _x: u8) { }

    fn skip_if_not_key_press(&mut self, _x: u8) { }

    fn keypress_to_value(&self, keypress: String) -> u8 {
        match keypress.trim() {
            "0" => 0x0,
            "1" => 0x1,
            "2" => 0x2,
            "3" => 0x3,
            "4" => 0x4,
            "5" => 0x5,
            "6" => 0x6,
            "7" => 0x7,
            "8" => 0x8,
            "9" => 0x9,
            "a" => 0xA,
            "b" => 0xB,
            "c" => 0xC,
            "d" => 0xD,
            "e" => 0xE,
            "f" => 0xF,
            _ => panic!("UNKNOWN KEY")
        }
    }

    fn await_then_store_keypress(&mut self, x: u8) {
        //TODO - do not require hitting enter
        //TODO - be able to map keyboard keys to the hex keyboard
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).expect("Failed to read line");

        let keypress_value = self.keypress_to_value(line);

        self.registers[x as usize] = keypress_value
    }

    fn store_vx_binary_at_i(&mut self, x: u8) {
        let number = self.registers[x as usize];
        let ones = number % 10;
        let tens = number / 10 % 10;
        let hundreds = number / 10 / 10 % 10;

        self.memory[self.i] = hundreds;
        self.memory[1 + self.i] = tens;
        self.memory[2 + self.i] = ones;
    }

    fn store_registers_to_i(&mut self, x: u8) {
        for register in 0..x {
            let address = self.i + register as usize;
            self.memory[address] = self.registers[register as usize];
        }
    }

    fn load_registers_to_i(&mut self, x: u8) {
        for register in 0..x {
            let address = self.i + register as usize;
            self.registers[register as usize] = self.memory[address]
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
        let nnn = instruction & 0x0FFF;

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
            [3, x, _, _] => self.skip_if_eq(x, decoded_instruction.nn),
            [4, x, _, _] => self.skip_if_not_eq(x, decoded_instruction.nn),
            [5, x, y, 0x0] => self.skip_if_eq(x, self.registers[y as usize]),
            [6, x, _, _] => self.registers[x as usize] = decoded_instruction.nn,
            [7, x, _, _] => self.registers[x as usize] += decoded_instruction.nn,
            [8, x, y, 0] => self.registers[x as usize] = self.registers[y as usize],
            [8, x, y, 1] => self.registers[x as usize] = self.registers[x as usize] | self.registers[y as usize],
            [8, x, y, 2] => self.registers[x as usize] = self.registers[x as usize] & self.registers[y as usize],
            [8, x, y, 3] => self.registers[x as usize] = self.registers[x as usize] ^ self.registers[y as usize],
            [8, x, y, 4] => self.registers[x as usize] = self.registers[x as usize] + self.registers[y as usize],
            [8, x, y, 5] => self.registers[x as usize] = self.registers[x as usize] - self.registers[y as usize],
            [8, x, _, 6] => self.store_least_sig_vx_bit(x),
            [8, x, y, 7] => self.registers[x as usize] = self.registers[y as usize] - self.registers[x as usize],
            [8, x, _, 0xE] => self.store_most_sig_vx_bit(x),
            [9, x, y, 0] => self.skip_if_not_eq(x, self.registers[y as usize]),
            [0xA, _, _, _] => self.i = decoded_instruction.nnn as usize,
            [0xB, _, _, _] => self.pc = (self.registers[0] as u16 + decoded_instruction.nnn) as usize,
            [0xC, x, _, _] => self.set_vx_rand(x, decoded_instruction.nn),
            [0xD, x, y, n] => self.draw(x, y, n),
            [0xE, x, 0x9, 0xE] => self.skip_if_key_press(x),
            [0xE, x, 0xA, 0x1] => self.skip_if_not_key_press(x),
            [0xF, x, 0x0, 0x7] => self.registers[x as usize] = self.delay_timer.get(),
            [0xF, x, 0x0, 0xA] => self.await_then_store_keypress(x),
            [0xF, x, 0x1, 0x5] => self.delay_timer.set(self.registers[x as usize]),
            [0xF, x, 0x1, 0x8] => self.sound_timer.set(self.registers[x as usize]),
            [0xF, x, 0x1, 0xE] => self.i += self.registers[x as usize] as usize,
            [0xF, x, 0x2, 0x9] => self.set_i_location_of_vx_character_sprite(x),
            [0xF, x, 0x3, 0x3] => self.store_vx_binary_at_i(x),
            [0xF, x, 0x5, 0x5] => self.store_registers_to_i(x),
            [0xF, x, 0x6, 0x5] => self.load_registers_to_i(x),
            _ => ()
        }
    }

    fn step(&mut self) {
        let instruction = self.fetch();
        let decoded_instruction = self.decode(instruction);
        self.execute(decoded_instruction);
    }

    fn load_rom(&mut self, rom: &Vec<u8>) {
        for (offset, byte) in rom.iter().enumerate() {
            self.memory[ROM_ADDR + offset] = *byte;
            self.pc = ROM_ADDR;
        }
    }

    pub fn run(&mut self, rom: &Vec<u8>) {
        //Put the passed rom 
        self.load_rom(rom);

        print!("MEMORY: [");
        for b in self.memory {
            print!("{:#X} ", b)
        }
        println!("]");

        loop {
            self.step()
        }
    }
}

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


    #[test]
    fn fetch_gets_two_byte_instruction_at_pc() {
        let mut chip = Chip::new();
        chip.pc = 30;

        chip.memory[30] = 0xAB;
        chip.memory[31] = 0xCD;

        let instruction = chip.fetch();

        assert_eq!(instruction, 0xABCD);
    }

    #[test]
    fn fetch_increments_pc_by_2() {
        let mut chip = Chip::new();
        chip.pc = 30;

        chip.fetch();

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
    fn store_vx_binary_at_i_123() {
        let mut chip = Chip::new();
        chip.registers[4] = 123;
        chip.i = 10;

        chip.store_vx_binary_at_i(4);

        assert_eq!(chip.memory[10], 0x1);
        assert_eq!(chip.memory[11], 0x2);
        assert_eq!(chip.memory[12], 0x3);
    }

    #[test]
    fn store_vx_binary_at_i_999() {
        let mut chip = Chip::new();
        chip.registers[8] = 255;

        chip.store_vx_binary_at_i(8);

        assert_eq!(chip.memory[0], 0x2);
        assert_eq!(chip.memory[1], 0x5);
        assert_eq!(chip.memory[2], 0x5);
    }

    #[test]
    fn store_vx_binary_at_i_000() {
        let mut chip = Chip::new();
        chip.registers[8] = 0;

        chip.store_vx_binary_at_i(8);

        assert_eq!(chip.memory[0], 0x0);
        assert_eq!(chip.memory[1], 0x0);
        assert_eq!(chip.memory[2], 0x0);
    }

    #[test]
    fn store_most_sig_vx_bit_1() {
        let mut chip = Chip::new();
        chip.registers[8] = 0b10000001;

        assert_eq!(chip.registers[0xF], 0);

        chip.store_most_sig_vx_bit(8);

        assert_eq!(chip.registers[0x8], 0b00000010);
        assert_eq!(chip.registers[0xF], 1);
    }

    #[test]
    fn store_most_sig_vx_bit_0() {
        let mut chip = Chip::new();
        chip.registers[8] = 0b01111111;

        chip.store_most_sig_vx_bit(8);

        assert_eq!(chip.registers[0x8], 0b11111110);
        assert_eq!(chip.registers[0xF], 0);
    }

    #[test]
    fn store_least_sig_vx_bit_1() {
        let mut chip = Chip::new();
        chip.registers[0xA] = 0b11111101;

        assert_eq!(chip.registers[0xF], 0);

        chip.store_least_sig_vx_bit(0xA);

        assert_eq!(chip.registers[0xA], 0b01111110);
        assert_eq!(chip.registers[0xF], 1);
    }

    #[test]
    fn store_least_sig_vx_bit_0() {
        let mut chip = Chip::new();
        chip.registers[0xB] = 0b10000010;

        chip.store_least_sig_vx_bit(0xB);

        assert_eq!(chip.registers[0xB], 0b01000001);
        assert_eq!(chip.registers[0xF], 0);
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

    /* Commenting out since you have to type, but it did allow me to test manually
     * There has to a better way to test this, or even yet a better way to get the input
    #[test]
    fn await_then_store_keypress_works() {
        let mut chip = Chip::new();
        let vx: u8 = 5;

        chip.await_then_store_keypress(vx);

        assert_eq!(chip.registers[vx as usize], 0xE)
    }
    *
    */
}
