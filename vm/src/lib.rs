#![feature(try_from)]

/**
Each stack frame has its own set of 256 registers.

<u64> = uint64_t.
<Reg> = uint8_t.
<string> = <length:u32> <char>...

<Program> = <Instr>...

<Instr> = fd <Name:string> <nargs:u64> <nskip:n64>  # function definition
        | mov <Reg> <Val>
        | ineg <Reg>
        | iadd <Reg> <Val>
        | add <Reg> <Val>
        | sub <Reg> <Val>
        | mul <Reg> <Val>
        | urem <Reg> <Val>
        | srem <Reg> <Val>
        | udiv <Reg> <Val>
        | sdiv <Reg> <Val>
        | and <Reg> <Val>
        | or <Reg> <Val>
        | xor <Reg> <Val>
        | shl <Reg> <Val>
        | lshr <Reg> <Val>
        | ashr <Reg> <Val>
        | fadd <Reg> <Val>
        | fsub <Reg> <Val>
        | fmul <Reg> <Val>
        | frem <Reg> <Val>
        | fdiv <Reg> <Val>
        | eq <Reg> <Val>           # == bitwise
        | ne <Reg> <Val>           # != bitwise
        | slt <Reg> <Val>          # <  as signed integers
        | sle <Reg> <Val>          # <= as signed integers
        | sgt <Reg> <Val>          # >  as signed integers
        | sge <Reg> <Val>          # >= as signed integers
        | ult <Reg> <Val>          # <  as unsigned integers
        | ule <Reg> <Val>          # <= as unsigned integers
        | ugt <Reg> <Val>          # >  as unsigned integers
        | uge <Reg> <Val>          # >= as unsigned integers
        | ld8 <Reg> <Val>          # <Reg> = *(uint8_t *)<Val>
        | ld16 <Reg> <Val>         # <Reg> = *(uint16_t *)<Val>
        | ld32 <Reg> <Val>         # <Reg> = *(uint32_t *)<Val>
        | ld64 <Reg> <Val>         # <Reg> = *(uint64_t *)<Val>
        | st8 <Val> <Reg>          # *(uint8_t *)<Val> = <Reg>
        | st16 <Val> <Reg>         # *(uint16_t *)<Val> = <Reg>
        | st32 <Val> <Reg>         # *(uint32_t *)<Val> = <Reg>
        | st64 <Val> <Reg>         # *(uint64_t *)<Val> = <Reg>
        | jmp <Off>                # unconditional relative jump
        | jz <Reg> <Off>           # relative jump if <Reg> == 0
        | lea <Reg1> <Reg2>        # <Reg1> = &<Reg2> (load effective address)
        | leave                    # leave function returning nothing
        | ret <Val>                # return <Val> from function
        | gg <Name:string> <Reg>   # get global
        | sg <Name:string> <Reg>   # set global
        | css <string> <Reg>       # create static string; write pointer to <Reg>
        | css_dyn <Reg1> <Reg2>    # create static string of length 8 and put <Reg2> into it
        | call0 <Reg>
        | call1 <Reg> <Reg1>
        | call2 <Reg> <Reg1> <Reg2>
        | call3 <Reg> <Reg1> <Reg2> <Reg3>
        | call4 <Reg> <Reg1> <Reg2> <Reg3> <Reg4>
        | call5 <Reg> <Reg1> <Reg2> <Reg3> <Reg4> <Reg5>
        | call6 <Reg> <Reg1> <Reg2> <Reg3> <Reg4> <Reg5> <Reg6>
        | call7 <Reg> <Reg1> <Reg2> <Reg3> <Reg4> <Reg5> <Reg6> <Reg7>
        | call8 <Reg> <Reg1> <Reg2> <Reg3> <Reg4> <Reg5> <Reg6> <Reg7> <Reg8>

call<N> functions put the return value into the function register (<Reg>).

Instructions that may take either a register or constant operand (<Val>) are encoded as follows:
    <instruction byte> <byte with value 0> <Reg>
or
    <instruction byte> <byte with value 1> <Constant:u64>
*/
#[macro_use]
extern crate failure;
use failure::Error;
use std::convert::TryFrom;
use std::iter::Iterator;

