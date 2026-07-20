#![allow(dead_code)]

use std::usize;

use rand::random;

// Display contants used by the frontend
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096; // 4KB RAM allocated for the chip8 spec
const NUM_REGS: usize = 16; // Number of V registers
const STACK_SIZE: usize = 16; // Stack used for entering/exiting a subroutine
const NUM_KEYS: usize = 16; // 16 keyboard keys allocated for chip8

const START_ADDR: u16 = 0x200; // The start address of the emulator in memory

const FONTSET_SIZE: usize = 80; // 80 elements in array (16 characters x 5 bytes per character)

// A character is made of up a 8 pixels x 5 bytes grid in binary (converted to hex)
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

// main fetch-decode-execute struct for the emulator
// 1. Fetch the value from our game (loaded into RAM) at the memory address stored in our Program Counter.
// 2. Decode this instruction.
// 3. Execute, which will possibly involve modifying our CPU registers or RAM.
// 4. Move the PC to the next instruction and repeat.
pub struct Emulator {
    pc: u16,                                      // Program counter
    ram: [u8; RAM_SIZE],                          // Amount of ram allocated to the emulator
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], // 2048 black and white pixels
    v_reg: [u8; NUM_REGS],                        // Number of V registers allocated
    i_reg: u16,                                   // Register used to index into RAM for read/write
    sp: u16,                                      // Stack pointer
    stack: [u16; STACK_SIZE],                     // The stack
    keys: [bool; NUM_KEYS],                       // Array for holding which keys are pressed
    dt: u8,                                       // Delay timer value
    st: u8,                                       // Sound timer value
}

impl Emulator {
    // Constructor to initialize a new Emulator
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        // copy the FONTSET into the first 80 addresses in memory (0x000 to 0x04F)
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    // Function to reset the Emulator to default values
    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    // Executes the fetch-decode-execute sequence for the emulator
    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();

