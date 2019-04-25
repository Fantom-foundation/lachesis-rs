#[macro_use]
extern crate failure;
#[macro_use]
extern crate nom;

mod llvm;
mod parser;
pub use llvm::*;
pub use parser::Program;

pub fn parse(_content: &str) -> Result<Program, Vec<String>> {
    Err(Vec::new())
}
