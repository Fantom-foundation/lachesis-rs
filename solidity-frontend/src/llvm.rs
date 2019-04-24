use crate::parser::*;
use failure::Error;
use llvm_sys::core::{
    LLVMBuildGlobalStringPtr, LLVMConstInt, LLVMConstStructInContext, LLVMContextCreate,
    LLVMContextDispose, LLVMCreateBuilderInContext, LLVMDisposeBuilder, LLVMDisposeModule,
    LLVMIntTypeInContext, LLVMModuleCreateWithNameInContext, LLVMPointerType,
    LLVMStructCreateNamed, LLVMStructSetBody,
};
use llvm_sys::prelude::*;
use llvm_sys::{LLVMBuilder, LLVMModule};
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
    #[fail(display = "User defined type {} not found", 0)]
    UserDefinedTypeNotFound(String),
    #[fail(display = "User defined type {} has no default value", 0)]
    UserDefinedTypeHasNoDefault(String),
}

pub struct Context {
    context: LLVMContextRef,
    module: Module,
    builder: Builder,
    symbols: HashMap<String, LLVMValueRef>,
    type_symbols: HashMap<String, LLVMTypeRef>,
}

impl Context {
    pub fn new(name: &str) -> Result<Context, Error> {
        let context = unsafe { LLVMContextCreate() };
        Ok(Context {
            context,
            module: Module::new(name, context)?,
            builder: Builder::new(context),
            symbols: HashMap::new(),
            type_symbols: HashMap::new(),
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

impl<'a> CodeGenerator for Expression {
    fn codegen(&self, context: &mut Context) -> Result<Option<LLVMValueRef>, CodeGenerationError> {
        match self {
            Expression::PrimaryExpression(PrimaryExpression::Literal(l)) => match l {
                Literal::StringLiteral(s) => Ok(Some(unsafe {
                    LLVMBuildGlobalStringPtr(
                        context.builder.builder,
                        context.module.new_string_ptr(s),
                        context.module.new_string_ptr("tempstring"),
                    )
                })),
                Literal::HexLiteral(s) => Ok(Some(unsafe {
                    let value = usize::from_str(s).map_err(|_| {
                        CodeGenerationError::NumberParsingError(s.to_owned().to_owned())
                    })?;
                    let bits = find_int_size_in_bits(value);
                    let t = uint(context, bits as u32);
                    LLVMConstInt(t, value as u64, LLVM_FALSE)
                })),
                Literal::BooleanLiteral(b) => Ok(Some(unsafe {
                    LLVMConstInt(uint(context, 1), *b as _, LLVM_FALSE)
                })),
                _ => Err(CodeGenerationError::NotImplementedYet),
            },
            _ => Err(CodeGenerationError::NotImplementedYet),
        }
    }
}

impl<'a> CodeGenerator for Statement {
    fn codegen(&self, _context: &mut Context) -> Result<Option<LLVMValueRef>, CodeGenerationError> {
        Err(CodeGenerationError::NotImplementedYet)
    }
}

fn type_from_type_name(
    type_name: &TypeName,
    context: &mut Context,
) -> Result<LLVMTypeRef, CodeGenerationError> {
    match type_name {
        TypeName::ElementaryTypeName(e) => match e {
            ElementaryTypeName::String => Ok(unsafe { LLVMPointerType(uint(context, 8), 0) }),
            ElementaryTypeName::Address => Ok(uint(context, 8 * 20)),
            ElementaryTypeName::Bool => Ok(uint(context, 1)),
            ElementaryTypeName::Byte(b) => Ok(uint(context, *b as u32 * 8)),
            ElementaryTypeName::Uint(b) => Ok(uint(context, *b as u32 * 8)),
            ElementaryTypeName::Int(b) => Ok(uint(context, *b as u32 * 8)),
            ElementaryTypeName::Fixed(_, _) | ElementaryTypeName::Ufixed(_, _) => {
                Err(CodeGenerationError::FixedPointNumbersNotStable)
            }
        },
        TypeName::ArrayTypeName(_, _) => Err(CodeGenerationError::NotImplementedYet),
        TypeName::UserDefinedTypeName(user_defined_type_name) => context
            .type_symbols
            .get(user_defined_type_name.base.as_str())
            .ok_or(CodeGenerationError::UserDefinedTypeNotFound(
                user_defined_type_name.base.as_str().to_owned(),
            ))
            .map(|v| v.clone()),
        _ => panic!("Not implemented yet"),
    }
}

impl<'a> TypeGenerator for ContractPart {
    fn typegen(&self, context: &mut Context) -> Result<Option<LLVMTypeRef>, CodeGenerationError> {
        match self {
            ContractPart::EnumDefinition(e) => {
                let mut counter = 0;
                let s = find_int_size_in_bits(e.values.len());
                let t = uint(context, s as u32);
                for member in e.values.iter() {
                    let member_symbol = format!("{}_{}", e.name.as_str(), member.as_str());
                    let value = unsafe { LLVMConstInt(t, counter as u64, LLVM_FALSE) };
                    context.symbols.insert(member_symbol, value.clone());
                    if counter == 0 {
                        context
                            .symbols
                            .insert(format!("{}@default", e.name.as_str()), value.clone());
                    }
                    counter += 1;
                }
                context.type_symbols.insert(e.name.as_str().to_owned(), t);
                Ok(None)
            }
            ContractPart::StateVariableDeclaration(s) => {
                Ok(Some(type_from_type_name(&s.type_name, context)?))
            }
            ContractPart::StructDefinition(_) => Ok(None),
            _ => Err(CodeGenerationError::NotImplementedYet),
        }
    }
}

impl<'a> CodeGenerator for StateVariableDeclaration {
    fn codegen(&self, context: &mut Context) -> Result<Option<LLVMValueRef>, CodeGenerationError> {
        match &self.type_name {
            TypeName::ElementaryTypeName(e) => match e {
                ElementaryTypeName::String => {
                    if let Some(e) = &self.value {
                        e.codegen(context)
                    } else {
                        Ok(Some(unsafe {
                            LLVMBuildGlobalStringPtr(
                                context.builder.builder,
                                context.module.new_string_ptr(""),
                                context.module.new_string_ptr("tempstr"),
                            )
                        }))
                    }
                }
                ElementaryTypeName::Address => {
                    if let Some(e) = &self.value {
                        e.codegen(context)
                    } else {
                        Ok(Some(unsafe {
                            LLVMConstInt(uint(context, 20), 0, LLVM_FALSE)
                        }))
                    }
                }
                ElementaryTypeName::Bool => {
                    if let Some(e) = &self.value {
                        e.codegen(context)
                    } else {
                        Ok(Some(unsafe {
                            LLVMConstInt(uint(context, 1), 0, LLVM_FALSE)
                        }))
                    }
                }
                ElementaryTypeName::Byte(b) => {
                    if let Some(e) = &self.value {
                        e.codegen(context)
                    } else {
                        Ok(Some(unsafe {
                            LLVMConstInt(uint(context, *b as u32 * 8), 0, LLVM_FALSE)
                        }))
                    }
                }
                ElementaryTypeName::Uint(b) => {
                    if let Some(e) = &self.value {
                        e.codegen(context)
                    } else {
                        Ok(Some(unsafe {
                            LLVMConstInt(uint(context, *b as u32 * 8), 0, LLVM_FALSE)
                        }))
                    }
                }
                ElementaryTypeName::Int(b) => {
                    if let Some(e) = &self.value {
                        e.codegen(context)
                    } else {
                        Ok(Some(unsafe {
                            LLVMConstInt(uint(context, *b as u32 * 8), 0, LLVM_TRUE)
                        }))
                    }
                }
                ElementaryTypeName::Fixed(_, _) | ElementaryTypeName::Ufixed(_, _) => {
                    Err(CodeGenerationError::FixedPointNumbersNotStable)
                }
            },
            TypeName::ArrayTypeName(_, _) => panic!("Not implemented yet!"),
            TypeName::UserDefinedTypeName(user_defined_type_name) => {
                if let Some(e) = &self.value {
                    e.codegen(context)
                } else {
                    context
                        .symbols
                        .get(&format!("{}@default", user_defined_type_name.base.as_str()))
                        .ok_or(CodeGenerationError::UserDefinedTypeHasNoDefault(
                            user_defined_type_name.base.as_str().to_owned(),
                        ))
                        .map(|v| Some(v.clone()))
                }
            }
            _ => panic!("Not implemented yet"),
        }
    }
}

impl<'a> CodeGenerator for ContractPart {
    fn codegen(&self, _context: &mut Context) -> Result<Option<LLVMValueRef>, CodeGenerationError> {
        match self {
            ContractPart::EnumDefinition(_) => Ok(None),
            _ => Err(CodeGenerationError::NotImplementedYet),
        }
    }
}

impl<'a> CodeGenerator for Program {
    fn codegen(&self, context: &mut Context) -> Result<Option<LLVMValueRef>, CodeGenerationError> {
        for s in self.0.iter() {
            match s {
                SourceUnit::ContractDefinition(c) => {
                    context.symbols.clear();
                    let struct_type = unsafe {
                        LLVMStructCreateNamed(
                            context.context,
                            context.module.new_string_ptr(c.name.as_str()),
                        )
                    };
                    let mut types = Vec::new();
                    let mut vals = Vec::new();
                    for t in c.contract_parts.iter() {
                        let element_type = t.typegen(context)?;
                        if let Some(et) = element_type {
                            types.push(et);
                            vals.push(t.codegen(context)?.unwrap());
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
                    context.symbols.insert(c.name.as_str().to_owned(), contract);
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
