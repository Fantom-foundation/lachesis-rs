use crate::error::ParsingError;
use failure::Error;
use std::convert::TryFrom;

pub struct Program(pub Vec<Instruction>);

#[derive(Clone)]
pub enum Instruction {
    Fd {
        name: String,
        args: u64,
        skip: u64,
    },
    Mov {
        register: u8,
        value: Value,
    },
    Ineg {
        register: u8,
    },
    Iadd {
        register: u8,
        value: Value,
    },
    Isub {
        register: u8,
        value: Value,
    },
    Add {
        register: u8,
        value: Value,
    },
    Sub {
        register: u8,
        value: Value,
    },
    Umul {
        register: u8,
        value: Value,
    },
    Smul {
        register: u8,
        value: Value,
    },
    Urem {
        register: u8,
        value: Value,
    },
    Srem {
        register: u8,
        value: Value,
    },
    Udiv {
        register: u8,
        value: Value,
    },
    Sdiv {
        register: u8,
        value: Value,
    },
    And {
        register: u8,
        value: Value,
    },
    Or {
        register: u8,
        value: Value,
    },
    Xor {
        register: u8,
        value: Value,
    },
    Shl {
        register: u8,
        value: Value,
    },
    Lshr {
        register: u8,
        value: Value,
    },
    Ashr {
        register: u8,
        value: Value,
    },
    Fadd {
        register: u8,
        value: Value,
    },
    Fsub {
        register: u8,
        value: Value,
    },
    Fmul {
        register: u8,
        value: Value,
    },
    Frem {
        register: u8,
        value: Value,
    },
    Fdiv {
        register: u8,
        value: Value,
    },
    Eq {
        register: u8,
        value: Value,
    },
    Ne {
        register: u8,
        value: Value,
    },
    Slt {
        register: u8,
        value: Value,
    },
    Sle {
        register: u8,
        value: Value,
    },
    Sgt {
        register: u8,
        value: Value,
    },
    Sge {
        register: u8,
        value: Value,
    },
    Feq {
        register: u8,
        value: Value,
    },
    Fne {
        register: u8,
        value: Value,
    },
    Flt {
        register: u8,
        value: Value,
    },
    Fle {
        register: u8,
        value: Value,
    },
    Fgt {
        register: u8,
        value: Value,
    },
    Fge {
        register: u8,
        value: Value,
    },
    Ult {
        register: u8,
        value: Value,
    },
    Ule {
        register: u8,
        value: Value,
    },
    Ugt {
        register: u8,
        value: Value,
    },
    Uge {
        register: u8,
        value: Value,
    },
    Ld8 {
        register: u8,
        value: Value,
    },
    Ld16 {
        register: u8,
        value: Value,
    },
    Ld32 {
        register: u8,
        value: Value,
    },
    Ld64 {
        register: u8,
        value: Value,
    },
    St8 {
        register: u8,
        value: Value,
    },
    St16 {
        register: u8,
        value: Value,
    },
    St32 {
        register: u8,
        value: Value,
    },
    St64 {
        register: u8,
        value: Value,
    },
    Jmp {
        offset: i64,
    },
    Jz {
        offset: i64,
        register: u8,
    },
    Jnz {
        offset: i64,
        register: u8,
    },
    Lea {
        destiny: u8,
        source: u8,
    },
    Leave,
    Ret {
        value: Value,
    },
    Gg {
        string: String,
        register: u8,
    },
    Sg {
        string: String,
        register: u8,
    },
    Css {
        string: String,
        register: u8,
    },
    CssDyn {
        destiny: u8,
        source: u8,
    },
    Call {
        return_register: u8,
        arguments: [Option<u8>; 8],
    },
}

macro_rules! parse_instruction_with_register_and_offset {
    ($instr: ident, $stream: expr) => {
        Instruction::$instr {
            offset: Instruction::parse_i64($stream)?,
            register: Instruction::parse_register($stream)?,
        }
    };
}

macro_rules! parse_instruction_with_register {
    ($instr: ident, $stream: expr) => {
        Instruction::$instr {
            register: Instruction::parse_register($stream)?,
        }
    };
}

macro_rules! parse_instruction_from_register_to_register {
    ($instr: ident, $stream: expr) => {
        Instruction::$instr {
            source: Instruction::parse_register($stream)?,
            destiny: Instruction::parse_register($stream)?,
        }
    };
}

macro_rules! parse_instruction_from_register_to_value {
    ($instr: ident, $stream: expr) => {
        Instruction::$instr {
            value: Instruction::parse_value($stream)?,
            register: Instruction::parse_register($stream)?,
        }
    };
}

macro_rules! parse_instruction_with_string_and_register {
    ($instr: ident, $stream: expr) => {
        Instruction::$instr {
            string: Instruction::parse_string($stream)?,
            register: Instruction::parse_register($stream)?,
        }
    };
}

impl Instruction {
    fn parse_fd(stream: &mut Iterator<Item = u8>) -> Result<Instruction, Error> {
        let name = Instruction::parse_string(stream)?;
        let args = Instruction::parse_u64(stream)?;
        let skip = Instruction::parse_u64(stream)?;
        Ok(Instruction::Fd { name, args, skip })
    }

