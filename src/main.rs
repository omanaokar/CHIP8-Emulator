use std::fs::File;
use std::io::Read;
use rand::Rng;

// Chip8’s memory from 0x000 to 0x1FF is reserved, so the ROM instructions must start at 0x200
const START_ADDRESS: u16 = 0x200;
const FONTSET_START_ADDRESS: u8 = 0x50;
const FONTSET_SIZE: u32 = 80;
const VIDEO_WIDTH: u32 = 64;
const VIDEO_HEIGHT: u32 = 32;

const fontset: [u8; 80] = 
[
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
	0x20, 0x60, 0x20, 0x20, 0x70, // 1
	0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
	0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
	0x90, 0x90, 0xF0, 0x10, 0x10, // 4
	0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
	0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
	0xF0, 0x10, 0x20, 0x40, 0x40, // 7
	0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
	0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
	0xF0, 0x90, 0xF0, 0x90, 0x90, // A
	0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
	0xF0, 0x80, 0x80, 0x80, 0xF0, // C
	0xE0, 0x90, 0x90, 0x90, 0xE0, // D
	0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
	0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

// Struct for CHIP8 structure
struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    video: [u32; 64*32],
    opcode: u16
}

// Constructor
impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            registers: [0; 16],       // Default values for registers
            memory: [0; 4096],        // Default values for memory
            index: 0,                 // Default value for index
            pc: START_ADDRESS,        // Initialize pc to 0x200
            stack: [0; 16],           // Default values for stack
            sp: 0,                    // Default value for stack pointer
            delay_timer: 0,           // Default value for delay timer
            sound_timer: 0,           // Default value for sound timer
            keypad: [0; 16],          // Default values for keypad
            video: [0; 64 * 32],      // Default values for video
            opcode: 0,                // Default value for opcode
        }
    }
}

// Opens contents of ROM file into memory
impl Chip8 {
    fn load_rom(&mut self, filename: &String) {
        let mut f = File::open(filename).expect("Error opening image...");
        let mut buffer = Vec::new();

        f.read_to_end(&mut buffer).expect("Error reading file..."); // Opens as a vector of bytes

        for i in 0..buffer.len() {
            let addr = START_ADDRESS as usize;
            self.memory[addr + i] = buffer[i];
        }
    }
}


// Loads font set into memory
impl Chip8 {
    fn load_fonts(&mut self) {
        for i in 0..FONTSET_SIZE {
            let fnt_addr = FONTSET_START_ADDRESS as usize;
            let idx = i as usize;
            self.memory[fnt_addr + idx] = fontset[idx];
        }
    }
}

impl Chip8 {
    // 00E0 - CLS: Clears display
    fn op_00e0(&mut self) {
        self.video.fill(0);
    }

    // 00EE - RET: Return from a subroutine
    fn op_00ee(&mut self) {
        self.sp -= 1;
        let pc = self.pc as usize;
        self.pc = self.stack[pc];
    }

    // 1nnn - JP addr: Jump to address nnn
    fn op_1nnn(&mut self) {
        let address = self.opcode & 0x0FFF;
        self.pc = address;
    }

    // 2nnn - CALL addr: Call subroutine at nnn
    fn op_2nnn(&mut self) {
        let sp = self.sp as usize;
        self.stack[sp] = self.pc;
        self.sp += 1;
        let address = self.opcode & 0x0FFF;
        self.pc = address;
    }

    // 3xkk - SE Vx, byte: Skip next instruction if Vx = kk
    fn op_3xkk(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let byte = (self.opcode & 0x00FF) as u8;
        let vx_idx = Vx as usize;
        if self.registers[vx_idx] == byte {
            self.pc += 2;
        }
    }

    // 4xkk - SNE Vx, byte: Skip next instruction if Vx != kk
    fn op_4xkk(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let byte = (self.opcode & 0x00FF) as u8;
        let vx_idx = Vx as usize;
        if self.registers[vx_idx] != byte {
            self.pc += 2;
        }
    }

    // 5xy0 - SE Vx, Vy: Skip next instruction if Vx = Vy
    fn op_5xy0(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let Vy = ((self.opcode & 0x00F0) >> 4) as u8;
        let vx_idx = Vx as usize;
        let vy_idx = Vy as usize;
        if self.registers[vx_idx] == self.registers[vy_idx] {
            self.pc += 2;
        }
    }

    // 6xkk - LD Vx, byte: Interpreted puts value kk into register Vx
    fn op_6xkk(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let byte = (self.opcode & 0x00FF) as u8;

        let vx_idx = Vx as usize;

        self.registers[vx_idx] = byte;
    }

    // 7xkk - ADD Vx, byte: Set Vx = Vx + kk
    fn op_7xkk(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let byte = (self.opcode & 0x00FF) as u8;

        let vx_idx = Vx as usize;

        self.registers[vx_idx] += byte;
    }

    // 8xy0 - LD Vx, Vy: Set Vx = Vx + kk
    fn op_8xy0(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let Vy = ((self.opcode & 0x00F0) >> 4) as u8;

        let vx_idx = Vx as usize;
        let vy_idx = Vy as usize;

        self.registers[vx_idx] = self.registers[vy_idx];       
    }

    // 8xy1 - OR Vx, Vy: Set Vx = Vx OR Vy
    fn op_8xy1(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let Vy = ((self.opcode & 0x00F0) >> 4) as u8;

        let vx_idx = Vx as usize;
        let vy_idx = Vy as usize;

        self.registers[vx_idx] |= self.registers[vy_idx];       
    }

