use std::fs::File;
use std::io::Read;

// Chip8’s memory from 0x000 to 0x1FF is reserved, so the ROM instructions must start at 0x200
const START_ADDRESS: u16 = 0x200;
const FONTSET_START_ADDRESS: u8 = 0x50;
const FONTSET_SIZE: u32 = 80;

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
]

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

    // 1nnn - JP: Jump to address nnn
    fn op_1nnn(&mut self) {
        let address = self.opcode & 0x0FFF;
        self.pc = address;
    }
}

fn main() {

}