        // Decode & Execute
        self.execute(op);
    }

    // Pass a pointer to the screen buffer to the frontend
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    // Interface to the frontend to handle keypresses. Takes the index of the key pressed and sets
    // the value
    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    // Load game code from a file into RAM beginning at 0x200
    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = start + data.len();

        self.ram[start..end].copy_from_slice(data);
    }

    // Add a new element to the stack
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val; // Rust indexing requires unsigned int's
        self.sp += 1;
    }

    // Remove the last element from the stack
    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    // Fetch the 16-bit op code stored at the Program Counter
    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;

        // Left shift the higher_byte by a byte to make room for the lower_byte
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;

        op
    }

    // Decode and execute an op code
    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // NOP
            // Do nothing, progress to next opcode
            (0, 0, 0, 0) => return,

            // CLS
            // Clear the screen
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }

            // RET
            // Return from subroutine
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }

            // JMP NNN
            // Jump to address 0xNNN
            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }

            // CALL NNN
            // Enter subroutine at 0xNNN, adding current PC onto stack to return to
            (2, _, _, _) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }

            // SKIP VX == 0xNN
            // If the value in the V register == 0xNN, skip forward and increment PC
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }

            // SKIP VX != 0xNN
            // If the value in the V register != 0xNN, skip forward and increment PC
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }

            // SKIP VX == VY
            // If the value X in V register == the value Y in V register, skip forward and increment
            // PC
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // VX == 0xNN
            // Set the V Register specified by the second digit to the value given.
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            }

            // VX += 0xNN
            // Add the given value to the VX register
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn); // Using wrapping add so we do no
                // panic on overflow
            }

            // VX = VY
            // Move the value in the VY register into the VX register
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            }

            // VX |= VY
            // Bitwise OR the value in the VY register into the VX register
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            }

            // VX &= VY
            // Bitwise AND the value in the VY register into the VX register
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            }

            // VX ^= VY
            // Bitwise XOR the value in the VY register into the VX register
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            }

            // VX += VY
            // Add the value in the VY register to the VX register
            // Uses VF flag register to track if overflow caused carry
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            // VX -= VY
            // Subtract the value in the VY register from the VX register
            // Uses VF flag register to track if underflow caused borrow
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            // VX >>= 1
            // Perform a single right shift on the value in VX, storing the dropped value in VF
            (8, _, _, 6) => {
                let x = digit2 as usize;
                let lsb = self.v_reg[x] & 1;

                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            }

            // VX = VY - VX
            // Subtract the value in the VX register from the value in the VY register and store in
            // VX. Use VF flag to track if underflow caused borrow
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            // VX <<= 1
            // Perform a single left shift on the value in VX, storing the dropped value in VF
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;

                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            }

            // SKIP VX != VY
            // If the value in the VX register != the value VY register, skip forward and increment
            // PC
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // I == 0xNNN
            // Set the I register to the value in 0xNNN
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            }

            // JMP V0 + 0xNNN
            // Move the PC to the sum of the value of V0 + 0xNNN
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }

            // VX = rand() & 0xNN
            // Random number generator. the random number gets AND'd to the lower 8-bits of the opcode
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();

                self.v_reg[x] = rng & nn;
            }

            // DRAW
            // Draw sprite at (VX, VY)
            (0xD, _, _, _) => {
                // Get the (x, y) coords for our sprite
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;

                // The last digit determines how many rows high our sprite is
                let num_rows = digit4;

                // Keep track if any pixels were flipped
                let mut flipped = false;

                // Iterate over each row of the sprite
                for y_line in 0..num_rows {
                    // Determine which memory address our rows data is stored in
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    // Iterate over each column in our row
                    for x_line in 0..8 {
                        // Use a mask to fetch the current pixels bit. Only flip if a 1
                        // This performs a right shift of the current x_line with 0x80 and then
                        // AND's it to the current pixel byte
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // Sprites should wrap around the screen, so apply modulo
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            // Get the pixels index for the 1D screen array
                            let idx = x + SCREEN_WIDTH * y;

                            // Check if we are going to flip the pixel and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                // If we flipped the pixel, set VF to 1, else set to 0
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            }

            // SKIP KEY PRESS
            // This instruction checks if the index stored in VX is pressed and if so, skips the
            // next instruction
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];

                let key = self.keys[vx as usize];

                if key {
                    self.pc += 2;
                }
            }

            // SKIP KEY RELEASE
            // This instruction checks if the index stored in VX is not pressed and if so, skips the
            // next instruction
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];

                let key = self.keys[vx as usize];

                if !key {
                    self.pc += 2;
                }
            }

            // VX = Delay Timer
            // Stores the current delay timer value in VX
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            }

            // WAIT KEY
            // Blocking operation. Loops endlessly until an element in the keys array returns true
            // and stores in VX. If multiple keys are pressed, it returns the lower indexed one
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;

                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                // The reason we reset opcode here and call the fetch function again is to allow new
                // keypresses to be registered. If we were stuck in the loop, the keypress opcodes
                // would never be fetched
                if !pressed {
                    // Redo opcode
                    self.pc -= 2;
                }
            }

            // Delay Timer = VX
            // Stores the current value in VX in DT
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            }

            // Sound Timer = VX
            // Stores the current value in VX in ST
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            }

            // I += VX
            // This takes the value stored in VX and adds it to the current value in I
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;

                self.i_reg = self.i_reg.wrapping_add(vx);
            }

            // I = FONT
            // Store the address of the character to print on screen into the I register
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;

                self.i_reg = c * 5;
            }

            // BCD
            // Stores the BCD representation of a value in VX in the I register
            // Converting to float is less efficient but easy to read
            // TODO: Optimize this
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;

                // Fetch the hundreds digit (Divide by 100 and drop decimal)
                let hundreds = (vx / 100.0).floor() as u8;

                // Fetch the tens digit (Divide by 10, dropping the ones digit and the decimal)
                let tens = ((vx / 10.0) % 10.0).floor() as u8;

                // Fetch the ones digit (Drop the hundreds and tens digits)
                let ones = (vx % 10.0).floor() as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }

            // STORE V0 - VX
            // Stores the V0 - VX registers into the I register
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;

                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            }

            // LOAD V0 - VX
            // Load the V0 - VX registers with RAM values in the I register
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;

                for idx in 0..x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented op code; {}", op),
        }
    }

    // Decrement timers every frame. When a time hits zero it does not automatically reset. The game
    // must reset them.
    // If the sound timer is set to 1, emit a beep
    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // TODO: Implement a way to create a beep
            }

            self.st -= 1;
        }
    }
}
