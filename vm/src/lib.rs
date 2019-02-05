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
use std::convert::TryFrom;

pub struct Program(Vec<Instruction>);

pub enum Instruction {
    Fd {
        name: String,
        args: [Option<u8>; 8],
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
        name: String,
        register: u8,
    },
    Sg {
        name: String,
        register: u8,
    },
    Css {
        content: String,
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

pub enum Value {
    Constant(u64),
    Register(u8),
}

pub enum ParsingError {}

impl TryFrom<Vec<u8>> for Program {
    type Error = ParsingError;
    fn try_from(instructions: Vec<u8>) -> Result<Program, ParsingError> {
        let _peekable = instructions.iter().peekable();
        Ok(Program(Vec::new()))
    }
}
