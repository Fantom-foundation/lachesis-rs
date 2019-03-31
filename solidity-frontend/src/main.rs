use lunarity::parse;
use std::env::args;
use std::fs::read_to_string;
use std::process::exit;
use solidity_frontend::*;

const USAGE: &'static str = "USAGE: solidity-frontend [solidity file] [output file]";

fn main() {
    let mut arguments = args();
    if arguments.len() != 3 {
        eprintln!("{}", USAGE);
        exit(1);
    }
    arguments.next();
    let file_name = arguments.next().unwrap();
    let file_content = read_to_string(file_name.as_str()).unwrap();
    match parse(file_content.as_str()) {
        Ok(program) => {
            let mut context = Context::new(file_name.as_str()).unwrap();
            let value = program.codegen(&mut context).unwrap();
        }
        Err(errors) => {
            for error in errors {
                eprintln!("{:?}", error);
            }
            exit(1);
        }
    }
}
