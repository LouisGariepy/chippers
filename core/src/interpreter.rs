use rand::{rngs::OsRng, Rng};

use crate::{
    core::{Ram, Screen, Stack, Timer, VariableRegisters},
    instructions::{decode, Instruction},
};

#[derive(Clone, Copy)]
pub enum Key {
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
}

impl From<u8> for Key {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Key::Key0,
            0x1 => Key::Key1,
            0x2 => Key::Key2,
            0x3 => Key::Key3,
            0x4 => Key::Key4,
            0x5 => Key::Key5,
            0x6 => Key::Key6,
            0x7 => Key::Key7,
            0x8 => Key::Key8,
            0x9 => Key::Key9,
            0xA => Key::KeyA,
            0xB => Key::KeyB,
            0xC => Key::KeyC,
            0xD => Key::KeyD,
            0xE => Key::KeyE,
            0xF => Key::KeyF,
            _ => unreachable!(),
        }
    }
}

impl From<Key> for u8 {
    fn from(value: Key) -> Self {
        match value {
            Key::Key0 => 0x0,
            Key::Key1 => 0x1,
            Key::Key2 => 0x2,
            Key::Key3 => 0x3,
            Key::Key4 => 0x4,
            Key::Key5 => 0x5,
            Key::Key6 => 0x6,
            Key::Key7 => 0x7,
            Key::Key8 => 0x8,
            Key::Key9 => 0x9,
            Key::KeyA => 0xA,
            Key::KeyB => 0xB,
            Key::KeyC => 0xC,
            Key::KeyD => 0xD,
            Key::KeyE => 0xE,
            Key::KeyF => 0xF,
        }
    }
}

#[derive(Clone, Copy)]
pub enum KeyState {
    // Key is not pressed (including if it was just released)
    NotPressed,
    // Key is pressed (including if it was just pressed)
    Pressed,
    // Special state when a key was already pressed before waiting for input
    AlreadyPressed,
}

pub struct InputHandler {
    pub keys_state: [KeyState; 16],
    pub waiting: Option<usize>,
    pub pressed_and_released: Option<Key>,
}

pub struct Interpreter {
    pub ram: Ram,
    pub screen: Screen,
    pub variable_registers: VariableRegisters,
    pub index_register: u16,
    pub program_counter: u16,
    pub stack: Stack,
    pub delay_timer: Timer,
    pub sound_timer: Timer,
    pub input_handler: InputHandler,
}

impl Interpreter {
    pub fn new(program: &[u8]) -> Self {
        let mut ram = Ram::new();
        ram.load_program(program);

        Self {
            ram,
            variable_registers: VariableRegisters::new(),
            index_register: 0,
            program_counter: 0x200,
            stack: Stack::new(),
            screen: Screen::new(),
            delay_timer: Timer::new(),
            sound_timer: Timer::new(),
            input_handler: InputHandler {
                keys_state: [KeyState::NotPressed; 16],
                waiting: None,
                pressed_and_released: None,
            },
        }
    }

    fn draw(&mut self, register_x: usize, register_y: usize, n: u8) {
        // Fetch coordinates from registers Vx and Vy
        // Note that the coordinates refers to *bit* (pixel) position.
        let initial_x = self.variable_registers[register_x] & 63; // mod 64
        let mut y = self.variable_registers[register_y] & 31; // mod 32

        // VF will act as a collision detector for sprites.
        // We set it to no collision initially.
        self.variable_registers.clear_vf();

        // Draw each sprite line
        for sprite_offset in 0..n {
            // Get sprite line
            let sprite_address = self.index_register + sprite_offset as u16;
            let sprite_line = self.ram[sprite_address];
            let mut x = initial_x;

            // Draw sprite pixels
            for bit_pos in 0..8 {
                // Each digit represents a pixel of the sprite line
                let mask = 0b10000000 >> bit_pos;
                let bit_digit = sprite_line & mask;

                // If the the sprite pixel is on
                if bit_digit != 0 {
                    // Set pixel and detect collision
                    let collision = self.screen.set_pixel(x, y);
                    // If collision is detected, set VF.
                    if collision {
                        self.variable_registers.set_vf();
                    }
                }

                // If we've reached the horizontal end of the screen, break
                // otherwise increment x
                if x == 63 {
                    break;
                } else {
                    x += 1;
                }
            }

            // If we've reached the vertical end of the screen, break
            // otherwise increment y
            if y == 31 {
                break;
            } else {
                y += 1;
            }
        }
    }