    // 8xy2 - AND Vx, Vy: Set Vx = Vx AND Vy
    fn op_8xy2(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let Vy = ((self.opcode & 0x00F0) >> 4) as u8;

        let vx_idx = Vx as usize;
        let vy_idx = Vy as usize;

        self.registers[vx_idx] &= self.registers[vy_idx];       
    }

    // 8xy3 - XOR Vx, Vy: Set Vx = Vx XOR Vy
    fn op_8xy3(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let Vy = ((self.opcode & 0x00F0) >> 4) as u8;

        let vx_idx = Vx as usize;
        let vy_idx = Vy as usize;

        self.registers[vx_idx] ^= self.registers[vy_idx];       
    }

    // 8xy4 - ADD Vx, Vy: Set Vx = Vx + Vy, set VF = carry
    fn op_8xy4(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let Vy = ((self.opcode & 0x00F0) >> 4) as u8;

        let vx_idx = Vx as usize;
        let vy_idx = Vy as usize;

        let sum = (self.registers[vx_idx] + self.registers[vy_idx]) as u16;

        if sum > 255 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }     
        self.registers[vx_idx] = (sum & 0xFF) as u8;
    }

    // 8xy5 - SUB Vx, Vy: Set Vx = Vx - Vy, set VF = NOT borrow
    fn op_8xy5(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let Vy = ((self.opcode & 0x00F0) >> 4) as u8;

        let vx_idx = Vx as usize;
        let vy_idx = Vy as usize;

        if self.registers[vx_idx] > self.registers[vy_idx] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.registers[vx_idx] -= self.registers[vy_idx];
    }

    // 8xy6 - SHR Vx: Set Vx = Vx SHR 1
    fn op_8xy6(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;

        let vx_idx = Vx as usize;
        
        self.registers[0xF] = self.registers[vx_idx] & 0x1;

        self.registers[vx_idx] >>= 1;
    }

    // 8xy7 - SUBN Vx, Vy: Set Vx = Vy - Vx, set VF = NOT borrow
    fn op_8xy7(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let Vy = ((self.opcode & 0x00F0) >> 4) as u8;

        let vx_idx = Vx as usize;
        let vy_idx = Vy as usize;

        if self.registers[vy_idx] > self.registers[vx_idx] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }     
        self.registers[vx_idx] = self.registers[vy_idx] - self.registers[vx_idx];
    }

    // 8xyE - SHL Vx: Set Vx = Vx SHL 1
    fn op_8xyE(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let vx_idx = Vx as usize;

        self.registers[0xF] = (self.registers[vx_idx] & 0x80) >> 7;

        self.registers[vx_idx] <<= 1;
    }

    // 9xy0 - SNE Vx, Vy: Skip next instruction if Vx != Vy
    fn op_9xy0(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let Vy = ((self.opcode & 0x00F0) >> 4) as u8;

        let vx_idx = Vx as usize;
        let vy_idx = Vy as usize;

        if self.registers[vx_idx] != self.registers[vy_idx] {
            self.pc += 2;
        }
    }

    // Annn - LD I, addr: Set I = nnn
    fn op_Annn(&mut self) {
        let address = self.opcode & 0x0FFF;

        self.index = address;
    }

    // Bnnn - JP V0, addr: Jump to location nnn + V0
    fn op_Bnnnn(&mut self) {
        let address = self.opcode & 0x0FFF;

        self.pc = (self.registers[0] as u16) + address;
    }

    // Cxkk - RND Vx, byte: Set Vx = random byte AND kk
    fn op_Cxkk(&mut self) {
        let mut rng = rand::thread_rng();
        
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let byte = (self.opcode & 0x00FF) as u8;

        let vx_idx = Vx as usize;

        self.registers[vx_idx] = rng.gen::<u8>() & byte;
    }

    // Dxyn - DRW Vx, Vy, nibble: Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
    fn op_Dxyn(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let Vy = ((self.opcode & 0x00F0) >> 4) as u8;
        let height = (self.opcode & 0x000F) as u8;

        
        let vx_idx = Vx as usize;
        let vy_idx = Vy as usize;


        let xPos = self.registers[vx_idx] % (VIDEO_WIDTH as u8);
        let yPos = self.registers[vy_idx] % (VIDEO_HEIGHT as u8);

        self.registers[0xF] = 0;

        for row in 0..height {
            let spriteByte = self.memory[(self.index + (row as u16)) as usize];

            for col in 0..8 {
                let spritePixel = spriteByte & (0x80 >> col);
                let mut screenPixel = self.video[(((yPos + row) as u32) * VIDEO_WIDTH + ((xPos + col) as u32)) as usize];

                if spritePixel != 0 {
                    if screenPixel == 0xFFFFFFFF {
                        self.registers[0xF] = 1;
                    }

                    screenPixel ^= 0xFFFFFFFF;
                }
            }
        }
    }

    // Ex9E - SKP Vx: Skip next instruction if key with the value of Vx is pressed
    fn op_Ex9E(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let vx_idx = Vx as usize; 

        let key = self.registers[vx_idx];

        let keypad: Option<u8> = Some(self.keypad[key as usize]);

        if keypad.is_some() {
            self.pc += 2;
        }
    }

    // ExA1 - SKNP Vx: Skip next instruction if key with the value of Vx is not pressed
    fn op_ExA1(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let vx_idx = Vx as usize; 

        let key = self.registers[vx_idx];

        let keypad: Option<u8> = Some(self.keypad[key as usize]);
        
        if keypad.is_none() {
            self.pc += 2;
        }
    }

    // Fx07 - LD Vx, DT: Set Vx = delay timer value.
    fn op_Fx07(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let vx_idx = Vx as usize; 

        self.registers[vx_idx] = self.delay_timer;
    }
}

fn main() {

}
