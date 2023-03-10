use rand::Rng;

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

const DISPLAY_SIZE: usize = 64 * 32;

#[derive(Debug)]
pub struct Chip {
    pub memory: [u8; 4096],
    pub stack: [u16; 32],
    pub display: [bool; DISPLAY_SIZE],
    pub registers: [u8; 16],
    pub i: u16,
    pub delay_timer: Timer,
    pub sound_timer: Timer,
    pub pc: u16
}

impl Chip {
    pub fn new() -> Self {
        Chip {
            memory: [0; 4096],
            stack: [0; 32],
            display: [false; DISPLAY_SIZE],
            registers: [0; 16],
            i: 0,
            delay_timer: Timer::new(),
            sound_timer: Timer::new(),
            pc: 0
        }
    }

    fn reset_display(&mut self) {
        self.display = [false; DISPLAY_SIZE];
    }

    fn handle_return(&self) { }

    fn jump(&mut self, address: u16) {
        self.pc = address
    }

    fn call_at(&mut self, address: u16) { }

    fn skip_if_eq(&mut self, x: u8, y:u8) {
        if x == y {
            self.pc += 1
        }
    }

    fn skip_if_not_eq(&mut self, x: u8, y:u8) {
        if x != y {
            self.pc += 1
        }
    }

    fn set_vx_rand(&mut self, x: u8, seed: u8) {
        let rand_number = rand::thread_rng().gen_range(0..255);
        self.registers[x as usize] = rand_number & seed;
    }

    fn store_least_sig_vx_bit(&mut self, x: u8, y:u8) { }

    fn store_most_sig_vx_bit(&mut self, x: u8, y:u8) { }

    fn set_i_location_of_vx_character_sprite(&mut self, x: u8) { }

    fn draw(&mut self, x: u8, y:u8, n: u8) { }

    fn skip_if_key_press(&mut self, x: u8) { }

    fn skip_if_not_key_press(&mut self, x: u8) { }

    fn await_then_store_keypress(&mut self, x: u8) { }

    fn store_vx_binary_at_i(&mut self, x: u8) { }

    fn store_registers_to_i(&mut self, x: u8) {
        for register in 0..x {
            let address = (self.i + register as u16) as usize;
            self.memory[address] = self.registers[register as usize];
        }
    }

    fn load_registers_to_i(&mut self, x: u8) {
        for register in 0..x {
            let address = (self.i + register as u16) as usize;
            self.registers[register as usize] = self.memory[address]
        }
    }

    fn fetch(&mut self) -> u16 {
        let first_byte = self.memory[self.pc as usize] as u16;
        let second_byte = self.memory[1 + self.pc as usize] as u16;

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
            [0, 0, 0xE, 0xE] => self.reset_display(),
            [0, 0, 0xE, 0] => self.handle_return(),
            [0, _, _, _] => { },
            [1, _, _, _] => self.jump(decoded_instruction.nnn),
            [2, _, _, _] => self.call_at(decoded_instruction.nnn),
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
            [8, x, y, 6] => self.store_least_sig_vx_bit(x, y),
            [8, x, y, 7] => self.registers[x as usize] = self.registers[y as usize] - self.registers[x as usize],
            [8, x, y, 0xE] => self.store_most_sig_vx_bit(x, y),
            [9, x, y, 0] => self.skip_if_not_eq(x, self.registers[y as usize]),
            [0xA, _, _, _] => self.i = decoded_instruction.nnn,
            [0xB, _, _, _] => self.pc = self.registers[0] as u16 + decoded_instruction.nnn,
            [0xC, x, _, _] => self.set_vx_rand(x, decoded_instruction.nn),
            [0xD, x, y, n] => self.draw(x, y, n),
            [0xE, x, 0x9, 0xE] => self.skip_if_key_press(x),
            [0xE, x, 0xA, 0x1] => self.skip_if_not_key_press(x),
            [0xF, x, 0x0, 0x7] => self.registers[x as usize] = self.delay_timer.get(),
            [0xF, x, 0x0, 0xA] => self.await_then_store_keypress(x),
            [0xF, x, 0x1, 0x5] => self.delay_timer.set(self.registers[x as usize]),
            [0xF, x, 0x1, 0x8] => self.sound_timer.set(self.registers[x as usize]),
            [0xF, x, 0x1, 0xE] => self.i += self.registers[x as usize] as u16,
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

    pub fn run(&mut self) {
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
}
