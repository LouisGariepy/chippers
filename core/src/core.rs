use std::{
    fmt::Display,
    ops::{Index, IndexMut, Range},
};

pub struct Ram([u8; 4096]);

impl Ram {
    pub(crate) fn new() -> Self {
        let mut buffer = [0; 4096];

        // Initialize font data in RAM
        for (font_data, memory_cell) in FONT_DATA.into_iter().zip(buffer.iter_mut()) {
            *memory_cell = font_data
        }

        Self(buffer)
    }

    pub(crate) fn load_program(&mut self, program: &[u8]) {
        for (offset, byte) in program.iter().copied().enumerate() {
            self.0[0x200 + offset] = byte;
        }
    }
}

impl Index<Range<usize>> for Ram {
    type Output = [u8];

    fn index(&self, range: Range<usize>) -> &Self::Output {
        &self.0[range]
    }
}

impl Index<u16> for Ram {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u16> for Ram {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

pub struct VariableRegisters([u8; 16]);

impl VariableRegisters {
    pub(crate) fn new() -> Self {
        Self([0; 16])
    }

    pub(crate) fn set_vf(&mut self) {
        self.0[15] = 1;
    }

    pub(crate) fn set_vf_to(&mut self, val: u8) {
        self.0[15] = val;
    }

    pub(crate) fn clear_vf(&mut self) {
        self.0[15] = 0;
    }
}

impl Index<usize> for VariableRegisters {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for VariableRegisters {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

pub struct Stack(Vec<u16>);

impl Stack {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn push(&mut self, address: u16) {
        self.0.push(address)
    }

    pub(crate) fn pop(&mut self) -> u16 {
        self.0.pop().unwrap()
    }
}

pub struct Screen([bool; 32 * 64]);

impl Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", "-".repeat(129))?;
        for row in 0..32 {
            write!(f, "|")?;
            for pixel in 0..64 {
                let pixel_value = self.0[row * 64 + pixel];
                let pixel_display = if pixel_value { "██" } else { "  " };
                write!(f, "{pixel_display}")?;
            }
            writeln!(f, "|")?;
        }
        writeln!(f, "{}", "-".repeat(129))?;
        Ok(())
    }
}

impl Screen {
    pub(crate) fn new() -> Self {
        Self([false; 32 * 64])
    }

    pub(crate) fn clear(&mut self) {
        self.0 = [false; 32 * 64];
    }

    pub(crate) fn set_pixel(&mut self, x: u8, y: u8) -> bool {
        let index = (y as usize * 64) + x as usize;
        let collision = self.0[index];
        self.0[index] ^= true;
        collision
    }
}

pub struct Timer {
    pub value: u8,
    pub state: TimerState,
}

impl Timer {
    pub(crate) fn new() -> Self {
        Self {
            value: 0,
            state: TimerState::AboveZero,
        }
    }

    pub fn decrement(&mut self) {
        match self.value {
            0 => {}
            1 => {
                self.value = 0;
                self.state = TimerState::Zero
            }
            _ => {
                self.value -= 1;
            }
        }
    }

    pub fn reset(&mut self) {
        self.value = 60;
    }
}

pub enum TimerState {
    Zero,
    AboveZero,
}

#[rustfmt::skip]
pub const FONT_DATA: [u8; 80] = [
    // 0
    0b11110000, 
    0b10010000, 
    0b10010000, 
    0b10010000, 
    0b11110000,
    
    // 1
    0b00100000,
    0b01100000,
    0b00100000,
    0b00100000,
    0b01110000,
    
    // 2
    0b11110000,
    0b00010000,
    0b11110000,
    0b10000000,
    0b11110000,
    
    // 3
    0b11110000, 
    0b00010000, 
    0b11110000,
    0b00010000, 
    0b11110000, 
    
    // 4
    0b10010000, 
    0b10010000, 
    0b11110000, 
    0b00010000, 
    0b00010000, 
    
    // 5
    0b11110000, 
    0b10000000,
    0b11110000, 
    0b10010000, 
    0b11110000,

    // 6
    0b11110000,
    0b10000000,
    0b11110000,
    0b10010000,
    0b11110000,
    
    // 7
    0b11110000, 
    0b00010000, 
    0b00100000, 
    0b01000000,
    0b01000000, 
    
    // 8
    0b11110000,
    0b10010000, 
    0b11110000, 
    0b10010000, 
    0b11110000, 
    
    // 9
    0b11110000,
    0b10010000,
    0b11110000,
    0b00010000,
    0b11110000,

    // A
    0b11110000, 
    0b10010000, 
    0b11110000, 
    0b10010000, 
    0b10010000, 
    
    // B
    0b11100000,
    0b10010000,
    0b11100000,
    0b10010000,
    0b11100000, 
    
    // C
    0b11110000, 
    0b10000000, 
    0b10000000, 
    0b10000000, 
    0b11110000,
    
    // D
    0b11100000, 
    0b10010000, 
    0b10010000,
    0b10010000, 
    0b11100000, 
    
    // E
    0b11110000, 
    0b10000000, 
    0b11110000, 
    0b10000000, 
    0b11110000, 
    
    // F
    0b11110000, 
    0b10000000,
    0b11110000, 
    0b10000000, 
    0b10000000,
];