pub struct Program(Vec<Instruction>);

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
    Add {
        register: u8,
        value: Value,
    },
    Sub {
        register: u8,
        value: Value,
    },
    Mul {
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

macro_rules! parse_instruction_from_register_to_value {
    ($instr: ident, $stream: ident) => {
        Instruction::$instr {
            value: Instruction::parse_value($stream)?,
            register: Instruction::parse_register($stream)?,
        }
    };
}

macro_rules! parse_instruction_with_string_and_register {
    ($instr: ident, $stream: ident) => {
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

    fn parse_mov(stream: &mut Iterator<Item = u8>) -> Result<Instruction, Error> {
        Ok(parse_instruction_from_register_to_value!(Mov, stream))
    }

    fn parse_gg(stream: &mut Iterator<Item = u8>) -> Result<Instruction, Error> {
        Ok(parse_instruction_with_string_and_register!(Gg, stream))
    }

    fn parse_sg(stream: &mut Iterator<Item = u8>) -> Result<Instruction, Error> {
        Ok(parse_instruction_with_string_and_register!(Sg, stream))
    }

    fn parse_css(stream: &mut Iterator<Item = u8>) -> Result<Instruction, Error> {
        Ok(parse_instruction_with_string_and_register!(Css, stream))
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

    fn parse_u64(stream: &mut Iterator<Item = u8>) -> Result<u64, Error> {
        let bytes: Vec<u8> = stream.take(8).collect();
        if bytes.len() == 8 {
            let mut current_shift: u64 = 7 * 8;
            Ok(bytes.into_iter().fold(0u64, |acc, n| {
                let r = acc + ((n as u64) << current_shift);
                current_shift = current_shift.wrapping_sub(8);
                r
            }))
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

pub enum Value {
    Constant(u64),
    Register(u8),
}

#[derive(Debug, Fail)]
pub enum ParsingError {
    #[fail(display = "The string being parsed doesn't have lenght")]
    StringWithoutLenght,
    #[fail(display = "There weren't enough bytes to form a u64")]
    U64LacksInformation,
    #[fail(display = "Type flag not present when parsing value")]
    ValueWithNoFlag,
    #[fail(display = "Empty strean when trying to parse a register")]
    RegisterExpectedNothingFound,
    #[fail(display = "Invalid instruction byte")]
    InvalidInstructionByte,
}

impl TryFrom<Vec<u8>> for Program {
    type Error = Error;
    fn try_from(instructions: Vec<u8>) -> Result<Program, Error> {
        let mut peekable = instructions.into_iter();
        let mut instructions = Vec::new();
        let mut next = peekable.next();
        while next.is_some() {
            let byte = next.expect("can't happen");
            let instruction = match byte {
                0x00 => Instruction::parse_fd(&mut peekable),
                0x01 => Instruction::parse_mov(&mut peekable),
                0x02 => Instruction::parse_gg(&mut peekable),
                0x03 => Instruction::parse_sg(&mut peekable),
                0x04 => Instruction::parse_css(&mut peekable),
                _ => Err(Error::from(ParsingError::InvalidInstructionByte)),
            }?;
            instructions.push(instruction);
            next = peekable.next();
        }
        Ok(Program(instructions))
    }
}
/*
enum Commands : unsigned char {
    CMD_LD8,
    CMD_LD16,
    CMD_LD32,
    CMD_LD64,
    CMD_ST8,
    CMD_ST16,
    CMD_ST32,
    CMD_ST64,
    CMD_LEA,

    CMD_IADD,
    CMD_ISUB,
    CMD_SMUL,
    CMD_UMUL,
    CMD_SREM,
    CMD_UREM,
    CMD_SDIV,
    CMD_UDIV,

    CMD_AND,
    CMD_OR,
    CMD_XOR,
    CMD_SHL,
    CMD_LSHR,
    CMD_ASHR,
    CMD_INEG,

    CMD_FADD,
    CMD_FSUB,
    CMD_FMUL,
    CMD_FDIV,
    CMD_FREM,

    CMD_EQ,
    CMD_NE,

    CMD_SLT,
    CMD_SLE,
    CMD_SGT,
    CMD_SGE,
    CMD_ULT,
    CMD_ULE,
    CMD_UGT,
    CMD_UGE,
    CMD_FEQ,
    CMD_FNE,
    CMD_FLT,
    CMD_FLE,
    CMD_FGT,
    CMD_FGE,
    CMD_JMP,
    CMD_JNZ,
    CMD_JZ,
    CMD_CALL0,
    CMD_CALL1,
    CMD_CALL2,
    CMD_CALL3,
    CMD_CALL4,
    CMD_CALL5,
    CMD_CALL6,
    CMD_CALL7,
    CMD_CALL8,
    CMD_RET,
    CMD_LEAVE,
    CMD_CSS_DYN,
};
*/