    fn fetch_instruction(&mut self) -> u16 {
        // Fetch raw instruction bytes
        let raw_instruction = [
            self.ram[self.program_counter],
            self.ram[self.program_counter + 1],
        ];

        // Make 16 bit instruction out of raw instruction (note the big-endianness)
        let instruction = u16::from_be_bytes(raw_instruction);

        // Increment program counter
        self.program_counter += 2;

        instruction
    }

    pub fn step(&mut self) {
        if let Some(register) = self.input_handler.waiting {
            let Some(key) = self.input_handler.pressed_and_released else {
                return;
            };
            self.variable_registers[register] = key.into();
        }

        let instruction = self.fetch_instruction();
        let decoded_instruction = decode(instruction);

        self.execute(decoded_instruction);
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            // Subroutines
            Instruction::Call { address } => {
                self.stack.push(self.program_counter);
                self.program_counter = address;
            }
            Instruction::Return => self.program_counter = self.stack.pop(),

            // Control flow
            Instruction::Jump { address } => self.program_counter = address,
            Instruction::JumpOffset {
                base_address,
                #[cfg(feature = "modern")]
                register,
            } => {
                #[cfg(not(feature = "modern"))]
                let register = 0usize;

                self.program_counter = base_address + self.variable_registers[register] as u16
            }
            Instruction::SkipEqualByte { register, byte } => {
                if self.variable_registers[register] == byte {
                    self.program_counter += 2;
                }
            }
            Instruction::SkipNotEqualByte { register, byte } => {
                if self.variable_registers[register] != byte {
                    self.program_counter += 2;
                }
            }
            Instruction::SkipEqualVariable {
                register_x,
                register_y,
            } => {
                if self.variable_registers[register_x] == self.variable_registers[register_y] {
                    self.program_counter += 2;
                }
            }
            Instruction::SkipNotEqualVariable {
                register_x,
                register_y,
            } => {
                if self.variable_registers[register_x] != self.variable_registers[register_y] {
                    self.program_counter += 2;
                }
            }
            Instruction::SkipKey { register } => {
                let key_index = self.variable_registers[register] as usize;
                if let KeyState::Pressed = self.input_handler.keys_state[key_index] {
                    self.program_counter += 2;
                }
            }
            Instruction::SkipNotKey { register } => {
                let key_index = self.variable_registers[register] as usize;
                if !matches!(self.input_handler.keys_state[key_index], KeyState::Pressed) {
                    self.program_counter += 2;
                }
            }

            // Register setters
            Instruction::SetWithByte { register, byte } => self.variable_registers[register] = byte,
            Instruction::SetWithVariable {
                register_x,
                register_y,
            } => self.variable_registers[register_x] = self.variable_registers[register_y],
            Instruction::SetIndexWithAddress { address } => self.index_register = address,
            Instruction::SetIndexWithSpriteAddress { register } => {
                self.index_register = self.variable_registers[register] as u16 * 5;
            }

            // Arithmetic operations
            Instruction::AddWithByte { register, byte } => {
                self.variable_registers[register] =
                    self.variable_registers[register].wrapping_add(byte);
            }
            Instruction::AddWithVariable {
                register_x,
                register_y,
            } => {
                // Compute overflowing sum
                let (sum, overflow) = self.variable_registers[register_x]
                    .overflowing_add(self.variable_registers[register_y]);
                // Set register to sum
                self.variable_registers[register_x] = sum;
                // Set VF to overflow flag
                self.variable_registers.set_vf_to(overflow as u8);
            }
            Instruction::AddIndexWithVariable { register } => {
                self.index_register += self.variable_registers[register] as u16;
            }
            Instruction::SubWithVariable {
                register_x,
                register_y,
            } => {
                // Compute overflowing sum
                let (difference, overflow) = self.variable_registers[register_x]
                    .overflowing_sub(self.variable_registers[register_y]);
                // Set register to sum
                self.variable_registers[register_x] = difference;
                // Set VF to overflow flag
                self.variable_registers.set_vf_to(!overflow as u8);
            }
            Instruction::SubWithVariableNot {
                register_x,
                register_y,
            } => {
                // Compute overflowing sum
                let (difference, overflow) = self.variable_registers[register_y]
                    .overflowing_sub(self.variable_registers[register_x]);
                // Set register to sum
                self.variable_registers[register_x] = difference;
                // Set VF to overflow flag
                self.variable_registers.set_vf_to(!overflow as u8);
            }
            Instruction::ShiftRight {
                register_x,
                #[cfg(not(feature = "modern"))]
                register_y,
            } => {
                #[cfg(not(feature = "modern"))]
                {
                    // Set Vx to Vy
                    self.variable_registers[register_x] = self.variable_registers[register_y];
                }
                // Get the digit that will be shifted out
                let last_digit = self.variable_registers[register_x] & 0b00000001;
                // Shift Vx
                self.variable_registers[register_x] >>= 1;
                // Set VF to the shifted digit
                self.variable_registers.set_vf_to(last_digit);
            }
            Instruction::ShiftLeft {
                register_x,
                #[cfg(not(feature = "modern"))]
                register_y,
            } => {
                #[cfg(not(feature = "modern"))]
                {
                    // Set Vx to Vy
                    self.variable_registers[register_x] = self.variable_registers[register_y];
                }
                // Get the digit that will be shifted out
                let first_digit = (self.variable_registers[register_x] & 0b10000000) >> 7;
                // Shift Vx
                self.variable_registers[register_x] <<= 1;
                // Set VF to the shifted digit
                self.variable_registers.set_vf_to(first_digit);
            }

            // Logical operations
            Instruction::Or {
                register_x,
                register_y,
            } => {
                self.variable_registers[register_x] |= self.variable_registers[register_y];
            }
            Instruction::And {
                register_x,
                register_y,
            } => {
                self.variable_registers[register_x] &= self.variable_registers[register_y];
            }
            Instruction::Xor {
                register_x,
                register_y,
            } => {
                self.variable_registers[register_x] ^= self.variable_registers[register_y];
            }

            // Display
            Instruction::ClearScreen => self.screen.clear(),
            Instruction::Draw {
                register_x,
                register_y,
                n,
            } => self.draw(register_x, register_y, n),

            // Timers
            Instruction::SetVariableWithDelayTimer { register } => {
                self.variable_registers[register] = self.delay_timer.value;
            }
            Instruction::SetDelayTimer { register } => {
                self.delay_timer.value = self.variable_registers[register];
            }
            Instruction::SetSoundTimer { register } => {
                self.sound_timer.value = self.variable_registers[register];
            }

            // RAM load and store
            Instruction::StoreRegisters { up_to_register } => {
                #[cfg(feature = "modern")]
                let index_register = self.index_register;

                for register in 0..=up_to_register {
                    self.ram[self.index_register] = self.variable_registers[register];
                    self.index_register += 1;
                }

                #[cfg(feature = "modern")]
                {
                    self.index_register = index_register;
                }
            }
            Instruction::LoadIntoRegisters { up_to_register } => {
                #[cfg(feature = "modern")]
                let index_register = self.index_register;

                for register in 0..=up_to_register {
                    self.variable_registers[register] = self.ram[self.index_register];
                    self.index_register += 1;
                }

                #[cfg(feature = "modern")]
                {
                    self.index_register = index_register;
                }
            }

            // Misc
            Instruction::StoreDecimalConversion { register } => {
                let value = self.variable_registers[register];
                let hundreds = value / 100;
                let tens = (value - (hundreds * 100)) / 10;
                let ones = value - (hundreds * 100) - (tens * 10);

                self.ram[self.index_register] = hundreds;
                self.ram[self.index_register + 1] = tens;
                self.ram[self.index_register + 2] = ones;
            }
            Instruction::WaitForKey { register } => {
                self.input_handler.keys_state =
                    self.input_handler.keys_state.map(|state| match state {
                        KeyState::NotPressed => state,
                        KeyState::Pressed => KeyState::AlreadyPressed,
                        KeyState::AlreadyPressed => state,
                    });
                self.input_handler.waiting = Some(register);
            }
            Instruction::RandomAnd { register, byte } => {
                let mut rng = OsRng;
                let random_byte = rng.gen::<u8>();
                self.variable_registers[register] = random_byte & byte;
            }

            // Defunct
            Instruction::MachineRoutine { .. } => {}
        }
    }
}
