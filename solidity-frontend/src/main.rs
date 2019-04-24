use solidity_frontend::*;
use std::env::args;
use std::fs::read_to_string;
use std::process::exit;

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
            program.codegen(&mut context).unwrap();
            let output_file = arguments.next().unwrap();
            context.print_to_file(output_file.as_str()).unwrap();
        }
        Err(errors) => {
            for error in errors {
                eprintln!("{:?}", error);
            }
            exit(1);
        }
    }
}
