use failure::Error;
use llvm_sys::core::{
    LLVMBuildGlobalStringPtr, LLVMConstInt, LLVMConstPointerNull, LLVMConstStructInContext,
    LLVMContextCreate, LLVMContextDispose, LLVMCreateBuilderInContext, LLVMDisposeBuilder,
    LLVMDisposeModule, LLVMIntTypeInContext, LLVMModuleCreateWithNameInContext, LLVMPointerType,
    LLVMStructCreateNamed, LLVMStructSetBody,
};
use llvm_sys::prelude::*;
use llvm_sys::{LLVMBuilder, LLVMModule};
use lunarity::ast::{
    ContractPart, ElementaryTypeName, Expression, Primitive, Program, SourceUnit,
    StateVariableDeclaration, Statement, TypeName,
};
use std::collections::HashMap;
use std::ffi::CString;
use std::str::FromStr;

const LLVM_FALSE: LLVMBool = 0 as LLVMBool;
const LLVM_TRUE: LLVMBool = 1 as LLVMBool;

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
    #[fail(display = "Number parsing error {}", 0)]
    NumberParsingError(String),
    #[fail(display = "(Un)Fixed point numbers are not a stable feature")]
    FixedPointNumbersNotStable,
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
    pub fn print_to_file(&self, _file: &str) -> Result<(), Vec<String>> {
        Ok(())
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
    fn typegen(&self, context: &mut Context) -> Result<Option<LLVMTypeRef>, CodeGenerationError>;
}

impl<'a> CodeGenerator for Expression<'a> {
    fn codegen(&self, context: &mut Context) -> Result<Option<LLVMValueRef>, CodeGenerationError> {
        match self {
            Expression::PrimitiveExpression(p) => match p {
                Primitive::String(s) => Ok(Some(unsafe {
                    LLVMBuildGlobalStringPtr(
                        context.builder.builder,
                        context.module.new_string_ptr(s),
                        context.module.new_string_ptr("tempstring"),
                    )
                })),
                Primitive::HexNumber(s) => Ok(Some(unsafe {
                    let value = usize::from_str(s).map_err(|_| {
                        CodeGenerationError::NumberParsingError(s.to_owned().to_owned())
                    })?;
                    let bits = find_int_size_in_bits(value);
                    let t = uint(context, bits as u32);
                    LLVMConstInt(t, value as u64, LLVM_FALSE)
                })),
                Primitive::Bool(b) => Ok(Some(unsafe {
                    LLVMConstInt(uint(context, 1), *b as _, LLVM_FALSE)
                })),
                _ => Err(CodeGenerationError::NotImplementedYet),
            },
            _ => Err(CodeGenerationError::NotImplementedYet),
        }
    }
}

impl<'a> CodeGenerator for Statement<'a> {
    fn codegen(&self, _context: &mut Context) -> Result<Option<LLVMValueRef>, CodeGenerationError> {
        Err(CodeGenerationError::NotImplementedYet)
    }
}

fn type_from_type_name(
    type_name: TypeName,
    context: &mut Context,
) -> Result<LLVMTypeRef, CodeGenerationError> {
    match type_name {
        TypeName::ElementaryTypeName(e) => match e {
            ElementaryTypeName::String => Ok(unsafe { LLVMPointerType(uint(context, 8), 0) }),
            ElementaryTypeName::Address => Ok(uint(context, 8 * 20)),
            ElementaryTypeName::Bool => Ok(uint(context, 1)),
            ElementaryTypeName::Byte(b) => Ok(uint(context, b as u32 * 8)),
            ElementaryTypeName::Uint(b) => Ok(uint(context, b as u32 * 8)),
            ElementaryTypeName::Int(b) => Ok(uint(context, b as u32 * 8)),
            ElementaryTypeName::Fixed(_, _) | ElementaryTypeName::Ufixed(_, _) => {
                Err(CodeGenerationError::FixedPointNumbersNotStable)
            }
            ElementaryTypeName::Bytes => Ok(unsafe { LLVMPointerType(uint(context, 8), 0) }),
        },
        _ => panic!("Not implemented yet"),
    }
}

impl<'a> TypeGenerator for ContractPart<'a> {
    fn typegen(&self, context: &mut Context) -> Result<Option<LLVMTypeRef>, CodeGenerationError> {
        match self {
            ContractPart::EnumDefinition(e) => {
                let mut counter = 0;
                let s = find_int_size_in_bits(e.variants.iter().collect::<Vec<_>>().len());
                let t = uint(context, s as u32);
                for member in e.variants {
                    let member_symbol = format!("{}_{}", e.name.value, member.value);
                    context.symbols.insert(member_symbol, unsafe {
                        LLVMConstInt(t, counter as u64, LLVM_FALSE)
                    });
                    counter += 1;
                }
                Ok(None)
            }
            ContractPart::StateVariableDeclaration(s) => {
                Ok(Some(type_from_type_name(s.type_name.value, context)?))
            }
            _ => Err(CodeGenerationError::NotImplementedYet),
        }
    }
}

