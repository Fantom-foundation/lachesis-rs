#[macro_use]
extern crate failure;
use failure::Error;
use lunarity::parse;
use lunarity::lexer::Token;
use std::ops::Range;

mod llvm;
pub use llvm::*;
