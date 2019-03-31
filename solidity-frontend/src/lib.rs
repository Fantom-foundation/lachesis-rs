#[macro_use]
extern crate failure;
use failure::Error;
use lunarity::lexer::Token;
use lunarity::parse;
use std::ops::Range;

mod llvm;
pub use llvm::*;
