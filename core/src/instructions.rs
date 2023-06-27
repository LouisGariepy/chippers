pub fn decode(instruction: u16) -> Instruction {
    use Instruction::*;

    let a = (instruction & 0xF000) >> 12;
    let b = (instruction & 0x0F00) >> 8;
    let c = (instruction & 0x00F0) >> 4;
    let d = instruction & 0x000F;

    match (a, b, c, d) {
        (0x0, 0x0, 0xE, 0x0) => ClearScreen,
        (0x0, 0x0, 0xE, 0xE) => Return,
        (0x0, _, _, _) => MachineRoutine {
            address: instruction & 0x0FFF,
        },
        (0x1, _, _, _) => Jump {
            address: instruction & 0x0FFF,
        },
        (0x2, _, _, _) => Call {
            address: instruction & 0x0FFF,
        },
        (0x3, _, _, _) => SkipEqualByte {
            register: b as usize,
            byte: (instruction & 0x00FF) as u8,
        },
        (0x4, _, _, _) => SkipNotEqualByte {
            register: b as usize,
            byte: (instruction & 0x00FF) as u8,
        },
        (0x5, _, _, 0x0) => SkipEqualVariable {
            register_x: b as usize,
            register_y: c as usize,
        },
        (0x6, _, _, _) => SetWithByte {
            register: b as usize,
            byte: (instruction & 0x00FF) as u8,
        },
        (0x7, _, _, _) => AddWithByte {
            register: b as usize,
            byte: (instruction & 0x00FF) as u8,
        },
        (0x8, _, _, 0x0) => SetWithVariable {
            register_x: b as usize,
            register_y: c as usize,
        },
        (0x8, _, _, 0x1) => Or {
            register_x: b as usize,
            register_y: c as usize,
        },
        (0x8, _, _, 0x2) => And {
            register_x: b as usize,
            register_y: c as usize,
        },
        (0x8, _, _, 0x3) => Xor {
            register_x: b as usize,
            register_y: c as usize,
        },
        (0x8, _, _, 0x4) => AddWithVariable {
            register_x: b as usize,
            register_y: c as usize,
        },
        (0x8, _, _, 0x5) => SubWithVariable {
            register_x: b as usize,
            register_y: c as usize,
        },
        (0x8, _, _, 0x6) => ShiftRight {
            register_x: b as usize,
            #[cfg(not(feature = "modern"))]
            register_y: c as usize,
        },
        (0x8, _, _, 0x7) => SubWithVariableNot {
            register_x: b as usize,
            register_y: c as usize,
        },
        (0x8, _, _, 0xE) => ShiftLeft {
            register_x: b as usize,
            #[cfg(not(feature = "modern"))]
            register_y: c as usize,
        },
        (0x9, _, _, 0x0) => SkipNotEqualVariable {
            register_x: b as usize,
            register_y: c as usize,
        },
        (0xA, _, _, _) => SetIndexWithAddress {
            address: instruction & 0x0FFF,
        },
        (0xB, _, _, _) => JumpOffset {
            base_address: instruction & 0x0FFF,
            #[cfg(feature = "modern")]
            register: b as usize,
        },
        (0xC, _, _, _) => RandomAnd {
            register: b as usize,
            byte: (instruction & 0x00FF) as u8,
        },
        (0xD, _, _, _) => Draw {
            register_x: b as usize,
            register_y: c as usize,
            n: d as u8,
        },
        (0xE, _, 0x9, 0xE) => SkipKey {
            register: b as usize,
        },
        (0xE, _, 0xA, 0x1) => SkipNotKey {
            register: b as usize,
        },
        (0xF, _, 0x0, 0x7) => SetVariableWithDelayTimer {
            register: b as usize,
        },
        (0xF, _, 0x0, 0xA) => WaitForKey {
            register: b as usize,
        },
        (0xF, _, 0x1, 0x5) => SetDelayTimer {
            register: b as usize,
        },
        (0xF, _, 0x1, 0x8) => SetSoundTimer {
            register: b as usize,
        },
        (0xF, _, 0x1, 0xE) => AddIndexWithVariable {
            register: b as usize,
        },
        (0xF, _, 0x2, 0x9) => SetIndexWithSpriteAddress {
            register: b as usize,
        },
        (0xF, _, 0x3, 0x3) => StoreDecimalConversion {
            register: b as usize,
        },
        (0xF, _, 0x5, 0x5) => StoreRegisters {
            up_to_register: b as usize,
        },
        (0xF, _, 0x6, 0x5) => LoadIntoRegisters {
            up_to_register: b as usize,
        },
        _ => unreachable!("invalid opcode ({a:01X}{b:01X}{c:01X}{d:01X})"),
    }
}

