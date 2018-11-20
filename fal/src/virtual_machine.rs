use crate::state::{State, Program};

pub trait VirtualMachine<T: State, U: Program> {
    fn transition(state:T, program: U) -> U;
}

