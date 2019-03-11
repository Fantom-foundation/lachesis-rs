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
use crate::allocator::Allocator;
use crate::error::RuntimeError;
use crate::instruction::{Instruction, Program, Value};
use crate::memory::Memory;
use crate::register_set::RegisterSet;
use failure::Error;
use libc::scanf;
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::Iterator;
use std::ops::Rem;
use std::rc::Rc;

mod allocator;
mod error;
pub mod instruction;
mod memory;
mod register_set;

type CpuFn = Box<Fn(&NativeFunctions, Vec<u8>) -> Result<u64, Error>>;
#[repr(C)]
enum Function {
    Native(CpuFn),
    UserDefined(usize, u64, u64),
}

struct NativeFunctions {
    allocator: Rc<RefCell<Allocator>>,
    register_stack: Rc<RefCell<Vec<RegisterSet>>>,
}

impl NativeFunctions {
    fn puts(&self, args: Vec<u8>) -> Result<u64, Error> {
        if args.len() == 1 {
            self.register_stack
                .borrow()
                .last()
                .unwrap()
                .to_string(args[0] as usize)
                .map(|s| {
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
        let rs = self.register_stack.borrow();
        let registers: &RegisterSet = rs.last().unwrap();
        match args.len() {
            1 => {
                let content = registers.to_string(args[0] as usize)?;
                rt_println!(content).unwrap();
                Ok(0)
            }
            2 => {
                let content = registers.to_string(args[0] as usize)?;
                rt_println!(content, registers.to_string(args[1] as usize)?).unwrap();
                Ok(0)
            }
            3 => {
                let content = registers.to_string(args[0] as usize)?;
                rt_println!(
                    content,
                    registers.to_string(args[1] as usize)?,
                    registers.to_string(args[2] as usize)?,
                )
                .unwrap();
                Ok(0)
            }
            4 => {
                let content = registers.to_string(args[0] as usize)?;
                rt_println!(
                    content,
                    registers.to_string(args[1] as usize)?,
                    registers.to_string(args[2] as usize)?,
                    registers.to_string(args[3] as usize)?,
                )
                .unwrap();
                Ok(0)
            }
            5 => {
                let content = registers.to_string(args[0] as usize)?;
                rt_println!(
                    content,
                    registers.to_string(args[1] as usize)?,
                    registers.to_string(args[2] as usize)?,
                    registers.to_string(args[3] as usize)?,
                    registers.to_string(args[4] as usize)?,
                )
                .unwrap();
                Ok(0)
            }
            6 => {
                let content = registers.to_string(args[0] as usize)?;
                rt_println!(
                    content,
                    registers.to_string(args[1] as usize)?,
                    registers.to_string(args[2] as usize)?,
                    registers.to_string(args[3] as usize)?,
                    registers.to_string(args[4] as usize)?,
                    registers.to_string(args[5] as usize)?,
                )
                .unwrap();
                Ok(0)
            }
            7 => {
                let content = registers.to_string(args[0] as usize)?;
                rt_println!(
                    content,
                    registers.to_string(args[1] as usize)?,
                    registers.to_string(args[2] as usize)?,
                    registers.to_string(args[3] as usize)?,
                    registers.to_string(args[4] as usize)?,
                    registers.to_string(args[5] as usize)?,
                    registers.to_string(args[6] as usize)?,
                )
                .unwrap();
                Ok(0)
            }
            8 => {
                let content = registers.to_string(args[0] as usize)?;
                rt_println!(
                    content,
                    registers.to_string(args[1] as usize)?,
                    registers.to_string(args[2] as usize)?,
                    registers.to_string(args[3] as usize)?,
                    registers.to_string(args[4] as usize)?,
                    registers.to_string(args[5] as usize)?,
                    registers.to_string(args[6] as usize)?,
                    registers.to_string(args[7] as usize)?,
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

    fn scanf(&self, args: Vec<u8>) -> Result<u64, Error> {
        if args.len() == 0 {
            return Err(Error::from(RuntimeError::WrongArgumentsNumber {
                name: "scanf".to_owned(),
                expected: 8,
                got: 0,
            }));
        }
        let rc = self.register_stack.borrow();
        let registers = rc.last().unwrap();
        let content: Vec<i8> = registers
            .to_string(args[0] as usize)?
            .into_bytes()
            .iter()
            .map(|v| v.clone() as i8)
            .collect();
        let mut args_ptr = args.clone();
        let r = match args.len() {
            1 => Ok(unsafe { scanf((&content).as_ptr()) }),
            2 => Ok(unsafe { scanf((&content).as_ptr(), args_ptr.as_mut_ptr().add(1)) }),
            3 => Ok(unsafe {
                scanf(
                    (&content).as_ptr(),
                    args_ptr.as_mut_ptr().add(1),
                    args_ptr.as_mut_ptr().add(2),
                )
            }),
            4 => Ok(unsafe {
                scanf(
                    (&content).as_ptr(),
                    args_ptr.as_mut_ptr().add(1),
                    args_ptr.as_mut_ptr().add(2),
                    args_ptr.as_mut_ptr().add(3),
                )
            }),
            5 => Ok(unsafe {
                scanf(
                    (&content).as_ptr(),
                    args_ptr.as_mut_ptr().add(1),
                    args_ptr.as_mut_ptr().add(2),
                    args_ptr.as_mut_ptr().add(3),
                    args_ptr.as_mut_ptr().add(4),
                )
            }),
            6 => Ok(unsafe {
                scanf(
                    (&content).as_ptr(),
                    args_ptr.as_mut_ptr().add(1),
                    args_ptr.as_mut_ptr().add(2),
                    args_ptr.as_mut_ptr().add(3),
                    args_ptr.as_mut_ptr().add(4),
                    args_ptr.as_mut_ptr().add(5),
                )
            }),
            7 => Ok(unsafe {
                scanf(
                    (&content).as_ptr(),
                    args_ptr.as_mut_ptr().add(1),
                    args_ptr.as_mut_ptr().add(2),
                    args_ptr.as_mut_ptr().add(3),
                    args_ptr.as_mut_ptr().add(4),
                    args_ptr.as_mut_ptr().add(5),
                    args_ptr.as_mut_ptr().add(6),
                )
            }),
            8 => Ok(unsafe {
                scanf(
                    (&content).as_ptr(),
                    args_ptr.as_mut_ptr().add(1),
                    args_ptr.as_mut_ptr().add(2),
                    args_ptr.as_mut_ptr().add(3),
                    args_ptr.as_mut_ptr().add(4),
                    args_ptr.as_mut_ptr().add(5),
                    args_ptr.as_mut_ptr().add(6),
                    args_ptr.as_mut_ptr().add(7),
                )
            }),
            n => Err(Error::from(RuntimeError::WrongArgumentsNumber {
                name: "scanf".to_owned(),
                expected: 8,
                got: n,
            })),
        }?;
        Ok(r as u64)
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
                errno: self.register_stack.borrow().last().unwrap().get(0)?,
            }))
        }
    }

    fn malloc(&self, args: Vec<u8>) -> Result<u64, Error> {
        if args.len() != 1 {
            Err(Error::from(RuntimeError::WrongArgumentsNumber {
                name: "malloc".to_owned(),
                expected: 1,
                got: args.len(),
            }))
        } else {
            let size = self.register_stack.borrow().last().unwrap().get(0)? as usize;
            self.allocator.borrow_mut().malloc(size).map(|v| v as u64)
        }
    }

    fn free(&self, args: Vec<u8>) -> Result<u64, Error> {
        if args.len() != 1 {
            Err(Error::from(RuntimeError::WrongArgumentsNumber {
                name: "free".to_owned(),
                expected: 1,
                got: args.len(),
            }))
        } else {
            let address = self.register_stack.borrow().last().unwrap().get(0)? as usize;
            self.allocator.borrow_mut().free(address).map(|_| 0)
        }
    }
}

pub struct Cpu {
    allocator: Rc<RefCell<Allocator>>,
    call_stack: Vec<(usize, u8)>,
    pub(crate) functions: HashMap<String, Function>,
    globals: HashMap<String, u64>,
    memory: Memory,
    register_stack: Rc<RefCell<Vec<RegisterSet>>>,
}

impl Cpu {
    pub fn new(capacity: usize) -> Result<Cpu, Error> {
        let memory = Memory::new(capacity);
        let allocator = Rc::new(RefCell::new(Allocator::new(capacity)));
        let register_stack = Rc::new(RefCell::new(vec![]));
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
        functions.insert(
            "malloc".to_owned(),
            Function::Native(Box::new(|cpu, args| cpu.malloc(args))),
        );
        functions.insert(
            "free".to_owned(),
            Function::Native(Box::new(|cpu, args| cpu.free(args))),
        );
        let mut cpu = Cpu {
            allocator,
            functions,
            memory,
            register_stack,
            call_stack: Vec::new(),
            globals: HashMap::new(),
        };
        cpu.add_function("puts")?;
        cpu.add_function("printf")?;
        cpu.add_function("malloc")?;
        cpu.add_function("exit")?;
        cpu.add_function("free")?;
        let register_set = cpu.create_new_register_set()?;
        cpu.register_stack.borrow_mut().push(register_set);
        Ok(cpu)
    }

    fn add_function(&mut self, name: &'static str) -> Result<(), Error> {
        match self.functions.get(name) {
            Some(f) => {
                let address = self.allocator.borrow_mut().malloc_t::<Function>()?;
                self.memory.copy_t(f, address);
                Ok(())
            }
            None => Err(Error::from(RuntimeError::GlobalNotFound {
                name: name.clone().to_owned(),
            })),
        }
    }

    pub fn execute(&mut self, program: Program) -> Result<(), Error> {
        let mut i = 0;
        while i < program.0.len() {
            let instruction = program.0[i].clone();
            match instruction {
                Instruction::Fd { name, args, skip } => {
                    self.functions
                        .insert(name.clone(), Function::UserDefined(i, args, skip));
                    i += skip as usize;
                }
                Instruction::Mov { register, value } => self.value_to_register(register, value)?,
                Instruction::Gg { string, register } => {
                    let value = self
                        .globals
                        .get(&string)
                        .ok_or(Error::from(RuntimeError::GlobalNotFound {
                            name: string.clone(),
                        }))?
                        .clone();
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    registers.set(register as usize, value)?;
                }
                Instruction::Sg { string, register } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let value = registers.get(register as usize)?;
                    self.globals.insert(string, value);
                }
                Instruction::Css { string, register } => {
                    let string_size = (string.len() as f64 / 8f64).ceil() as usize;
                    let address = self.allocator.borrow_mut().malloc(string_size)?;
                    self.memory.copy_u8_vector(string.as_bytes(), address);
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    registers.set(register as usize, address as u64)?;
                }
                Instruction::Ld8 { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    registers.set(
                        register as usize,
                        match value {
                            Value::Register(source) => {
                                (registers.get(source as usize)? as u8) as u64
                            }
                            Value::Constant(value) => value,
                        },
                    )?;
                }
                Instruction::Ld16 { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    registers.set(
                        register as usize,
                        match value {
                            Value::Register(source) => {
                                (registers.get(source as usize)? as u16) as u64
                            }
                            Value::Constant(value) => value,
                        },
                    )?;
                }
                Instruction::Ld32 { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    registers.set(
                        register as usize,
                        match value {
                            Value::Register(source) => {
                                (registers.get(source as usize)? as u32) as u64
                            }
                            Value::Constant(value) => value,
                        },
                    )?;
                }
                Instruction::Ld64 { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    registers.set(
                        register as usize,
                        match value {
                            Value::Register(source) => registers.get(source as usize)?,
                            Value::Constant(value) => value,
                        },
                    )?;
                }
                Instruction::St8 {
                    register,
                    value: address_value,
                } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let value = registers.get(register as usize)? as u8;
                    let address = match address_value {
                        Value::Constant(a) => a as usize,
                        Value::Register(r) => registers.get(r as usize)? as usize,
                    };
                    self.memory.copy_u8(value, address);
                }
                Instruction::St16 {
                    register,
                    value: address_value,
                } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let value = registers.get(register as usize)? as u16;
                    let address = match address_value {
                        Value::Constant(a) => a as usize,
                        Value::Register(r) => registers.get(r as usize)? as usize,
                    };
                    self.memory.copy_u16(value, address);
                }
                Instruction::St32 {
                    register,
                    value: address_value,
                } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let value = registers.get(register as usize)? as u32;
                    let address = match address_value {
                        Value::Constant(a) => a as usize,
                        Value::Register(r) => registers.get(r as usize)? as usize,
                    };
                    self.memory.copy_u32(value, address);
                }
                Instruction::St64 {
                    register,
                    value: address_value,
                } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let value = registers.get(register as usize)? as u64;
                    let address = match address_value {
                        Value::Constant(a) => a as usize,
                        Value::Register(r) => registers.get(r as usize)? as usize,
                    };
                    self.memory.copy_u64(value, address);
                }
                Instruction::Lea { destiny, source } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let effective_address = registers.address + source as usize;
                    registers.set(destiny as usize, effective_address as u64)?;
                }
                Instruction::Iadd { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_i64(register as usize)?;
                    let new_value = destiny_value.wrapping_add(match value {
                        Value::Register(s) => registers.get_i64(s as usize)?,
                        Value::Constant(v) => v as i64,
                    });
                    registers.set_i64(register as usize, new_value);
                }
                Instruction::Isub { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_i64(register as usize)?;
                    let new_value = destiny_value.wrapping_sub(match value {
                        Value::Register(s) => registers.get_i64(s as usize)?,
                        Value::Constant(v) => v as i64,
                    });
                    registers.set_i64(register as usize, new_value);
                }
                Instruction::Smul { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_i64(register as usize)?;
                    let new_value = destiny_value.wrapping_mul(match value {
                        Value::Register(s) => registers.get_i64(s as usize)?,
                        Value::Constant(v) => v as i64,
                    });
                    registers.set_i64(register as usize, new_value);
                }
                Instruction::Umul { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get(register as usize)?;
                    let new_value = destiny_value.wrapping_mul(match value {
                        Value::Register(s) => registers.get(s as usize)?,
                        Value::Constant(v) => v,
                    });
                    registers.set(register as usize, new_value)?;
                }
                Instruction::Srem { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_i64(register as usize)?;
                    let new_value = destiny_value.wrapping_rem(match value {
                        Value::Register(s) => registers.get_i64(s as usize)?,
                        Value::Constant(v) => v as i64,
                    });
                    registers.set_i64(register as usize, new_value);
                }
                Instruction::Urem { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get(register as usize)?;
                    let new_value = destiny_value.wrapping_rem(match value {
                        Value::Register(s) => registers.get(s as usize)?,
                        Value::Constant(v) => v,
                    });
                    registers.set(register as usize, new_value)?;
                }
                Instruction::Sdiv { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_i64(register as usize)?;
                    let new_value = destiny_value.wrapping_div(match value {
                        Value::Register(s) => registers.get_i64(s as usize)?,
                        Value::Constant(v) => v as i64,
                    });
                    registers.set_i64(register as usize, new_value);
                }
                Instruction::Udiv { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get(register as usize)?;
                    let new_value = destiny_value.wrapping_div(match value {
                        Value::Register(s) => registers.get(s as usize)?,
                        Value::Constant(v) => v,
                    });
                    registers.set(register as usize, new_value)?;
                }
                Instruction::And { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get(register as usize)?;
                    let new_value = destiny_value
                        & (match value {
                            Value::Register(s) => registers.get(s as usize)?,
                            Value::Constant(v) => v,
                        });
                    registers.set(register as usize, new_value)?;
                }
                Instruction::Or { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get(register as usize)?;
                    let new_value = destiny_value
                        | (match value {
                            Value::Register(s) => registers.get(s as usize)?,
                            Value::Constant(v) => v,
                        });
                    registers.set(register as usize, new_value)?;
                }
                Instruction::Xor { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get(register as usize)?;
                    let new_value = destiny_value
                        ^ (match value {
                            Value::Register(s) => registers.get(s as usize)?,
                            Value::Constant(v) => v,
                        });
                    registers.set(register as usize, new_value)?;
                }
                Instruction::Shl { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get(register as usize)?;
                    let new_value = destiny_value.wrapping_shl(match value {
                        Value::Register(s) => registers.get(s as usize)?,
                        Value::Constant(v) => v,
                    } as u32);
                    registers.set(register as usize, new_value)?;
                }
                Instruction::Ashr { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get(register as usize)?;
                    let new_value = destiny_value.wrapping_shr(match value {
                        Value::Register(s) => registers.get(s as usize)?,
                        Value::Constant(v) => v,
                    } as u32);
                    registers.set(register as usize, new_value)?;
                }
                Instruction::Lshr { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_u32(register as usize)?;
                    let new_value = destiny_value.wrapping_shr(match value {
                        Value::Register(s) => registers.get(s as usize)?,
                        Value::Constant(v) => v,
                    } as u32);
                    registers.set_u32(register as usize, new_value);
                }
                Instruction::Ineg { register } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    registers.set(register as usize, !registers.get(register as usize)?)?;
                }
                Instruction::Fadd { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_f64(register as usize)?;
                    let new_value = destiny_value
                        + (match value {
                            Value::Register(s) => registers.get_f64(s as usize)?,
                            Value::Constant(v) => v as f64,
                        });
                    registers.set_f64(register as usize, new_value);
                }
                Instruction::Fsub { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_f64(register as usize)?;
                    let new_value = destiny_value
                        - (match value {
                            Value::Register(s) => registers.get_f64(s as usize)?,
                            Value::Constant(v) => v as f64,
                        });
                    registers.set_f64(register as usize, new_value);
                }
                Instruction::Fmul { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_f64(register as usize)?;
                    let new_value = destiny_value
                        * (match value {
                            Value::Register(s) => registers.get_f64(s as usize)?,
                            Value::Constant(v) => v as f64,
                        });
                    registers.set_f64(register as usize, new_value);
                }
                Instruction::Frem { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_f64(register as usize)?;
                    let new_value = destiny_value.rem(match value {
                        Value::Register(s) => registers.get_f64(s as usize)?,
                        Value::Constant(v) => v as f64,
                    });
                    registers.set_f64(register as usize, new_value);
                }
                Instruction::Fdiv { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_f64(register as usize)?;
                    let new_value = destiny_value
                        / (match value {
                            Value::Register(s) => registers.get_f64(s as usize)?,
                            Value::Constant(v) => v as f64,
                        });
                    registers.set_f64(register as usize, new_value);
                }
                Instruction::Eq { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get(register as usize)?;
                    let new_value = destiny_value
                        == (match value {
                            Value::Register(s) => registers.get(s as usize)?,
                            Value::Constant(v) => v,
                        });
                    registers.set(register as usize, new_value as u64)?;
                }
                Instruction::Ne { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get(register as usize)?;
                    let new_value = destiny_value
                        != (match value {
                            Value::Register(s) => registers.get(s as usize)?,
                            Value::Constant(v) => v,
                        });
                    registers.set(register as usize, new_value as u64)?;
                }
                Instruction::Ult { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get(register as usize)?;
                    let new_value = destiny_value
                        < (match value {
                            Value::Register(s) => registers.get(s as usize)?,
                            Value::Constant(v) => v,
                        });
                    registers.set(register as usize, new_value as u64)?;
                }
                Instruction::Ule { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get(register as usize)?;
                    let new_value = destiny_value
                        <= (match value {
                            Value::Register(s) => registers.get(s as usize)?,
                            Value::Constant(v) => v,
                        });
                    registers.set(register as usize, new_value as u64)?;
                }
                Instruction::Ugt { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get(register as usize)?;
                    let new_value = destiny_value
                        > (match value {
                            Value::Register(s) => registers.get(s as usize)?,
                            Value::Constant(v) => v,
                        });
                    registers.set(register as usize, new_value as u64)?;
                }
                Instruction::Uge { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get(register as usize)?;
                    let new_value = destiny_value
                        >= (match value {
                            Value::Register(s) => registers.get(s as usize)?,
                            Value::Constant(v) => v,
                        });
                    registers.set(register as usize, new_value as u64)?;
                }
                Instruction::Slt { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_i64(register as usize)?;
                    let new_value = destiny_value
                        < (match value {
                            Value::Register(s) => registers.get_i64(s as usize)?,
                            Value::Constant(v) => v as i64,
                        });
                    registers.set_i64(register as usize, new_value as i64);
                }
                Instruction::Sle { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_i64(register as usize)?;
                    let new_value = destiny_value
                        <= (match value {
                            Value::Register(s) => registers.get_i64(s as usize)?,
                            Value::Constant(v) => v as i64,
                        });
                    registers.set_i64(register as usize, new_value as i64);
                }
                Instruction::Sgt { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_i64(register as usize)?;
                    let new_value = destiny_value
                        > (match value {
                            Value::Register(s) => registers.get_i64(s as usize)?,
                            Value::Constant(v) => v as i64,
                        });
                    registers.set_i64(register as usize, new_value as i64);
                }
                Instruction::Sge { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_i64(register as usize)?;
                    let new_value = destiny_value
                        >= (match value {
                            Value::Register(s) => registers.get_i64(s as usize)?,
                            Value::Constant(v) => v as i64,
                        });
                    registers.set_i64(register as usize, new_value as i64);
                }
                Instruction::Feq { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_f64(register as usize)?;
                    let new_value = destiny_value
                        == (match value {
                            Value::Register(s) => registers.get_f64(s as usize)?,
                            Value::Constant(v) => v as f64,
                        });
                    registers.set_f64(register as usize, new_value as i64 as f64);
                }
                Instruction::Fne { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_f64(register as usize)?;
                    let new_value = destiny_value
                        != (match value {
                            Value::Register(s) => registers.get_f64(s as usize)?,
                            Value::Constant(v) => v as f64,
                        });
                    registers.set_f64(register as usize, new_value as i64 as f64);
                }
                Instruction::Flt { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_f64(register as usize)?;
                    let new_value = destiny_value
                        < (match value {
                            Value::Register(s) => registers.get_f64(s as usize)?,
                            Value::Constant(v) => v as f64,
                        });
                    registers.set_f64(register as usize, new_value as i64 as f64);
                }
                Instruction::Fle { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_f64(register as usize)?;
                    let new_value = destiny_value
                        <= (match value {
                            Value::Register(s) => registers.get_f64(s as usize)?,
                            Value::Constant(v) => v as f64,
                        });
                    registers.set_f64(register as usize, new_value as i64 as f64);
                }
                Instruction::Fgt { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_f64(register as usize)?;
                    let new_value = destiny_value
                        > (match value {
                            Value::Register(s) => registers.get_f64(s as usize)?,
                            Value::Constant(v) => v as f64,
                        });
                    registers.set_f64(register as usize, new_value as i64 as f64);
                }
                Instruction::Fge { register, value } => {
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let destiny_value = registers.get_f64(register as usize)?;
                    let new_value = destiny_value
                        >= (match value {
                            Value::Register(s) => registers.get_f64(s as usize)?,
                            Value::Constant(v) => v as f64,
                        });
                    registers.set_f64(register as usize, new_value as i64 as f64);
                }
                Instruction::Jmp { offset } => {
                    i = ((i as i64) + offset - 1) as usize;
                }
                Instruction::Jnz { offset, register } => {
                    let rc = self.register_stack.borrow();
                    let registers = rc.last().unwrap();
                    if registers.get(register as usize)? != 0 {
                        i = ((i as i64) + offset - 1) as usize;
                    }
                }
                Instruction::Jz { offset, register } => {
                    let rc = self.register_stack.borrow();
                    let registers = rc.last().unwrap();
                    if registers.get(register as usize)? == 0 {
                        i = ((i as i64) + offset - 1) as usize;
                    }
                }
                Instruction::Call {
                    return_register,
                    arguments,
                } => {
                    let native_functions = NativeFunctions {
                        allocator: self.allocator.clone(),
                        register_stack: self.register_stack.clone(),
                    };
                    let rc = self.register_stack.borrow();
                    let function = {
                        let registers = rc.last().unwrap();
                        registers.get_t(return_register as usize)?
                    };
                    match function {
                        Function::Native(f) => {
                            let r = f(
                                &native_functions,
                                arguments
                                    .to_vec()
                                    .iter()
                                    .filter(|v| v.is_some())
                                    .map(|v| v.unwrap())
                                    .collect(),
                            )?;
                            let mut rc = self.register_stack.borrow_mut();
                            let registers = rc.last_mut().unwrap();
                            registers.set(return_register as usize, r)?;
                        }
                        Function::UserDefined(new_i, _nargs, _skip) => {
                            let new_i = new_i.clone();
                            self.call_stack.push((i, return_register));
                            i = new_i;
                            let new_register_set = self.create_new_register_set()?;
                            self.register_stack.borrow_mut().push(new_register_set);
                        }
                    };
                }
                Instruction::Ret { value } => {
                    let (new_i, r) = self
                        .call_stack
                        .pop()
                        .ok_or(Error::from(RuntimeError::ReturnOnNoFunction))?;
                    let mut rc = self.register_stack.borrow_mut();
                    let registers = rc.last_mut().unwrap();
                    let ret_value = match value {
                        Value::Constant(v) => v,
                        Value::Register(r) => registers.get(r as usize)?,
                    };
                    self.register_stack.borrow_mut().pop();
                    registers.set(r as usize, ret_value)?;
                    i = new_i;
                }
                Instruction::Leave => {
                    self.register_stack.borrow_mut().pop();
                    let (new_i, _) = self
                        .call_stack
                        .pop()
                        .ok_or(Error::from(RuntimeError::ReturnOnNoFunction))?;
                    i = new_i;
                }
                _ => panic!("Not implemented yet"),
            }
            i += 1;
        }
        Ok(())
    }

    fn value_to_register(&mut self, register: u8, value: Value) -> Result<(), Error> {
        let mut rc = self.register_stack.borrow_mut();
        let registers = rc.last_mut().unwrap();
        match value {
            Value::Constant(v) => {
                registers.set(register as usize, v)?;
            }
            Value::Register(source) => {
                let source_value = registers.get(source as usize)?;
                registers.set(register as usize, source_value)?;
            }
        };
        Ok(())
    }

    fn create_new_register_set(&self) -> Result<RegisterSet, Error> {
        let address = self.allocator.borrow_mut().malloc(256)?;
        Ok(RegisterSet {
            address,
            memory: self.memory.clone(),
            size: 256,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_should_add_a_new_function_on_fd() {
        let instructions = vec![
            Instruction::Fd {
                name: "test".to_owned(),
                args: 0,
                skip: 1,
            },
            Instruction::Leave,
        ];
        let program = Program(instructions);
        let mut cpu = Cpu::new(260).unwrap();
        cpu.execute(program).unwrap();
        let test = cpu.functions.get("test").unwrap();
        match test {
            Function::UserDefined(ref start, ref args, ref skip) => {
                assert_eq!(*start, 0);
                assert_eq!(*args, 0);
                assert_eq!(*skip, 1);
            }
            _ => panic!("Saved function should be user defined"),
        }
    }

    #[test]
    fn it_should_add_a_constant_to_a_register() {
        let instructions = vec![Instruction::Mov {
            register: 0,
            value: Value::Constant(42),
        }];
        let program = Program(instructions);
        let mut cpu = Cpu::new(260).unwrap();
        cpu.execute(program).unwrap();
        let mut rc = cpu.register_stack.borrow_mut();
        let registers = rc.last_mut().unwrap();
        assert_eq!(registers.get(0).unwrap(), 42);
    }

    #[test]
    fn it_should_add_a_register_to_a_register() {
        let instructions = vec![Instruction::Mov {
            register: 0,
            value: Value::Register(1),
        }];
        let program = Program(instructions);
        let mut cpu = Cpu::new(260).unwrap();
        {
            let mut rc = cpu.register_stack.borrow_mut();
            let registers = rc.last_mut().unwrap();
            registers.set(1, 42).unwrap();
        }
        cpu.execute(program).unwrap();
        let mut rc = cpu.register_stack.borrow_mut();
        let registers = rc.last_mut().unwrap();
        assert_eq!(registers.get(0).unwrap(), 42);
    }

    #[test]
    fn it_should_copy_a_global_to_a_register() {
        let instructions = vec![Instruction::Gg {
            string: "test".to_owned(),
            register: 0,
        }];
        let program = Program(instructions);
        let mut cpu = Cpu::new(260).unwrap();
        cpu.globals.insert("test".to_owned(), 42);
        cpu.execute(program).unwrap();
        let mut rc = cpu.register_stack.borrow_mut();
        let registers = rc.last_mut().unwrap();
        assert_eq!(registers.get(0).unwrap(), 42);
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: GlobalNotFound { name: \"test\" }"
    )]
    fn it_should_panic_when_copying_from_an_unexisting_global() {
        let instructions = vec![Instruction::Gg {
            string: "test".to_owned(),
            register: 0,
        }];
        let program = Program(instructions);
        let mut cpu = Cpu::new(260).unwrap();
        cpu.execute(program).unwrap();
    }

    #[test]
    fn it_should_copy_a_register_to_a_global() {
        let instructions = vec![Instruction::Sg {
            string: "test".to_owned(),
            register: 0,
        }];
        let program = Program(instructions);
        let mut cpu = Cpu::new(260).unwrap();
        {
            let mut rc = cpu.register_stack.borrow_mut();
            let registers = rc.last_mut().unwrap();
            registers.set(0, 42).unwrap();
        }
        cpu.execute(program).unwrap();
        let global = cpu.globals.get("test").unwrap().clone();
        assert_eq!(global, 42);
    }

    #[test]
    fn it_should_load_a_u8_into_a_register() {
        let instructions = vec![Instruction::Ld8 {
            register: 0,
            value: Value::Constant(42),
        }];
        let program = Program(instructions);
        let mut cpu = Cpu::new(260).unwrap();
        cpu.execute(program).unwrap();
        let mut rc = cpu.register_stack.borrow_mut();
        let registers = rc.last_mut().unwrap();
        assert_eq!(registers.get(0).unwrap(), 42);
    }

    #[test]
    fn it_should_load_a_u16_into_a_register() {
        let instructions = vec![Instruction::Ld16 {
            register: 0,
            value: Value::Constant(42),
        }];
        let program = Program(instructions);
        let mut cpu = Cpu::new(260).unwrap();
        cpu.execute(program).unwrap();
        let mut rc = cpu.register_stack.borrow_mut();
        let registers = rc.last_mut().unwrap();
        assert_eq!(registers.get(0).unwrap(), 42);
    }

    #[test]
    fn it_should_load_a_u32_into_a_register() {
        let instructions = vec![Instruction::Ld32 {
            register: 0,
            value: Value::Constant(42),
        }];
        let program = Program(instructions);
        let mut cpu = Cpu::new(260).unwrap();
        cpu.execute(program).unwrap();
        let mut rc = cpu.register_stack.borrow_mut();
        let registers = rc.last_mut().unwrap();
        assert_eq!(registers.get(0).unwrap(), 42);
    }

    #[test]
    fn it_should_load_a_u64_into_a_register() {
        let instructions = vec![Instruction::Ld64 {
            register: 0,
            value: Value::Constant(42),
        }];
        let program = Program(instructions);
        let mut cpu = Cpu::new(260).unwrap();
        cpu.execute(program).unwrap();
        let mut rc = cpu.register_stack.borrow_mut();
        let registers = rc.last_mut().unwrap();
        assert_eq!(registers.get(0).unwrap(), 42);
    }
}
