use crate::font::FONT;

const RAM_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const NUM_REGISTERS: usize = 16;

type Nibbles = (u16, u16, u16, u16);

pub struct CPU {
    ram: [u8; RAM_SIZE],
    pub vram: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    stack: [u16; STACK_SIZE],
    pc: usize,
    i: usize,
    delay_timer: u8,
    sound_timer: u8,
    registers: [u16; NUM_REGISTERS],
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
            stack: [0u16; STACK_SIZE],
            i: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            registers: [0u16; NUM_REGISTERS],
        }
    }

    fn is_clear_screen_command(nibbles: Nibbles) -> bool {
        return nibbles.0 == 0x00 && nibbles.1 == 0x00 && nibbles.2 == 0x0E && nibbles.3 == 0x00;
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
        self.registers[opcode.x as usize] += opcode.nn;
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

    pub fn execute_tick(&mut self) -> Result<(), &str> {
        let opcode = self.get_next_opcode();

        match opcode.nibbles {
            (0, 0, 0xE, 0) => {
                if Self::is_clear_screen_command(opcode.nibbles) {
                    self.clear_screen();
                }
            },
            (0x1, _, _, _) => { 
                self.jump(opcode);
            },
            (0x6, _, _, _) => {
                self.set_register(opcode);
            },
            (0x7, _, _, _) => { 
                self.add_to_register(opcode);
            },
            (0xA, _, _, _) => { 
                self.set_index_register(opcode);
            },
            (0xD, _, _, _) => { 
                self.draw(opcode);
            },
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
