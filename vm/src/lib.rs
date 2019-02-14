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
        | isub <Reg> <Val>
        | add <Reg> <Val>
        | sub <Reg> <Val>
        | umul <Reg> <Val>
        | smul <Reg> <Val>
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
#[macro_use]
extern crate runtime_fmt;
use failure::Error;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::iter::Iterator;

mod allocator;

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

    fn parse_i64(stream: &mut Iterator<Item = u8>) -> Result<i64, Error> {
        let bytes: Vec<u8> = stream.take(8).collect();
        if bytes.len() == 8 {
            let mut current_shift: u64 = 7 * 8;
            Ok(bytes.into_iter().fold(0i64, |acc, n| {
                let r = acc + ((n as i64) << current_shift);
                current_shift = current_shift.wrapping_sub(8);
                r
            }))
        } else {
            Err(Error::from(ParsingError::U64LacksInformation))
        }
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

#[derive(Debug, Fail)]
pub enum RuntimeError {
    #[fail(
        display = "Wrong number of arguments for {}: Expected {}, got {}.",
        name, expected, got
    )]
    WrongArgumentsNumber {
        name: String,
        expected: usize,
        got: usize,
    },
    #[fail(display = "Program ended with code {}", errno)]
    ProgramEnded { errno: u64 },
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
                0x05 => Ok(parse_instruction_from_register_to_value!(Ld8, &mut source)),
                0x06 => Ok(parse_instruction_from_register_to_value!(Ld16, &mut source)),
                0x07 => Ok(parse_instruction_from_register_to_value!(Ld32, &mut source)),
                0x08 => Ok(parse_instruction_from_register_to_value!(Ld64, &mut source)),
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

pub type CpuFn = Box<Fn(&StackBasedCpu, Vec<u8>) -> Result<u64, Error>>;
pub enum Function {
    Native(CpuFn),
    UserDefined,
}

fn registers_to_string(registers: &[u64; 256], index: usize) -> Result<String, Error> {
    let u8_contents = registers[index..]
        .iter()
        .map(|n| extract_bytes(n.clone()).to_vec().into_iter())
        .flatten()
        .collect::<Vec<u8>>();
    String::from_utf8(u8_contents).map_err(|e| Error::from(e))
}

pub trait StackBasedCpu {
    fn current_register_stack(&self) -> &[u64; 256];

    fn puts(&self, args: Vec<u8>) -> Result<u64, Error> {
        if args.len() == 1 {
            let registers: &[u64; 256] = self.current_register_stack();
            registers_to_string(registers, args[0] as usize).map(|s| {
                println!("{}", s);
                0
            })
        } else {
            Err(Error::from(RuntimeError::WrongArgumentsNumber {
                name: "puts".to_owned(),
                expected: 1,
                got: args.len(),
            }))
        }
    }

    fn printf(&self, args: Vec<u8>) -> Result<u64, Error> {
        let registers: &[u64; 256] = self.current_register_stack();
        match args.len() {
            1 => {
                let content = registers_to_string(registers, args[0] as usize)?;
                rt_println!(content).unwrap();
                Ok(0)
            }
            2 => {
                let content = registers_to_string(registers, args[0] as usize)?;
                rt_println!(content, registers_to_string(registers, args[1] as usize)?).unwrap();
                Ok(0)
            }
            3 => {
                let content = registers_to_string(registers, args[0] as usize)?;
                rt_println!(
                    content,
                    registers_to_string(registers, args[1] as usize)?,
                    registers_to_string(registers, args[2] as usize)?,
                )
                .unwrap();
                Ok(0)
            }
            4 => {
                let content = registers_to_string(registers, args[0] as usize)?;
                rt_println!(
                    content,
                    registers_to_string(registers, args[1] as usize)?,
                    registers_to_string(registers, args[2] as usize)?,
                    registers_to_string(registers, args[3] as usize)?,
                )
                .unwrap();
                Ok(0)
            }
            5 => {
                let content = registers_to_string(registers, args[0] as usize)?;
                rt_println!(
                    content,
                    registers_to_string(registers, args[1] as usize)?,
                    registers_to_string(registers, args[2] as usize)?,
                    registers_to_string(registers, args[3] as usize)?,
                    registers_to_string(registers, args[4] as usize)?,
                )
                .unwrap();
                Ok(0)
            }
            6 => {
                let content = registers_to_string(registers, args[0] as usize)?;
                rt_println!(
                    content,
                    registers_to_string(registers, args[1] as usize)?,
                    registers_to_string(registers, args[2] as usize)?,
                    registers_to_string(registers, args[3] as usize)?,
                    registers_to_string(registers, args[4] as usize)?,
                    registers_to_string(registers, args[5] as usize)?,
                )
                .unwrap();
                Ok(0)
            }
            7 => {
                let content = registers_to_string(registers, args[0] as usize)?;
                rt_println!(
                    content,
                    registers_to_string(registers, args[1] as usize)?,
                    registers_to_string(registers, args[2] as usize)?,
                    registers_to_string(registers, args[3] as usize)?,
                    registers_to_string(registers, args[4] as usize)?,
                    registers_to_string(registers, args[5] as usize)?,
                    registers_to_string(registers, args[6] as usize)?,
                )
                .unwrap();
                Ok(0)
            }
            8 => {
                let content = registers_to_string(registers, args[0] as usize)?;
                rt_println!(
                    content,
                    registers_to_string(registers, args[1] as usize)?,
                    registers_to_string(registers, args[2] as usize)?,
                    registers_to_string(registers, args[3] as usize)?,
                    registers_to_string(registers, args[4] as usize)?,
                    registers_to_string(registers, args[5] as usize)?,
                    registers_to_string(registers, args[6] as usize)?,
                    registers_to_string(registers, args[7] as usize)?,
                )
                .unwrap();
                Ok(0)
            }
            n => Err(Error::from(RuntimeError::WrongArgumentsNumber {
                name: "printf".to_owned(),
                expected: 8,
                got: n,
            })),
        }
    }

    fn exit(&self, args: Vec<u8>) -> Result<u64, Error> {
        if args.len() != 1 {
            Err(Error::from(RuntimeError::WrongArgumentsNumber {
                name: "exit".to_owned(),
                expected: 1,
                got: args.len(),
            }))
        } else {
            Err(Error::from(RuntimeError::ProgramEnded {
                errno: self.current_register_stack()[0].clone(),
            }))
        }
    }
}

pub struct Cpu {
    functions: HashMap<String, Function>,
    register_stack: Vec<[u64; 256]>,
}

fn extract_bytes(n: u64) -> [u8; 8] {
    let mut res = [0; 8];
    for i in 0..7 {
        res[i] = (n >> (7 - i)) as u8;
    }
    res
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut functions = HashMap::new();
        functions.insert(
            "puts".to_owned(),
            Function::Native(Box::new(|cpu, args| cpu.puts(args))),
        );
        functions.insert(
            "printf".to_owned(),
            Function::Native(Box::new(|cpu, args| cpu.printf(args))),
        );
        functions.insert(
            "exit".to_owned(),
            Function::Native(Box::new(|cpu, args| cpu.exit(args))),
        );
        Cpu {
            functions,
            register_stack: vec![[0; 256]],
        }
    }
}

impl StackBasedCpu for Cpu {
    fn current_register_stack(&self) -> &[u64; 256] {
        self.register_stack.last().unwrap()
    }
}
