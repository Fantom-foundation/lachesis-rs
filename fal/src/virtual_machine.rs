use crate::state::{Program, State};

pub trait VirtualMachine<T: State, U: Program> {
    fn transition(state: T, program: U) -> U;
}