impl<'a> CodeGenerator for StateVariableDeclaration<'a> {
    fn codegen(&self, context: &mut Context) -> Result<Option<LLVMValueRef>, CodeGenerationError> {
        match self.type_name.value {
            TypeName::ElementaryTypeName(e) => match e {
                ElementaryTypeName::String => self
                    .init
                    .map(|v| v.value.codegen(context))
                    .unwrap_or(Ok(Some(unsafe {
                        LLVMBuildGlobalStringPtr(
                            context.builder.builder,
                            context.module.new_string_ptr(""),
                            context.module.new_string_ptr("tempstr"),
                        )
                    }))),
                ElementaryTypeName::Address => self
                    .init
                    .map(|v| v.value.codegen(context))
                    .unwrap_or(Ok(Some(unsafe {
                        LLVMConstInt(uint(context, 20), 0, LLVM_FALSE)
                    }))),
                ElementaryTypeName::Bool => {
                    self.init
                        .map(|v| v.value.codegen(context))
                        .unwrap_or(Ok(Some(unsafe {
                            LLVMConstInt(uint(context, 1), 0, LLVM_FALSE)
                        })))
                }
                ElementaryTypeName::Byte(b) => self
                    .init
                    .map(|v| v.value.codegen(context))
                    .unwrap_or(Ok(Some(unsafe {
                        LLVMConstInt(uint(context, b as u32 * 8), 0, LLVM_FALSE)
                    }))),
                ElementaryTypeName::Uint(b) => self
                    .init
                    .map(|v| v.value.codegen(context))
                    .unwrap_or(Ok(Some(unsafe {
                        LLVMConstInt(uint(context, b as u32 * 8), 0, LLVM_FALSE)
                    }))),
                ElementaryTypeName::Int(b) => self
                    .init
                    .map(|v| v.value.codegen(context))
                    .unwrap_or(Ok(Some(unsafe {
                        LLVMConstInt(uint(context, b as u32 * 8), 0, LLVM_TRUE)
                    }))),
                ElementaryTypeName::Fixed(_, _) | ElementaryTypeName::Ufixed(_, _) => {
                    Err(CodeGenerationError::FixedPointNumbersNotStable)
                }
                ElementaryTypeName::Bytes => self
                    .init
                    .map(|v| v.value.codegen(context))
                    .unwrap_or(Ok(Some(unsafe { LLVMConstPointerNull(uint(context, 8)) }))),
            },
            _ => panic!("Not implemented yet"),
        }
    }
}

impl<'a> CodeGenerator for ContractPart<'a> {
    fn codegen(&self, _context: &mut Context) -> Result<Option<LLVMValueRef>, CodeGenerationError> {
        match self {
            ContractPart::EnumDefinition(_) => Ok(None),
            _ => Err(CodeGenerationError::NotImplementedYet),
        }
    }
}

impl<'a> CodeGenerator for Program<'a> {
    fn codegen(&self, context: &mut Context) -> Result<Option<LLVMValueRef>, CodeGenerationError> {
        for s in self.body() {
            match s.value {
                SourceUnit::ContractDefinition(c) => {
                    context.symbols.clear();
                    let struct_type = unsafe {
                        LLVMStructCreateNamed(
                            context.context,
                            context.module.new_string_ptr(c.name.value),
                        )
                    };
                    let mut types = Vec::new();
                    let mut vals = Vec::new();
                    for t in c.body {
                        let element_type = t.value.typegen(context)?;
                        if let Some(et) = element_type {
                            types.push(et);
                            vals.push(t.value.codegen(context)?.unwrap());
                        }
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
                SourceUnit::ImportDirective(_) => Err(CodeGenerationError::NotImplementedYet)?,
                SourceUnit::PragmaDirective(_) => Err(CodeGenerationError::NotImplementedYet)?,
            }
        }
        Ok(None)
    }
}

#[inline]
fn uint(context: &Context, bits: u32) -> LLVMTypeRef {
    unsafe { LLVMIntTypeInContext(context.context, bits) }
}

#[inline]
fn find_int_size_in_bits(number: usize) -> usize {
    let mut start = 8;
    while 2usize.pow(start as u32) < number {
        start += 8;
    }
    start
}