#[derive(Debug)]
pub enum Instruction {
    // Routines
    /// 2nnn
    Call { address: u16 },
    /// 00EE
    Return,

    // Control flow
    /// 1nnn
    Jump { address: u16 },
    /// Bnnn
    JumpOffset {
        base_address: u16,
        #[cfg(feature = "modern")]
        register: usize,
    },
    /// 3xkk
    SkipEqualByte { register: usize, byte: u8 },
    /// 4xkk
    SkipNotEqualByte { register: usize, byte: u8 },
    /// 5xy0
    SkipEqualVariable {
        register_x: usize,
        register_y: usize,
    },
    /// 9xy0
    SkipNotEqualVariable {
        register_x: usize,
        register_y: usize,
    },
    /// Ex9E
    SkipKey { register: usize },
    /// ExA1
    SkipNotKey { register: usize },

    // Register setters
    /// 6xkk
    SetWithByte { register: usize, byte: u8 },
    /// 8xy0
    SetWithVariable {
        register_x: usize,
        register_y: usize,
    },
    /// Annn
    SetIndexWithAddress { address: u16 },
    /// Fx29
    SetIndexWithSpriteAddress { register: usize },

    // Arithmetic operations
    /// 7xkk
    AddWithByte { register: usize, byte: u8 },
    /// 8xy4
    AddWithVariable {
        register_x: usize,
        register_y: usize,
    },
    /// Fx1E
    AddIndexWithVariable { register: usize },
    /// 8xy5
    SubWithVariable {
        register_x: usize,
        register_y: usize,
    },
    /// 8xy7
    SubWithVariableNot {
        register_x: usize,
        register_y: usize,
    },
    /// 8xy6
    ShiftRight {
        register_x: usize,
        #[cfg(not(feature = "modern"))]
        register_y: usize,
    },
    /// 8xyE
    ShiftLeft {
        register_x: usize,
        #[cfg(not(feature = "modern"))]
        register_y: usize,
    },

    // Logical operations
    /// 8xy1
    Or {
        register_x: usize,
        register_y: usize,
    },
    /// 8xy2
    And {
        register_x: usize,
        register_y: usize,
    },
    /// 8xy3
    Xor {
        register_x: usize,
        register_y: usize,
    },

    // Display
    /// 00E0
    ClearScreen,
    /// Dxyn
    Draw {
        register_x: usize,
        register_y: usize,
        n: u8,
    },

    // Timers
    /// Fx07
    SetVariableWithDelayTimer { register: usize },
    /// Fx15
    SetDelayTimer { register: usize },
    /// Fx18
    SetSoundTimer { register: usize },

    // RAM load and store
    /// Fx55
    StoreRegisters { up_to_register: usize },
    /// Fx65
    LoadIntoRegisters { up_to_register: usize },

    // Misc
    /// Fx33
    StoreDecimalConversion { register: usize },
    /// Fx0A
    WaitForKey { register: usize },
    /// Cxkk
    RandomAnd { register: usize, byte: u8 },

    // Defunct
    /// 0nnn
    MachineRoutine { address: u16 },
}
