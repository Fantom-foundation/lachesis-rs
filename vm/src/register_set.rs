use crate::error::RuntimeError;
use crate::memory::Memory;
use failure::Error;

pub struct RegisterSet {
    pub address: usize,
    pub memory: Memory,
    pub size: usize,
}

impl RegisterSet {
    pub(crate) fn get(&self, index: usize) -> Result<u64, Error> {
        if index >= self.size {
            Err(Error::from(RuntimeError::InvalidRegisterIndex {
                register: index,
            }))
        } else {
            self.memory.get(self.address+index)
        }
    }
    pub(crate) fn get_i64(&self, index: usize) -> Result<i64, Error> {
        self.memory.get_i64(self.address+index)
    }
    pub(crate) fn set(&mut self, index: usize, value: u64) -> Result<(), Error> {
        if index >= self.size {
            Err(Error::from(RuntimeError::InvalidRegisterIndex {
                register: index,
            }))
        } else {
            self.memory.copy_u64(value, self.address+index);
            Ok(())
        }
    }
    pub(crate) fn set_i64(&self, index: usize, value: i64) {
        let address = self.address + index;
        self.memory.copy_i64(value, address)
    }
    pub(crate) fn to_string(&self, start_index: usize) -> Result<String, Error> {
        let u8_contents = self.memory.get_u8_vector(start_index, self.size - start_index)?;
        String::from_utf8(u8_contents).map_err(|e| Error::from(e))
    }
}
