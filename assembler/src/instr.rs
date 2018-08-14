//! Defines available instructions.

// TODO: remove once we use all of this stuff
#![allow(dead_code)]


/// Represents a full instruction in the source code, including arguments.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Instruction {
    // $0_
    Nop,

    // $1_ (data transfer)
    Ld { src: Arg },
    Ldi { v: Arg },
    St { dst: Arg },
    Sti { v: Arg },
    Mov { src: Arg, dst: Arg },

    // $2_ (control flow)
    Jmp { target: Arg },
    Jz { target: Arg },

    // $3_ (arithmetic)
    Add { src: Arg },
    Addi { v: Arg },
    Sub { src: Arg },
    Subi { v: Arg },
    Shr,
    Shl,
    And { src: Arg },
    Andi { v: Arg },

    // $4_
    Print { src: Arg },

    // $5_
    Stop,
}

impl Instruction {
    /// Returns the opcode of this instruction. This basically just removes
    /// information about the arguments.
    pub fn opcode(&self) -> Opcode {
        match *self {
            Instruction::Nop => Opcode::Nop,
            Instruction::Ld { .. } => Opcode::Ld,
            Instruction::Ldi { .. } => Opcode::Ldi,
            Instruction::St { .. } => Opcode::St,
            Instruction::Sti { .. } => Opcode::Sti,
            Instruction::Mov { .. } => Opcode::Mov,
            Instruction::Jmp { .. } => Opcode::Jmp,
            Instruction::Jz { .. } => Opcode::Jz,
            Instruction::Add { .. } => Opcode::Add,
            Instruction::Addi { .. } => Opcode::Addi,
            Instruction::Sub { .. } => Opcode::Sub,
            Instruction::Subi { .. } => Opcode::Subi,
            Instruction::Shr => Opcode::Shr,
            Instruction::Shl => Opcode::Shl,
            Instruction::And { .. } => Opcode::And,
            Instruction::Andi { .. } => Opcode::Andi,
            Instruction::Print { .. } => Opcode::Print,
            Instruction::Stop => Opcode::Stop,
        }
    }
}


/// An argument to an instruction in the source code.
#[derive(Debug, Clone)]
pub enum Arg {
    /// A value is directly specified
    Value(u8),

    /// A label is used and must be resolved to the actual value later
    Label(String),
}


/// Represents an instruction without the arguments.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Opcode {
    // $0_
    Nop,

    // $1_ (data transfer)
    Ld,
    Ldi,
    St,
    Sti,
    Mov,

    // $2_ (control flow)
    Jmp,
    Jz,

    // $3_ (arithmetic)
    Add,
    Addi,
    Sub,
    Subi,
    Shr,
    Shl,
    And,
    Andi,

    // $4_
    Print,

    // $5_
    Stop,
}

impl Opcode {
    /// Returns the byte of this opcode.
    pub fn to_byte(&self) -> u8 {
        use self::Opcode::*;

        match *self {
            Nop => 0x00,
            Ld => 0x10,
            Ldi => 0x11,
            St => 0x12,
            Sti => 0x13,
            Mov => 0x14,
            Jmp => 0x20,
            Jz => 0x21,
            Add => 0x30,
            Addi => 0x31,
            Sub => 0x32,
            Subi => 0x33,
            Shr => 0x34,
            Shl => 0x35,
            And => 0x36,
            Andi => 0x37,
            Print => 0x40,
            Stop => 0x50,
        }
    }

    // Returns the number of bytes this instruction (with its arguments) will
    // occupy.
    pub fn len(&self) -> u8 {
        use self::Opcode::*;

        match *self {
            Nop => 1,
            Ld => 2,
            Ldi => 2,
            St => 2,
            Sti => 3,
            Mov => 3,
            Jmp => 2,
            Jz => 2,
            Add => 2,
            Addi => 2,
            Sub => 2,
            Subi => 2,
            Shr => 1,
            Shl => 1,
            And => 2,
            Andi => 2,
            Print => 2,
            Stop => 1,
        }
    }
}
