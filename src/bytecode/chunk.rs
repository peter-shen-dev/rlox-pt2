use std::io::Write;

use num_enum::{FromPrimitive, IntoPrimitive};

use crate::common::ui::Span;
use crate::value::Value;
use crate::{bytecode::interner::Interner, common::try_as::TryAs, value::function::ObjFunction};

#[derive(Debug, Eq, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum OpCode {
    // 0 follow bytes ====
    Return,
    Nil,
    True,
    False,
    // 1 follow bytes ====
    Constant, // 1: a constant index
    Call,
    // 2 follow bytes ====
    JumpRelIfFalse,
    JumpRelIfTrue,
    JumpRel,
    Loop,
    // variable-length
    Closure,
    // No follow bytes but data-dependent
    // Unary
    Negate,
    Not,
    Print,
    Pop,
    CloseUpvalue,
    GetGlobal,
    DefineGlobal,
    SetGlobal,
    GetLocal,
    SetLocal,
    GetUpvalue,
    SetUpvalue,
    // Binary
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    Greater,
    Less,
    #[num_enum(default)]
    Invalid,
}

#[derive(Default, Debug, Clone)]
pub struct Chunk {
    // INVARIANT: An OpCode must be followed by however many bytes are specified
    pub instructions: Vec<u8>,
    pub spans: Vec<Span>,
    // Owned by this
    constants: Vec<Value>,
    // Owned by this
    pub globals: Interner,
    pub native_globals: Vec<(u8, Value)>,
}

