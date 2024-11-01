use std::fs::File;
use std::io::Read;

// Chip8’s memory from 0x000 to 0x1FF is reserved, so the ROM instructions must start at 0x200
static START_ADDRESS: u16 = 0x200;

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

fn main() {
    
}