    fn parse_call(stream: &mut Iterator<Item = u8>, nargs: usize) -> Result<Instruction, Error> {
        let return_register = Instruction::parse_register(stream)?;
        let mut arguments = [None; 8];
        for i in 0..(nargs - 1) {
            arguments[i] = Some(Instruction::parse_register(stream)?);
        }
        Ok(Instruction::Call {
            arguments,
            return_register,
        })
    }

    fn parse_string(stream: &mut Iterator<Item = u8>) -> Result<String, Error> {
        let string_length = stream
            .next()
            .ok_or(Error::from(ParsingError::StringWithoutLenght))?
            as usize;
        let string_bytes: Vec<u8> = stream.take(string_length).collect();
        let name = String::from_utf8(string_bytes)?;
        Ok(name)
    }

    fn parse_u16(stream: &mut Iterator<Item = u8>) -> Result<u16, Error> {
        let bytes: Vec<u8> = stream.take(2).collect();
        if bytes.len() == 2 {
            let byte_pairs: &[u16] =
                unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u16, 1) };
            Ok(byte_pairs[0])
        } else {
            Err(Error::from(ParsingError::UnexpectedEndOfStream))
        }
    }

    fn parse_u32(stream: &mut Iterator<Item = u8>) -> Result<u32, Error> {
        let bytes: Vec<u8> = stream.take(4).collect();
        if bytes.len() == 4 {
            let byte_pairs: &[u32] =
                unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u32, 1) };
            Ok(byte_pairs[0])
        } else {
            Err(Error::from(ParsingError::UnexpectedEndOfStream))
        }
    }

    fn parse_i64(stream: &mut Iterator<Item = u8>) -> Result<i64, Error> {
        let bytes: Vec<u8> = stream.take(8).collect();
        if bytes.len() == 8 {
            let byte_groups: &[i64] =
                unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const i64, 1) };
            Ok(byte_groups[0])
        } else {
            Err(Error::from(ParsingError::U64LacksInformation))
        }
    }

    fn parse_u64(stream: &mut Iterator<Item = u8>) -> Result<u64, Error> {
        let bytes: Vec<u8> = stream.take(8).collect();
        if bytes.len() == 8 {
            let byte_groups: &[u64] =
                unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u64, 1) };
            Ok(byte_groups[0])
        } else {
            Err(Error::from(ParsingError::U64LacksInformation))
        }
    }

    fn parse_register(stream: &mut Iterator<Item = u8>) -> Result<u8, Error> {
        stream
            .next()
            .ok_or(Error::from(ParsingError::RegisterExpectedNothingFound))
    }

    fn parse_value(stream: &mut Iterator<Item = u8>) -> Result<Value, Error> {
        let flag = stream
            .next()
            .ok_or(Error::from(ParsingError::ValueWithNoFlag))?;
        if flag == 0 {
            Instruction::parse_register(stream).map(|v| Value::Register(v))
        } else {
            Instruction::parse_u64(stream).map(|c| Value::Constant(c))
        }
    }
}

#[derive(Clone)]
pub enum Value {
    Constant(u64),
    Register(u8),
}