impl Drop for Chunk {
    fn drop(&mut self) {
        for constant in self
            .constants
            .iter()
            .chain(self.native_globals.iter().map(|(_, v)| v))
        {
            if let Value::Object(obj) = constant {
                unsafe {
                    // SAFETY: See safety invariant on constants
                    obj.free();
                }
            }
        }
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_native(&mut self, nameid: u8, value: Value) {
        self.native_globals.push((nameid, value));
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        let index = self.constants.len() - 1;
        index.try_into().expect("Too many constants")
    }

    pub fn get_constant(&self, index: u8) -> Value {
        self.constants[index as usize]
    }

    pub fn write_byte(&mut self, byte: impl Into<u8>, origin: Span) {
        self.instructions.push(byte.into());
        self.spans.push(origin);
    }

    pub fn disassemble(&self, name: &str, source: &str, mut stdout: impl Write) {
        writeln!(stdout, "==== {name} ====").unwrap();
        let mut i = 0;
        while i < self.instructions.len() {
            i = self.disassemble_instruction(i, source, &mut stdout);
        }
    }

    fn simple_instruction(name: &str, offset: &mut usize, mut stdout: impl Write) {
        writeln!(stdout, "{name}").unwrap();
        *offset += 1;
    }

    fn constant_instruction(&self, name: &str, offset: &mut usize, mut stdout: impl Write) {
        let index = self.instructions[*offset + 1];
        let value = self.constants[index as usize];
        writeln!(stdout, "{name:<16} {index:>4} '{value}'").unwrap();
        *offset += 2;
    }

    fn global_instruction(&self, name: &str, offset: &mut usize, mut stdout: impl Write) {
        let index = self.instructions[*offset + 1];
        let value = self.globals.get_name(index);
        writeln!(stdout, "{name:<16} {index:>4} '{value}'").unwrap();
        *offset += 2;
    }

    fn byte_instruction(&self, name: &str, offset: &mut usize, mut stdout: impl Write) {
        let value = self.instructions[*offset + 1];
        writeln!(stdout, "{name:<16} {value}").unwrap();
        *offset += 2;
    }

    fn jmp_instruction(&self, name: &str, offset: &mut usize, mut stdout: impl Write) {
        let value = &self.instructions[*offset + 1..][..2];
        let addr: u16 = bytemuck::pod_read_unaligned(value);
        writeln!(stdout, "{name:<16} {addr}").unwrap();
        *offset += 3;
    }

    fn closure(&self, offset: &mut usize, mut stdout: impl Write) {
        let value = self.instructions[*offset + 1];
        let fun: ObjFunction = self.constants[value as usize].try_as().unwrap();
        writeln!(stdout, "{:<16} {}", "CLOSURE", fun).unwrap();
        *offset += 2;
        for _ in 0..fun.upvalues {
            let local = if self.instructions[*offset] == 1 {
                "local"
            } else {
                "upvalue"
            };
            let index = self.instructions[*offset + 1];
            writeln!(
                stdout,
                "{:0>4}                               {} {}",
                *offset, local, index
            )
            .unwrap();
            *offset += 2;
        }
    }

    pub fn disassemble_instruction(
        &self,
        mut offset: usize,
        source: &str,
        mut stdout: impl Write,
    ) -> usize {
        write!(stdout, "{:0>4} ", offset).unwrap();
        if offset > 0 && self.spans[offset] == self.spans[offset - 1] {
            write!(stdout, "{:<8}", "|").unwrap();
        } else {
            let snippet = &source[self.spans[offset]];
            let snippet = &snippet[snippet.len().saturating_sub(7)..];
            write!(stdout, "{:<8}", snippet).unwrap();
        }

        let chunk = self.instructions[offset];
        let instruction: OpCode = chunk.into();
        let mut simple = |str| Chunk::simple_instruction(str, &mut offset, &mut stdout);
        match instruction {
            OpCode::Return => simple("RETURN"),
            OpCode::Constant => self.constant_instruction("CONSTANT", &mut offset, stdout),
            OpCode::Closure => self.closure(&mut offset, stdout),
            OpCode::Negate => simple("NEGATE"),
            OpCode::Add => simple("ADD"),
            OpCode::Sub => simple("SUBTRACT"),
            OpCode::Mul => simple("MULTIPLY"),
            OpCode::Div => simple("DIVIDE"),
            OpCode::Nil => simple("NIL"),
            OpCode::Not => simple("NOT"),
            OpCode::True => simple("TRUE"),
            OpCode::False => simple("FALSE"),
            OpCode::Equal => simple("EQUAL"),
            OpCode::Greater => simple("GREATER"),
            OpCode::Less => simple("LESS"),
            OpCode::Print => simple("PRINT"),
            OpCode::Pop => simple("POP"),
            OpCode::CloseUpvalue => simple("CLOSE_UPVALUE"),
            OpCode::DefineGlobal => self.global_instruction("DEFINE_GLOBAL", &mut offset, stdout),
            OpCode::GetGlobal => self.global_instruction("GET_GLOBAL", &mut offset, stdout),
            OpCode::SetGlobal => self.global_instruction("SET_GLOBAL", &mut offset, stdout),
            OpCode::SetLocal => self.byte_instruction("SET_LOCAL", &mut offset, stdout),
            OpCode::GetLocal => self.byte_instruction("GET_LOCAL", &mut offset, stdout),
            OpCode::SetUpvalue => self.byte_instruction("SET_UPVALUE", &mut offset, stdout),
            OpCode::GetUpvalue => self.byte_instruction("GET_UPVALUE", &mut offset, stdout),
            OpCode::Call => self.byte_instruction("CALL", &mut offset, stdout),
            OpCode::JumpRelIfFalse => {
                self.jmp_instruction("JUMP_REL_IF_FALSE", &mut offset, stdout)
            }
            OpCode::JumpRelIfTrue => self.jmp_instruction("JUMP_REL_IF_TRUE", &mut offset, stdout),
            OpCode::JumpRel => self.jmp_instruction("JUMP_REL", &mut offset, stdout),
            OpCode::Loop => self.jmp_instruction("LOOP", &mut offset, stdout),
            OpCode::Invalid => {
                writeln!(stdout, "INVALID OPCODE: {chunk}").unwrap();
                offset += 1;
            }
        }
        offset
    }
}
