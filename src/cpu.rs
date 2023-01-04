use crate::font::FONT;
use rand::Rng;
use sdl2::keyboard::Keycode;

const RAM_SIZE: usize = 4096;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const NUM_REGISTERS: usize = 16;
const NUM_KEYS: usize = 16;

type Nibbles = (u16, u16, u16, u16);

pub struct CPU {
    ram: [u8; RAM_SIZE],
    pub vram: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    stack: Vec<usize>,
    pc: usize,
    i: usize,
    delay_timer: u8,
    sound_timer: u8,
    registers: [u16; NUM_REGISTERS],
    inputs: [bool; NUM_KEYS]
}

#[derive(Debug)]
pub struct Opcode {
    nibbles: Nibbles, 
    x: u16,
    y: u16,
    n: u16,
    nn: u16,
    nnn: u16
}

impl CPU {
    pub fn new(rom: &Vec<u8>) -> Self {
        let mut ram = [0u8; RAM_SIZE];

        for i in 0..FONT.len() {
            ram[0x50 + i] = FONT[i];
        }

        for i in 0..rom.len() {
            ram[0x200 + i] = rom[i];
        }

        CPU {
            vram: [[0u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
            ram,
            stack: vec![],
            i: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            registers: [0u16; NUM_REGISTERS],
            inputs: [false; NUM_KEYS]
        }
    }

    pub fn key_pressed(&mut self, key: Keycode) {
        let key_index = CPU::get_input_mapping(key);

        match key_index {
            Some(index) => self.inputs[index] = true,
            None => {},
        }
    }

    pub fn key_released(&mut self, key: Keycode) {
        let key_index = CPU::get_input_mapping(key);

        match key_index {
            Some(index) => self.inputs[index] = false,
            None => {},
        }
    }

    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn get_input_mapping(key: Keycode) -> Option<usize> {
        match key {
            Keycode::Num1 => Some(0x1),
            Keycode::Num2 => Some(0x2),
            Keycode::Num3 => Some(0x3),
            Keycode::Num4 => Some(0xC),
            Keycode::Q => Some(0x4),
            Keycode::W => Some(0x5),
            Keycode::E => Some(0x6),
            Keycode::R => Some(0xD),
            Keycode::A => Some(0x7),
            Keycode::S => Some(0x8),
            Keycode::D => Some(0x9),
            Keycode::F => Some(0xE),
            Keycode::Z => Some(0xA),
            Keycode::X => Some(0x0),
            Keycode::C => Some(0xB),
            Keycode::V => Some(0xF),

            _ => None
        }
    }

    fn clear_screen(&mut self) {
        for i in 0..SCREEN_HEIGHT {
            for j in 0..SCREEN_WIDTH {
                self.vram[i][j] = 0x00;
            }
        } 
    }

    fn jump(&mut self, opcode: Opcode) {
        self.pc = opcode.nnn as usize; 
    }

    fn set_register(&mut self, opcode: Opcode) {
        self.registers[opcode.x as usize] = opcode.nn;
    }

    fn add_to_register(&mut self, opcode: Opcode) {
        let reg_x = self.registers[opcode.x as usize] as u8;
        let value = opcode.nn as u8;

        let (result, _) = reg_x.overflowing_add(value);
        self.registers[opcode.x as usize] = result as u16;
    }

    fn set_index_register(&mut self, opcode: Opcode) {
        self.i = opcode.nnn as usize; 
    }

    fn draw(&mut self, opcode: Opcode) {
        let x = self.registers[opcode.x as usize] as usize;
        let y = self.registers[opcode.y as usize] as usize;
        self.registers[0xF] = 0;

        let base = self.i;

        for row in 0..opcode.n {
            let sprite_data = self.ram[base + row as usize]; 
            let y_pos = y + row as usize;

            if y_pos >= SCREEN_HEIGHT {
                break;
            }

            for i in 0..8 {
                let x_pos = x + i as usize;

                if x_pos >= SCREEN_WIDTH {
                    break;
                }

                let sprite_pixel_on = sprite_data & (0x80 >> i) > 0;
                let screen_on = self.vram[y_pos][x_pos] > 0;

                if sprite_pixel_on && screen_on {
                    self.vram[y_pos][x_pos] = 0;
                    self.registers[0xF] = 1;
                } else if sprite_pixel_on && !screen_on {
                    self.vram[y_pos][x_pos] = 1;
                }
            }
        }
    }

    fn call_subroutine(&mut self, opcode: Opcode) {
        self.stack.push(self.pc);
        self.pc = opcode.nnn as usize;
    }

    fn return_from_subroutine(&mut self) {
        let return_address = self.stack.pop().unwrap();
        self.pc = return_address;
    }

    fn skip_compare_reg_value(&mut self, opcode: Opcode, equal: bool) {
        let reg = self.registers[opcode.x as usize];

        if (equal && reg == opcode.nn) || (!equal && reg != opcode.nn) {
            self.pc += 2;
        }
    }

    fn skip_compare_registers(&mut self, opcode: Opcode, equal: bool) {
        let reg_x = self.registers[opcode.x as usize];
        let reg_y = self.registers[opcode.y as usize];

        if (equal && reg_x == reg_y) || (!equal && reg_x != reg_y) {
            self.pc += 2;
        }
    }

    fn copy_register(&mut self, opcode: Opcode) {
        self.registers[opcode.x as usize] = self.registers[opcode.y as usize];
    }

    fn binary_or(&mut self, opcode: Opcode) {
        self.registers[opcode.x as usize] |= self.registers[opcode.y as usize];
    }

    fn binary_and(&mut self, opcode: Opcode) {
        self.registers[opcode.x as usize] &= self.registers[opcode.y as usize];
    }

    fn binary_xor(&mut self, opcode: Opcode) {
        self.registers[opcode.x as usize] ^= self.registers[opcode.y as usize];
    }

    fn add_register(&mut self, opcode: Opcode) {
        let reg_x = self.registers[opcode.x as usize] as u8;
        let reg_y = self.registers[opcode.y as usize] as u8;
        let (result, did_overflow) = reg_x.overflowing_add(reg_y);
        
        self.registers[opcode.x as usize] = result as u16;
        self.registers[0xF] = if did_overflow { 1 } else { 0 };
    }

    fn subtract_register(&mut self, opcode: Opcode, opposite: bool) {
        let reg_x = self.registers[opcode.x as usize] as u8;
        let reg_y = self.registers[opcode.y as usize] as u8;

        let (result, did_underflow) = if opposite { reg_y.overflowing_sub(reg_x) } else { reg_x.overflowing_sub(reg_y) };

        self.registers[opcode.x as usize] = result as u16;
        self.registers[0xF] = if did_underflow { 0 } else { 1 };
    }

    fn shift_right(&mut self, opcode: Opcode) {
        // Optional
        // self.registers[opcode.x as usize] = self.registers[opcode.y as usize];

        let reg_x = self.registers[opcode.x as usize]; 
        self.registers[0xF] = reg_x & 0x01;
        self.registers[opcode.x as usize] = (reg_x >> 1) & 0xFF;
    }

    fn shift_left(&mut self, opcode: Opcode) {
        // Optional
        // self.registers[opcode.x as usize] = self.registers[opcode.y as usize];
        //
        let reg_x = self.registers[opcode.x as usize]; 
        self.registers[0xF] = (reg_x & 0x80) >> 7; 
        self.registers[opcode.x as usize] = (reg_x << 1) & 0xFF;
    }

    fn jump_with_offset(&mut self, opcode: Opcode) {
        self.pc += (opcode.nnn + self.registers[0x0]) as usize;
    }

    fn random(&mut self, opcode: Opcode) {
        let mut rng = rand::thread_rng();    
        let random_num: u16 = rng.gen();

        self.registers[opcode.x as usize] = random_num & opcode.nn;
    }

    fn copy_delay_timer_to_register(&mut self, opcode: Opcode) {
        self.registers[opcode.x as usize] = self.delay_timer as u16;
    }

    fn copy_register_to_delay_timer(&mut self, opcode: Opcode) {
        self.delay_timer = self.registers[opcode.x as usize] as u8;
    }

    fn copy_register_to_sound_timer(&mut self, opcode: Opcode) {
        self.sound_timer = self.registers[opcode.x as usize] as u8;
    }

    fn add_to_index(&mut self, opcode: Opcode) {
        let new_i = self.i + self.registers[opcode.x as usize] as usize;

        let did_overflow = new_i >= 0x1000;
        self.i = new_i & 0xFFF;
        self.registers[0xF] = if did_overflow { 1 } else { 0 };
    }

    fn skip_if_key(&mut self, opcode: Opcode, pressed: bool) {
        let reg_x = self.registers[opcode.x as usize];

        if (pressed && self.inputs[reg_x as usize]) || (!pressed && !self.inputs[reg_x as usize]) {
            self.pc += 2;
        }
    }

    fn font_character(&mut self, opcode: Opcode) {
        let reg_x = self.registers[opcode.x as usize] as usize;
        self.i = 0x50 + reg_x * 5;
    }

    fn wait_for_key(&mut self, opcode: Opcode) {
        self.pc -= 2;

        for i in 0..self.inputs.len() {
            if self.inputs[i] {
                self.registers[opcode.x as usize] = i as u16;
                break;
            }
        }
    }

    fn binary_coded_decimal_conversion(&mut self, opcode: Opcode) {
        let reg_x = self.registers[opcode.x as usize];

        self.ram[self.i] = (reg_x / 100) as u8;
        self.ram[self.i + 1] = (reg_x % 100 / 10) as u8;
        self.ram[self.i + 2] = ((reg_x % 100) % 10) as u8;
    }

    fn store_registers_to_memory(&mut self, opcode: Opcode) {
        for i in 0..(opcode.x as usize) + 1 {
            self.ram[self.i + i] = self.registers[i] as u8;
        }
    }

    fn load_registers_from_memory(&mut self, opcode: Opcode) {
        for i in 0..(opcode.x as usize) + 1 {
            self.registers[i] = self.ram[self.i + i] as u16;
        }
    }

    pub fn execute_tick(&mut self) -> Result<(), &str> {
        let opcode = self.get_next_opcode();

        match opcode.nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.clear_screen(),
            (0x0, 0x0, 0xE, 0xE) => self.return_from_subroutine(),
            (0x1, _, _, _) => self.jump(opcode),
            (0x2, _, _, _) => self.call_subroutine(opcode),
            (0x3, _, _, _) => self.skip_compare_reg_value(opcode, true),
            (0x4, _, _, _) => self.skip_compare_reg_value(opcode, false),
            (0x5, _, _, _) => self.skip_compare_registers(opcode, true),
            (0x6, _, _, _) => self.set_register(opcode),
            (0x7, _, _, _) => self.add_to_register(opcode),
            (0x8, _, _, _) => {
                match opcode.nibbles.3 {
                    0x0 => self.copy_register(opcode),
                    0x1 => self.binary_or(opcode),
                    0x2 => self.binary_and(opcode),
                    0x3 => self.binary_xor(opcode),
                    0x4 => self.add_register(opcode),
                    0x5 => self.subtract_register(opcode, false),
                    0x6 => self.shift_right(opcode),
                    0x7 => self.subtract_register(opcode, true),
                    0xE => self.shift_left(opcode),
                    _ => {}
                }
            }
            (0x9, _, _, _) => self.skip_compare_registers(opcode, false),
            (0xA, _, _, _) => self.set_index_register(opcode),
            (0xB, _, _, _) => self.jump_with_offset(opcode),
            (0xC, _, _, _) => self.random(opcode),
            (0xD, _, _, _) => self.draw(opcode),
            (0xE, _, 0x9, 0xE) => self.skip_if_key(opcode, true),
            (0xE, _, 0xA, 0x1) => self.skip_if_key(opcode, false),
            (0xF, _, 0x0, 0x7) => self.copy_delay_timer_to_register(opcode),
            (0xF, _, 0x0, 0xA) => self.wait_for_key(opcode),
            (0xF, _, 0x1, 0x5) => self.copy_register_to_delay_timer(opcode),
            (0xF, _, 0x1, 0x8) => self.copy_register_to_sound_timer(opcode),
            (0xF, _, 0x1, 0xE) => self.add_to_index(opcode),
            (0xF, _, 0x2, 0x9) => self.font_character(opcode),
            (0xF, _, 0x3, 0x3) => self.binary_coded_decimal_conversion(opcode),
            (0xF, _, 0x5, 0x5) => self.store_registers_to_memory(opcode),
            (0xF, _, 0x6, 0x5) => self.load_registers_from_memory(opcode),
            _ => ()
        };

        Ok(())
    }

    pub fn get_next_opcode(&mut self) -> Opcode {
        let operation: u16 = (self.ram[self.pc] as u16) << 8 | self.ram[self.pc + 1] as u16;
        self.pc += 2;

        let nibbles = (
            (operation & 0xF000) >> 12,
            (operation & 0x0F00) >> 8,
            (operation & 0x00F0) >> 4,
            (operation & 0x000F),
        );

        Opcode { 
            nibbles: (
                (operation & 0xF000) >> 12,
                (operation & 0x0F00) >> 8,
                (operation & 0x00F0) >> 4,
                (operation & 0x000F),
            ), 
            x: nibbles.1,
            y: nibbles.2,
            n: nibbles.3,
            nn: operation & 0x00FF,
            nnn: operation & 0x0FFF
        }
    }

}