impl TryFrom<Vec<u8>> for Program {
    type Error = Error;
    fn try_from(instructions: Vec<u8>) -> Result<Program, Error> {
        let mut source = instructions.into_iter();
        let mut instructions = Vec::new();
        let mut next = source.next();
        while next.is_some() {
            let byte = next.expect("can't happen");
            let instruction = match byte {
                0x00 => Instruction::parse_fd(&mut source),
                0x01 => Ok(parse_instruction_from_register_to_value!(Mov, &mut source)),
                0x02 => Ok(parse_instruction_with_string_and_register!(Gg, &mut source)),
                0x03 => Ok(parse_instruction_with_string_and_register!(Sg, &mut source)),
                0x04 => Ok(parse_instruction_with_string_and_register!(
                    Css,
                    &mut source
                )),
                0x05 => {
                    let content = source
                        .next()
                        .ok_or(Error::from(ParsingError::UnexpectedEndOfStream))?;
                    let value = Value::Constant(content as u64);
                    Ok(Instruction::Ld8 {
                        value,
                        register: Instruction::parse_register(&mut source)?,
                    })
                }
                0x06 => {
                    let content = Instruction::parse_u16(&mut source)?;
                    let value = Value::Constant(content as u64);
                    Ok(Instruction::Ld16 {
                        value,
                        register: Instruction::parse_register(&mut source)?,
                    })
                }
                0x07 => {
                    let content = Instruction::parse_u32(&mut source)?;
                    let value = Value::Constant(content as u64);
                    Ok(Instruction::Ld32 {
                        value,
                        register: Instruction::parse_register(&mut source)?,
                    })
                }
                0x08 => {
                    let content = Instruction::parse_u64(&mut source)?;
                    let value = Value::Constant(content as u64);
                    Ok(Instruction::Ld64 {
                        value,
                        register: Instruction::parse_register(&mut source)?,
                    })
                }
                0x09 => Ok(parse_instruction_from_register_to_value!(St8, &mut source)),
                0x0a => Ok(parse_instruction_from_register_to_value!(St16, &mut source)),
                0x0b => Ok(parse_instruction_from_register_to_value!(St32, &mut source)),
                0x0c => Ok(parse_instruction_from_register_to_value!(St64, &mut source)),
                0x0d => Ok(parse_instruction_from_register_to_register!(
                    Lea,
                    &mut source
                )),
                0x0e => Ok(parse_instruction_from_register_to_value!(Iadd, &mut source)),
                0x0f => Ok(parse_instruction_from_register_to_value!(Isub, &mut source)),
                0x10 => Ok(parse_instruction_from_register_to_value!(Smul, &mut source)),
                0x11 => Ok(parse_instruction_from_register_to_value!(Umul, &mut source)),
                0x12 => Ok(parse_instruction_from_register_to_value!(Srem, &mut source)),
                0x13 => Ok(parse_instruction_from_register_to_value!(Urem, &mut source)),
                0x14 => Ok(parse_instruction_from_register_to_value!(Sdiv, &mut source)),
                0x15 => Ok(parse_instruction_from_register_to_value!(Udiv, &mut source)),
                0x16 => Ok(parse_instruction_from_register_to_value!(And, &mut source)),
                0x17 => Ok(parse_instruction_from_register_to_value!(Or, &mut source)),
                0x18 => Ok(parse_instruction_from_register_to_value!(Xor, &mut source)),
                0x19 => Ok(parse_instruction_from_register_to_value!(Shl, &mut source)),
                0x1a => Ok(parse_instruction_from_register_to_value!(Lshr, &mut source)),
                0x1b => Ok(parse_instruction_from_register_to_value!(Ashr, &mut source)),
                0x1c => Ok(parse_instruction_with_register!(Ineg, &mut source)),
                0x1d => Ok(parse_instruction_from_register_to_value!(Fadd, &mut source)),
                0x1e => Ok(parse_instruction_from_register_to_value!(Fsub, &mut source)),
                0x1f => Ok(parse_instruction_from_register_to_value!(Fmul, &mut source)),
                0x20 => Ok(parse_instruction_from_register_to_value!(Fdiv, &mut source)),
                0x21 => Ok(parse_instruction_from_register_to_value!(Frem, &mut source)),
                0x22 => Ok(parse_instruction_from_register_to_value!(Eq, &mut source)),
                0x23 => Ok(parse_instruction_from_register_to_value!(Ne, &mut source)),
                0x24 => Ok(parse_instruction_from_register_to_value!(Slt, &mut source)),
                0x25 => Ok(parse_instruction_from_register_to_value!(Sle, &mut source)),
                0x26 => Ok(parse_instruction_from_register_to_value!(Sgt, &mut source)),
                0x27 => Ok(parse_instruction_from_register_to_value!(Sge, &mut source)),
                0x28 => Ok(parse_instruction_from_register_to_value!(Ult, &mut source)),
                0x29 => Ok(parse_instruction_from_register_to_value!(Ule, &mut source)),
                0x2a => Ok(parse_instruction_from_register_to_value!(Ugt, &mut source)),
                0x2b => Ok(parse_instruction_from_register_to_value!(Uge, &mut source)),
                0x2c => Ok(parse_instruction_from_register_to_value!(Feq, &mut source)),
                0x2d => Ok(parse_instruction_from_register_to_value!(Fne, &mut source)),
                0x2e => Ok(parse_instruction_from_register_to_value!(Flt, &mut source)),
                0x2f => Ok(parse_instruction_from_register_to_value!(Fle, &mut source)),
                0x30 => Ok(parse_instruction_from_register_to_value!(Fgt, &mut source)),
                0x31 => Ok(parse_instruction_from_register_to_value!(Fge, &mut source)),
                0x32 => Ok(Instruction::Jmp {
                    offset: Instruction::parse_i64(&mut source)?,
                }),
                0x33 => Ok(parse_instruction_with_register_and_offset!(
                    Jnz,
                    &mut source
                )),
                0x34 => Ok(parse_instruction_with_register_and_offset!(Jz, &mut source)),
                0x35 => Instruction::parse_call(&mut source, 0),
                0x36 => Instruction::parse_call(&mut source, 1),
                0x37 => Instruction::parse_call(&mut source, 2),
                0x38 => Instruction::parse_call(&mut source, 3),
                0x39 => Instruction::parse_call(&mut source, 4),
                0x3a => Instruction::parse_call(&mut source, 5),
                0x3b => Instruction::parse_call(&mut source, 6),
                0x3c => Instruction::parse_call(&mut source, 7),
                0x3d => Instruction::parse_call(&mut source, 8),
                0x3e => Ok(Instruction::Ret {
                    value: Instruction::parse_value(&mut source)?,
                }),
                0x3f => Ok(Instruction::Leave),
                0x40 => Ok(parse_instruction_from_register_to_register!(
                    CssDyn,
                    &mut source
                )),
                _ => Err(Error::from(ParsingError::InvalidInstructionByte)),
            }?;
            instructions.push(instruction);
            next = source.next();
        }
        Ok(Program(instructions))
    }
}
