use std::env::args;
use std::process::exit;

const USAGE: &'static str = "USAGE: llvm-vm-backend [llvm file] [output file]";
fn main() {
    let arguments = args();
    if arguments.len() != 3 {
        eprintln!("{}", USAGE);
        exit(1);
    }
}
