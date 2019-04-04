use failure::Error;
use llvm_sys::core::{
    LLVMConstStructInContext, LLVMContextCreate, LLVMContextDispose, LLVMCreateBuilderInContext,
    LLVMDisposeBuilder, LLVMDisposeModule, LLVMModuleCreateWithNameInContext,
    LLVMStructCreateNamed, LLVMStructSetBody,
};
use llvm_sys::prelude::*;
use llvm_sys::{LLVMBuilder, LLVMModule};
use lunarity::ast::{ContractPart, Expression, Program, SourceUnit, Statement};
use std::collections::HashMap;
use std::ffi::CString;

struct Module {
    module: *mut LLVMModule,
    strings: Vec<CString>,
}

impl Module {
    fn new(module_name: &str, context: LLVMContextRef) -> Result<Module, Error> {
        let c_module_name = CString::new(module_name)?;
        Ok(Module {
            module: unsafe {
                LLVMModuleCreateWithNameInContext(
                    c_module_name.to_bytes_with_nul().as_ptr() as *const _,
                    context,
                )
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
    fn new(context: LLVMContextRef) -> Builder {
        unsafe {
            Builder {
                builder: LLVMCreateBuilderInContext(context),
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
    context: LLVMContextRef,
    module: Module,
    builder: Builder,
    symbols: HashMap<String, LLVMValueRef>,
}

impl Context {
    pub fn new(name: &str) -> Result<Context, Error> {
        let context = unsafe { LLVMContextCreate() };
        Ok(Context {
            context,
            module: Module::new(name, context)?,
            builder: Builder::new(context),
            symbols: HashMap::new(),
        })
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { LLVMContextDispose(self.context) };
    }
}

pub trait CodeGenerator {
    fn codegen(&self, context: &mut Context) -> Result<Option<LLVMValueRef>, CodeGenerationError>;
}

pub trait TypeGenerator {
    fn typegen(&self, context: &mut Context) -> Result<LLVMTypeRef, CodeGenerationError>;
}

impl<'a> CodeGenerator for Expression<'a> {
    fn codegen(&self, context: &mut Context) -> Result<Option<LLVMValueRef>, CodeGenerationError> {
        Err(CodeGenerationError::NotImplementedYet)
    }
}

impl<'a> CodeGenerator for Statement<'a> {
    fn codegen(&self, context: &mut Context) -> Result<Option<LLVMValueRef>, CodeGenerationError> {
        Err(CodeGenerationError::NotImplementedYet)
    }
}

impl<'a> TypeGenerator for ContractPart<'a> {
    fn typegen(&self, context: &mut Context) -> Result<LLVMTypeRef, CodeGenerationError> {
        match self {
            _ => Err(CodeGenerationError::NotImplementedYet),
        }
    }
}

impl<'a> CodeGenerator for ContractPart<'a> {
    fn codegen(&self, context: &mut Context) -> Result<Option<LLVMValueRef>, CodeGenerationError> {
        match self {
            _ => Err(CodeGenerationError::NotImplementedYet),
        }
    }
}

impl<'a> CodeGenerator for Program<'a> {
    fn codegen(&self, context: &mut Context) -> Result<Option<LLVMValueRef>, CodeGenerationError> {
        for s in self.body() {
            match s.value {
                SourceUnit::ContractDefinition(c) => {
                    let struct_type = unsafe {
                        LLVMStructCreateNamed(
                            context.context,
                            context.module.new_mut_string_ptr(c.name.value),
                        )
                    };
                    let mut types = Vec::new();
                    let mut vals = Vec::new();
                    for t in c.body {
                        types.push(t.value.typegen(context)?);
                        vals.push(t.value.codegen(context)?.unwrap());
                    }
                    unsafe {
                        LLVMStructSetBody(
                            struct_type,
                            types.as_mut_ptr(),
                            types.len() as u32,
                            1 as LLVMBool,
                        )
                    };
                    let contract = unsafe {
                        LLVMConstStructInContext(
                            context.context,
                            vals.as_mut_ptr(),
                            vals.len() as u32,
                            1 as LLVMBool,
                        )
                    };
                    context.symbols.insert(c.name.value.to_owned(), contract);
                }
                SourceUnit::ImportDirective(i) => Err(CodeGenerationError::NotImplementedYet)?,
                SourceUnit::PragmaDirective(p) => Err(CodeGenerationError::NotImplementedYet)?,
            }
        }
        Ok(None)
    }
}
