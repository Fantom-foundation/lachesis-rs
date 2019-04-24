#[macro_use]
extern crate failure;

mod llvm;
mod parser;
pub use llvm::*;
pub use parser::Program;

pub fn parse(_content: &str) -> Result<Program, Vec<String>> {
    Err(Vec::new())
}
