extern crate sdl2;

use std::fs::File;
use std::io::Read;
use std::env;
use std::process;
use rand::Rng;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{Canvas, Texture, TextureAccess};
use sdl2::video::Window;
use sdl2::Sdl;
use std::time::Duration;


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
    fn op_8xye(&mut self) {
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
    fn op_annn(&mut self) {
        let address = self.opcode & 0x0FFF;

        self.index = address;
    }

    // Bnnn - JP V0, addr: Jump to location nnn + V0
    fn op_bnnnn(&mut self) {
        let address = self.opcode & 0x0FFF;

        self.pc = (self.registers[0] as u16) + address;
    }

    // Cxkk - RND Vx, byte: Set Vx = random byte AND kk
    fn op_cxkk(&mut self) {
        let mut rng = rand::thread_rng();
        
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let byte = (self.opcode & 0x00FF) as u8;

        let vx_idx = Vx as usize;

        self.registers[vx_idx] = rng.gen::<u8>() & byte;
    }

    // Dxyn - DRW Vx, Vy, nibble: Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
    fn op_dxyn(&mut self) {
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
    fn op_ex9e(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let vx_idx = Vx as usize; 

        let key = self.registers[vx_idx];

        let keypad: Option<u8> = Some(self.keypad[key as usize]);

        if keypad.is_some() {
            self.pc += 2;
        }
    }

    // ExA1 - SKNP Vx: Skip next instruction if key with the value of Vx is not pressed
    fn op_exa1(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let vx_idx = Vx as usize; 

        let key = self.registers[vx_idx];

        let keypad: Option<u8> = Some(self.keypad[key as usize]);
        
        if keypad.is_none() {
            self.pc += 2;
        }
    }

    // Fx07 - LD Vx, DT: Set Vx = delay timer value.
    fn op_fx07(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let vx_idx = Vx as usize; 

        self.registers[vx_idx] = self.delay_timer;
    }

    // Fx0A - LD Vx, K: Wait for a key press, store the value of the key in Vx.
    fn op_fx0a(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let vx_idx = Vx as usize; 

        let mut keypad: [Option<u8>; 16] = [Some(0); 16];

        for i in 0..16 {
            keypad[i as usize] = Some(self.keypad[i as usize]);
        }

        if keypad[0].is_some() {
            self.registers[vx_idx] = 0;
        } else if keypad[1].is_some() {
            self.registers[vx_idx] = 1;
        } else if keypad[2].is_some() {
            self.registers[vx_idx] = 2;
        } else if keypad[3].is_some() {
            self.registers[vx_idx] = 3;
        } else if keypad[4].is_some() {
            self.registers[vx_idx] = 4;
        } else if keypad[5].is_some() {
            self.registers[vx_idx] = 5;
        } else if keypad[6].is_some() {
            self.registers[vx_idx] = 6;
        } else if keypad[7].is_some() {
            self.registers[vx_idx] = 7;
        } else if keypad[8].is_some() {
            self.registers[vx_idx] = 8;
        } else if keypad[9].is_some() {
            self.registers[vx_idx] = 9;
        } else if keypad[10].is_some() {
            self.registers[vx_idx] = 10;
        } else if keypad[11].is_some() {
            self.registers[vx_idx] = 11;
        } else if keypad[12].is_some() {
            self.registers[vx_idx] = 12;
        } else if keypad[13].is_some() {
            self.registers[vx_idx] = 13;
        } else if keypad[14].is_some() {
            self.registers[vx_idx] = 14;
        } else if keypad[15].is_some() {
            self.registers[vx_idx] = 15;
        } else {
            self.pc -= 2;
        }
    }

    // Fx15 - LD DT, Vx: Set delay timer = Vx
    fn op_fx15(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let vx_idx = Vx as usize;

        self.delay_timer = self.registers[vx_idx];
    }

    // Fx18 - LD ST, Vx: Set sound timer = Vx
    fn op_fx18(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let vx_idx = Vx as usize;

        self.sound_timer = self.registers[vx_idx];
    }

    // Fx1E - ADD I, Vx: Set I = I + Vx
    fn op_fx1e(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let vx_idx = Vx as usize;

        self.index += (self.registers[vx_idx]) as u16;
    }

    // Fx29 - LD F, Vx: Set I = location of sprite for digit Vx
    fn op_fx29(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let vx_idx = Vx as usize;
        let digit = self.registers[vx_idx];

        self.index = (FONTSET_START_ADDRESS + (5 * digit)) as u16;
    }

    // Fx33 - LD B, Vx: Store BCD representation of Vx in memory locations I, I+1, and I+2
    fn op_fx33(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;
        let vx_idx = Vx as usize;
        let mut value = self.registers[vx_idx];

        // Ones place
        self.memory[(self.index + 2) as usize] = value % 10;
        value /= 10;

        // Tens place
        self.memory[(self.index + 1) as usize] = value % 10;
        value /= 10;

        // Hundreds Place
        self.memory[self.index as usize] = value % 10;
    }

    // Fx55 - LD [I], Vx: Store registers V0 through Vx in memory starting at location I
    fn op_fx55(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;

        for i in 0..Vx {
            self.memory[(self.index + i as u16) as usize] = self.registers[i as usize];
        }
    }

    // Fx65 - LD Vx, [I]: Read registers V0 through Vx from memory starting at location I
    fn op_fx65(&mut self) {
        let Vx = ((self.opcode & 0x0F00) >> 8) as u8;

        for i in 0..Vx {
            self.registers[i as usize] = self.memory[(self.index + i as u16) as usize];
        }
    }

    // NULL : function that does nothing, but will be the default function called if a proper function pointer is not set
    fn op_null(&mut self) {
        
    }
}

impl Chip8 {
    fn cycle(&mut self) {

        // Fetch
        let opcode = (self.memory[self.pc as usize] << 8) | self.memory[(self.pc+1) as usize];

        // Increment program counter 
        self.pc += 2;

        // Decode and Execute
        match opcode {
            0x0 => {
                match opcode & 0x000F {
                    0x0 => self.op_00e0(),
                    0xE => self.op_00ee(),
                    _ => self.op_null(),
                }
            },
            0x1 => self.op_1nnn(),
            0x2 => self.op_2nnn(),
            0x3 => self.op_3xkk(),
            0x4 => self.op_4xkk(),
            0x5 => self.op_5xy0(),
            0x6 => self.op_6xkk(),
            0x7 => self.op_7xkk(),
            0x8 => {
                match opcode & 0x000F  {
                    0x0 => self.op_8xy0(),
                    0x1 => self.op_8xy1(),
                    0x2 => self.op_8xy2(),
                    0x3 => self.op_8xy3(),
                    0x4 => self.op_8xy4(),
                    0x5 => self.op_8xy5(),
                    0x6 => self.op_8xy6(),
                    0x7 => self.op_8xy7(),
                    0xE => self.op_8xye(),
                    _ => self.op_null(),
                }
            },
            0x9 => self.op_9xy0(),
            0xA => self.op_annn(),
            0xB => self.op_bnnnn(),
            0xC => self.op_cxkk(),
            0xD => self.op_dxyn(),
            0xE => {
                match opcode & 0x000F {
                    0x1 => self.op_exa1(),
                    0xE => self.op_ex9e(),
                    _ => self.op_null(),
                }
            },
            0xF => {
                match opcode & 0x00FF {
                    0x07 => self.op_fx07(),
                    0x0A => self.op_fx0a(),
                    0x15 => self.op_fx15(),
                    0x18 => self.op_fx18(),
                    0x1E => self.op_fx1e(),
                    0x29 => self.op_fx29(),
                    0x33 => self.op_fx33(),
                    0x55 => self.op_fx55(),
                    0x65 => self.op_fx65(),
                    _ => self.op_null(),
                }
            },
            _ => self.op_null()
        }

        // Decrement the delay timer if it's been set
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        // Decrement the sound timer if it's been set
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}

struct Platform<'a> {
    canvas: Canvas<Window>,
    texture: Texture<'a>,
}

impl<'a> Platform<'a> {
    fn platform(title: &str, window_width: u32, window_height: u32, texture_width: u32, texture_height: u32){
        let sdl_context = sdl2::init().unwrap();

        let window = sdl_context
            .video()
            .unwrap()
            .window(title, window_width, window_height)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas()
            .accelerated() 
            .build()
            .unwrap();

            let texture_creator =  canvas.texture_creator();

            let texture = texture_creator.create_texture_target(PixelFormatEnum::RGBA8888, texture_width, texture_height);
    }

    fn update(canvas: &mut Canvas<Window>, texture: &mut Texture, buffer: &[u8], pitch: usize) {

        texture.update(None, buffer, pitch).expect("Failed to update texture");

        canvas.clear();

        canvas.copy(texture, None, None).expect("Failed to copy texture to renderer");

        canvas.present();
    }

    fn process_input(mut keys: [u8; 16]) -> bool {
        let sdl_context = sdl2::init().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut quit = false;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    quit = true;
                }
                Event::KeyDown { keycode: Some(key), .. } => {
                    match key {
                        Keycode::Escape => {
                            quit = true;
                        }
                        Keycode::X => keys[0] = 1,
                        Keycode::Num1 => keys[1] = 1,
                        Keycode::Num2 => keys[2] = 1,
                        Keycode::Num3 => keys[3] = 1,
                        Keycode::Q => keys[4] = 1,
                        Keycode::W => keys[5] = 1,
                        Keycode::E => keys[6] = 1,
                        Keycode::A => keys[7] = 1,
                        Keycode::S => keys[8] = 1,
                        Keycode::D => keys[9] = 1,
                        Keycode::Z => keys[0xA] = 1,
                        Keycode::C => keys[0xB] = 1,
                        Keycode::Num4 => keys[0xC] = 1,
                        Keycode::R => keys[0xD] = 1,
                        Keycode::F => keys[0xE] = 1,
                        Keycode::V => keys[0xF] = 1,
                        _ => {}
                    }
                }
                Event::KeyUp { keycode: Some(key), .. } => {
                    match key {
                        Keycode::X => keys[0] = 0,
                        Keycode::Num1 => keys[1] = 0,
                        Keycode::Num2 => keys[2] = 0,
                        Keycode::Num3 => keys[3] = 0,
                        Keycode::Q => keys[4] = 0,
                        Keycode::W => keys[5] = 0,
                        Keycode::E => keys[6] = 0,
                        Keycode::A => keys[7] = 0,
                        Keycode::S => keys[8] = 0,
                        Keycode::D => keys[9] = 0,
                        Keycode::Z => keys[0xA] = 0,
                        Keycode::C => keys[0xB] = 0,
                        Keycode::Num4 => keys[0xC] = 0,
                        Keycode::R => keys[0xD] = 0,
                        Keycode::F => keys[0xE] = 0,
                        Keycode::V => keys[0xF] = 0,
                        _ => {}
                    }
                }
                _ => {}    
            }
        }

        quit
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage: {} <Scale> <Delay> <ROM>\n", args[0]);
        process::exit(1);
    }

    let video_scale: = args[1].parse::<i32>;

}
