use rand::random;

// const defns
// backend only
const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const NUM_KEYS: usize = 16;
const STACK_SIZE: usize = 16;
const START_ADDR: u16 = 0x200;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

// frontend
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub struct Emu{
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS], // general purposed regs
    i_reg: u16, // for indexing into RAM for reads and writes
    sp: u16, // stack pointer
    keys: [bool; NUM_KEYS],
    stack: [u16; STACK_SIZE],
    dt: u8, // delay timer
    st: u8, // sound timer

}

impl Emu {
    // constructor
    pub fn new() -> Self{
        let mut new_emu = Self { 
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT], 
            v_reg: [0; NUM_REGS], 
            i_reg: 0, 
            sp: 0, 
            keys: [false; NUM_KEYS],
            stack: [0; STACK_SIZE],
            dt: 0, 
            st: 0 
        };   // self
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }   // new

    // reset
    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.sp = 0;
        self.keys = [false; NUM_KEYS];
        self.stack = [0; STACK_SIZE];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn push(&mut self, val: u16) {
        assert_ne!(self.sp, STACK_SIZE as u16);
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    pub fn pop(&mut self) -> u16 {
        assert_ne!(self.sp, 0);
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    // share display buff
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    // press key
    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    // load file
    pub fn load(&mut self, data: &[u8]){
        let start = START_ADDR as usize;
        let end = (start + data.len()) as usize;
        self.ram[start..end].copy_from_slice(data);
    }

    /*
    how cpu process each instruction
    1. Fetch the value from our game (loaded into RAM) at the memory address stored in our Program Counter.
    2. Decode this instruction.
    3. Execute, which will possibly involve modifying our CPU registers or RAM.
    4. Move the PC to the next instruction and repeat.
    */
    pub fn tick(&mut self){
        // fetch
        let op = self.fetch();
        // decode
        // execute
        self.execute(op);
        
    }

    fn fetch(&mut self) ->u16{
        let high = self.ram[self.pc as usize] as u16;
        let low = self.ram[(self.pc + 1) as usize] as u16;
        let op = (high << 8) | low;
        self.pc += 2;
        op
    }
    // execute
    fn execute(&mut self, op:u16){
        let digit1 = (op & 0xF000) >> 12;
        let digit2: u16 = (op & 0x0F00) >> 8;
        let digit3: u16 = (op & 0x00F0) >> 4;
        let digit4: u16 = op & 0x000F;
        let x = digit2 as usize;
        let y = digit3 as usize;
        //println!("ticking {} {} {} {}", digit1, digit2, digit3, digit4);
        match (digit1, digit2, digit3, digit4) {
            // NOOP
            (0,0,0,0) => return,
            // CLS (clear screen)
            (0,0,0xE,0) => {
                self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH]
            },
            // RET (return from subroutine)
            (0,0,0xE, 0xE) => {
                let ret = self.pop();
                self.pc = ret;
            },
            // 1NNN - Jump
            (1,_,_,_) => {
                let addr = op & 0xFFF;
                self.pc = addr;
            },
            // 2NNN - Call Subroutine
            (2,_,_,_) => {
                let addr = op & 0xFFF;
                self.push(self.pc);
                self.pc = addr;
            },
            // 3XNN - Skip next if VX == NN
            (3,_,_,_) => {
                let nn = op & 0xFF;
                if self.v_reg[x] == nn as u8{
                    self.pc += 2;
                }
            },
            // 4XNN - Skip next if VX != NN
            (4,_,_,_) => {
                let nn = op & 0xFF;
                if self.v_reg[x] != nn as u8{
                    self.pc += 2;
                }
            },
            // 5XY0 - Skip next if VX == VY
            (5,_,_,0) => {
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            },
            // 6XNN - VX = NN
            (6,_,_,_) => {
                let nn = op & 0xFF;
                self.v_reg[x] = nn as u8;
            },
            // 7XNN - VX += NN
            (7,_,_,_) => {
                let nn = op & 0xFF;
                self.v_reg[x] =  self.v_reg[x].wrapping_add(nn as u8);
            },
            // 8XY0 - VX = VY
            (8,_,_,0) => {
                self.v_reg[x] = self.v_reg[y];
            },
            // 8XY1, 8XY2, 8XY3 - Bitwise operations
            (8,_,_,1) => {
                self.v_reg[x] |= self.v_reg[y];
            },
            (8,_,_,2) => {
                self.v_reg[x] &= self.v_reg[y];
            },
            (8,_,_,3) => {
                self.v_reg[x] ^= self.v_reg[y];
            },
            // 8XY4 - VX += VY
            (8,_,_,4) => {
                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry {1} else {0};
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // 8XY5 - VX -= VY
            (8,_,_,5) => {
                let (new_vx, carry) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if carry {0} else {1};
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // 8XY6 - VX »= 1
            (8, _, _, 6) =>{
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            },
            // 8XY7 - VX = VY - VX
            (8,_,_,7) => {
                let (new_vx, carry) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if carry {0} else {1};
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // 8XY8 - VX <<= 1
            (8, _, _, 8) =>{
                let msb = self.v_reg[x] & 0xFF;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            },
            // 9XY0 - Skip if VX != VY
            (9, _, _, 0) =>{
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            },
            // ANNN - I = NNN
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            },
            // BNNN - Jump to V0 + NNN
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = self.v_reg[0] as u16 + nnn;
            },
            // CXNN - VX = rand() & NN
            (0xC, _, _, _) => {
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = nn & rng;
            },
            // DRAW
            (0xD, _, _, _) => {
                // Get the (x, y) coords for our sprite
                let x_coord = self.v_reg[x] as u16;
                let y_coord = self.v_reg[y] as u16;

                // The last digit determines how many rows high our sprite is
                let num_rows = digit4;

                // keep track if any pixel is flipped
                let mut flipped = false;

                // iterate over each row of sprite
                for y_line in 0..num_rows{

                    // Determine which memory address our row's data is stored
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    // Iterate over each column in our row
                    for x_line in 0..8{
                        // Use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b10000000 >> x_line)) != 0 {
                            // Sprites should wrap around screen, so apply modulo
                            let x_pos = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y_pos = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                            // Get our pixel's index for our 1D screen array
                            let idx = x_pos + SCREEN_WIDTH * y_pos;
                            // Check if we're about to flip the pixel and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }   // if
                    }   // for
                }   // for

                if flipped {
                    self.v_reg[0xF] = 1;
                }   else {
                    self.v_reg[0xF] = 0;
                }
            },
            // EX9E - Skip if Key Pressed
            (0xE, _, 9, 0xE) => {
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            },
            // EXA1 - Skip if Key Not Pressed
            (0xE, _, 0xA, 1) => {
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            },
            // FX07 - VX = DT
            (0xF, _, 0, 7) => {
                self.v_reg[x] = self.dt;
            },
            // FX0A - Wait for Key Press
            (0xF, _, 0, 0xA) => {
                let mut pressed = false;
                for i in 0..self.keys.len(){
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }   // if
                }   // for
                if !pressed {
                    self.pc -= 2;
                }
            },
            // FX15 - DT = VX
            (0xF, _, 1, 5) => {
                self.dt = self.v_reg[x];
            },
            // FX18 - ST = VX
            (0xF, _, 1, 8) => {
                self.st = self.v_reg[x];
            },
            // FX1E - I += VX
            (0xF, _, 1, 0xE) => {
                self.i_reg = self.i_reg.wrapping_add(self.v_reg[x] as u16);
            },
            // FX29 - Set I to Font Address
            (0xF, _, 2, 9) => {
                self.i_reg = (self.v_reg[x] * 5) as u16;
            },
            // FX33 - I = BCD of VX
            (0xF, _, 3, 3) => {
                let vx = (self.v_reg[x]) as f32;
                self.ram[self.i_reg as usize] = (vx / 100.0).floor() as u8;
                self.ram[(self.i_reg + 1) as usize] = ((vx / 10.0) % 10.0).floor() as u8;
                self.ram[(self.i_reg + 2) as usize] = (vx  % 10.0).floor() as u8;
            },
            // FX55 - Store V0 - VX into I
            (0xF, _, 5, 5) => {
                for idx in 0..x{
                    self.ram[self.i_reg as usize + idx] = self.v_reg[idx];
                }
            },
            // FX65 - Store I into V0 - VX
            (0xF, _, 6, 5) => {
                for idx in 0..x{
                    self.v_reg[idx] = self.ram[self.i_reg as usize + idx];
                }
            },
            (_,_,_,_) => unimplemented!("Unimplemented op code {}", op),
        }
    }

    // timer tick at each frame
    pub fn timer_tick(&mut self){
        if self.dt > 0 {
            self.dt -= 1;
        }   
        if self.st > 0 {
            if self.st == 1{
                // beep
            }
            self.st -= 1;  
        }  
    }   // timer tick



}