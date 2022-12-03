use crate::font::FONT;

const RAM_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const VRAM_WIDTH: usize = 64;
const VRAM_HEIGHT: usize = 32;
const NUM_REGISTERS: usize = 16;

pub struct CPU {
    ram: [u8; RAM_SIZE],
    vram: [[u8; VRAM_WIDTH]; VRAM_HEIGHT],
    stack: [u16; STACK_SIZE],
    pc: usize,
    i: usize,
    delay_timer: u8,
    sound_timer: u8,
    registers: [u8; NUM_REGISTERS],
}

pub struct Opcode {
    nibbles: (u16, u16, u16, u16),
    x: u16,
    y: u16,
    n: u16,
    nn: u16,
    nnn: u16
}

impl CPU {
    pub fn new() -> Self {
        let mut ram = [0u8; RAM_SIZE];

        for i in 0..FONT.len() {
            ram[0x50 + i] = FONT[i];
        }

        CPU {
            vram: [[0u8; VRAM_WIDTH]; VRAM_HEIGHT],
            ram,
            stack: [0u16; STACK_SIZE],
            i: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            registers: [0u8; NUM_REGISTERS],
        }
    }

    pub fn execute_tick(&mut self) -> Result<(), &str> {
        let opcode = self.get_next_opcode();

        match opcode.nibbles {
            (0x00, 0x00, 0x0e, 0x00) => Ok(()), // 00E0
            (0x01, 0x4e, 0x4e, 0x4e) => Ok(()), // 1NNN
            (0x06, 0x58, 0x4e, 0x4e) => Ok(()), // 6XNN
            (0x07, 0x58, 0x4e, 0x4e) => Ok(()), // 7XNN
            (0x0a, 0x4e, 0x4e, 0x4e) => Ok(()), // ANNN
            (0x0d, 0x00, 0x00, 0x00) => Ok(()), // DXYN

            _ => Err("unknown instruction")
        }
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

    pub fn test(&self) {
        println!("Hello from the CPU! {:#x}", self.ram[0x050])
    }
}
