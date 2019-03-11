#[derive(Debug, Fail)]
pub enum ParsingError {
    #[fail(display = "The string being parsed doesn't have lenght")]
    StringWithoutLenght,
    #[fail(display = "There weren't enough bytes to form a u64")]
    U64LacksInformation,
    #[fail(display = "Type flag not present when parsing value")]
    ValueWithNoFlag,
    #[fail(display = "Empty strean when trying to parse a register")]
    RegisterExpectedNothingFound,
    #[fail(display = "Invalid instruction byte")]
    InvalidInstructionByte,
    #[fail(display = "Unexpected end of stream")]
    UnexpectedEndOfStream,
}

#[derive(Debug, Fail)]
pub enum RuntimeError {
    #[fail(
        display = "Wrong number of arguments for {}: Expected {}, got {}.",
        name, expected, got
    )]
    WrongArgumentsNumber {
        name: String,
        expected: usize,
        got: usize,
    },
    #[fail(display = "Program ended with code {}", errno)]
    ProgramEnded { errno: u64 },
    #[fail(display = "Global not found {}", name)]
    GlobalNotFound { name: String },
    #[fail(display = "Invalid register index {}", register)]
    InvalidRegisterIndex { register: usize },
    #[fail(display = "Trying to return from a never called function")]
    ReturnOnNoFunction,
}

#[derive(Debug, Fail)]
pub enum MemoryError {
    #[fail(display = "Wrong memory address {}", address)]
    WrongMemoryAddress { address: usize },
    #[fail(display = "Error converting memory to function")]
    ErrorFetchingFunctionFromMemory,
}
