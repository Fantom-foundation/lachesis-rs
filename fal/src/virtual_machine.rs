use crate::state::{State, StateTransition};

pub trait VirtualMachine<T: State, U: StateTransition> {
    fn transition(state:T, transition: U) -> U;
}

