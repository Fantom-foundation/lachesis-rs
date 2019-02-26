use crate::error::RuntimeError;
use crate::memory::Memory;
use failure::Error;

fn extract_bytes(n: u64) -> [u8; 8] {
    let mut res = [0; 8];
    for i in 0..7 {
        res[i] = (n >> (7 - i)) as u8;
    }
    res
}

pub struct RegisterSet {
    pub address: usize,
    pub memory: Memory,
    pub size: usize,
}

impl RegisterSet {
    pub(crate) fn get(&self, index: usize) -> Result<u64, Error> {
        if index >= self.size {
            Err(Error::from(RuntimeError::InvalidRegisterIndex { register: index }))
        } else {
            Ok(self.memory.get().borrow()[index])
        }
    }
    pub(crate) fn set(&mut self, index: usize, value: u64) -> Result<(), Error> {
        if index >= self.size {
            Err(Error::from(RuntimeError::InvalidRegisterIndex { register: index }))
        } else {
            self.memory.get().borrow_mut()[index] = value;
            Ok(())
        }
    }
    pub(crate) fn to_string(&self, start_index: usize) -> Result<String, Error> {
        let u8_contents = self.memory.get().borrow()[start_index..self.address+self.size]
            .iter()
            .map(|n| extract_bytes(n.clone()).to_vec().into_iter())
            .flatten()
            .collect::<Vec<u8>>();
        String::from_utf8(u8_contents).map_err(|e| Error::from(e))
    }
}
