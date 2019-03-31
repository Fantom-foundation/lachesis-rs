use failure::Error;
use llvm_sys::core::{
    LLVMCreateBuilder, LLVMDisposeBuilder, LLVMDisposeModule, LLVMModuleCreateWithName,
};
use llvm_sys::prelude::*;
use llvm_sys::{LLVMBuilder, LLVMModule};
use lunarity::ast::{Expression, Program, Statement};
use std::collections::HashMap;
use std::ffi::CString;

struct Module {
    module: *mut LLVMModule,
    strings: Vec<CString>,
}

impl Module {
    fn new(module_name: &str) -> Result<Module, Error> {
        let c_module_name = CString::new(module_name)?;
        Ok(Module {
            module: unsafe {
                LLVMModuleCreateWithName(c_module_name.to_bytes_with_nul().as_ptr() as *const _)
            },
            strings: vec![c_module_name],
        })
    }
    fn new_string_ptr(&mut self, s: &str) -> *const i8 {
        self.new_mut_string_ptr(s)
    }

    fn new_mut_string_ptr(&mut self, s: &str) -> *mut i8 {
        let cstring = CString::new(s).unwrap();
        let ptr = cstring.as_ptr() as *mut _;
        self.strings.push(cstring);
        ptr
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeModule(self.module);
        }
    }
}

struct Builder {
    builder: *mut LLVMBuilder,
}

impl Builder {
    fn new() -> Builder {
        unsafe {
            Builder {
                builder: LLVMCreateBuilder(),
            }
        }
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe { LLVMDisposeBuilder(self.builder) }
    }
}

#[derive(Debug, Fail)]
pub enum CodeGenerationError {
    #[fail(display = "Feature not implemented yet")]
    NotImplementedYet,
}

pub struct Context {
    module: Module,
    builder: Builder,
    symbols: HashMap<String, LLVMValueRef>,
}

impl Context {
    pub fn new(name: &str) -> Result<Context, Error> {
        Ok(Context {
            module: Module::new(name)?,
            builder: Builder::new(),
            symbols: HashMap::new(),
        })
    }
}

pub trait CodeGenerator {
    fn codegen(&self, context: &mut Context) -> Result<LLVMValueRef, CodeGenerationError>;
}

impl<'a> CodeGenerator for Expression<'a> {
    fn codegen(&self, context: &mut Context) -> Result<LLVMValueRef, CodeGenerationError> {
        Err(CodeGenerationError::NotImplementedYet)
    }
}

impl<'a> CodeGenerator for Statement<'a> {
    fn codegen(&self, context: &mut Context) -> Result<LLVMValueRef, CodeGenerationError> {
        Err(CodeGenerationError::NotImplementedYet)
    }
}

impl<'a> CodeGenerator for Program<'a> {
    fn codegen(&self, context: &mut Context) -> Result<LLVMValueRef, CodeGenerationError> {
        Err(CodeGenerationError::NotImplementedYet)
    }
}
