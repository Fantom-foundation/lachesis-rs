use crate::error::MemoryError;
use failure::Error;
use std::cell::RefCell;
use std::mem::size_of;
use std::rc::Rc;

#[derive(Clone)]
pub struct Memory(Rc<RefCell<Vec<u64>>>);

impl Memory {
    pub(crate) fn new(capacity: usize) -> Memory {
        let memory = Rc::new(RefCell::new(Vec::with_capacity(capacity)));
        {
            let mut raw_memory = memory.borrow_mut();
            for _ in 0..capacity {
                raw_memory.push(0);
            }
        }
        Memory(memory)
    }

    pub(crate) fn get(&self, address: usize) -> Result<u64, Error> {
        let value = self
            .0
            .borrow()
            .get(address)
            .map(|v| v.clone())
            .ok_or(Error::from(MemoryError::WrongMemoryAddress { address }))?;
        Ok(value)
    }

    pub(crate) fn get_u32(&self, address: usize) -> Result<u32, Error> {
        let memory: &[u32] = unsafe {
            std::slice::from_raw_parts(self.0.borrow()[address..].as_ptr() as *const u32, 1)
        };
        Ok(memory[0])
    }

    pub(crate) fn get_i64(&self, address: usize) -> Result<i64, Error> {
        let value = self
            .0
            .borrow()
            .get(address)
            .map(|v| v.clone())
            .ok_or(Error::from(MemoryError::WrongMemoryAddress { address }))?;
        Ok(value as i64)
    }

    pub(crate) fn get_f64(&self, address: usize) -> Result<f64, Error> {
        let memory: &[f64] = unsafe {
            std::slice::from_raw_parts(self.0.borrow()[address..].as_ptr() as *const f64, 1)
        };
        Ok(memory[0])
    }

    pub(crate) fn get_u8_vector(&self, address: usize, size: usize) -> Result<Vec<u8>, Error> {
        let memory: &[u8] = unsafe {
            std::slice::from_raw_parts(self.0.borrow()[address..].as_ptr() as *const u8, size)
        };
        Ok(Vec::from(memory))
    }

    pub(crate) fn get_t<T>(&self, address: usize) -> Result<&T, Error> {
        let raw_data = self.get_u8_vector(address, size_of::<T>())?;
        unsafe { (raw_data.as_ptr() as *const T).as_ref() }.ok_or(Error::from(
            MemoryError::ErrorFetchingFunctionFromMemory
        ))
    }

    pub(crate) fn copy_u8_vector(&self, vector: &[u8], address: usize) {
        let memory: &mut [u8] = unsafe {
            std::slice::from_raw_parts_mut(
                self.0.borrow_mut()[address..].as_ptr() as *mut u8,
                vector.len(),
            )
        };
        memory.copy_from_slice(vector);
    }

    pub(crate) fn copy_u8(&self, value: u8, address: usize) {
        self.copy_u8_vector(&[value], address)
    }

    pub(crate) fn copy_u16_vector(&self, vector: &[u16], address: usize) {
        let memory: &mut [u16] = unsafe {
            std::slice::from_raw_parts_mut(
                self.0.borrow_mut()[address..].as_ptr() as *mut u16,
                vector.len(),
            )
        };
        memory.copy_from_slice(vector);
    }

    pub(crate) fn copy_u16(&self, value: u16, address: usize) {
        self.copy_u16_vector(&[value], address)
    }

    pub(crate) fn copy_u32_vector(&self, vector: &[u32], address: usize) {
        let memory: &mut [u32] = unsafe {
            std::slice::from_raw_parts_mut(
                self.0.borrow_mut()[address..].as_ptr() as *mut u32,
                vector.len(),
            )
        };
        memory.copy_from_slice(vector);
    }

    pub(crate) fn copy_u32(&self, value: u32, address: usize) {
        self.copy_u32_vector(&[value], address)
    }

    pub(crate) fn copy_u64_vector(&self, vector: &[u64], address: usize) {
        self.0.borrow_mut()[address..(address + vector.len())].copy_from_slice(vector);
    }

    pub(crate) fn copy_u64(&self, value: u64, address: usize) {
        self.0.borrow_mut()[address] = value;
    }

    pub(crate) fn copy_f64(&self, value: f64, address: usize) {
        let raw_memory = unsafe {
            std::slice::from_raw_parts_mut(
                self.0.borrow()[address..address + 1].as_ptr() as *mut f64,
                1,
            )
        };
        raw_memory.copy_from_slice(&[value]);
    }

    pub(crate) fn copy_i64(&self, value: i64, address: usize) {
        let raw_memory = unsafe {
            std::slice::from_raw_parts_mut(
                self.0.borrow()[address..address + 1].as_ptr() as *mut i64,
                1,
            )
        };
        raw_memory.copy_from_slice(&[value]);
    }

    pub(crate) fn copy_t<T>(&self, value: &T, address: usize) {
        let v: *const T = value;
        let p: &[u8] = unsafe { std::slice::from_raw_parts(v as *const u8, size_of::<T>()) };
        self.copy_u8_vector(p, address)
    }
}

#[cfg(test)]
mod tests {
    use crate::Memory;

    #[test]
    fn it_should_copy_a_u8_aray() {
        let data = &[1u8, 1, 1, 1, 1, 1, 1, 1];
        let memory = Memory::new(3);
        memory.copy_u8_vector(data, 1);
        assert_eq!(memory.0.borrow()[0], 0);
        assert_eq!(memory.0.borrow()[1], 72340172838076673);
        assert_eq!(memory.0.borrow()[2], 0);
    }

    #[test]
    fn it_should_copy_a_u8() {
        let memory = Memory::new(3);
        memory.0.borrow_mut()[1] = 257;
        memory.copy_u8(42, 1);
        assert_eq!(memory.0.borrow()[0], 0);
        assert_eq!(memory.0.borrow()[1], 298);
        assert_eq!(memory.0.borrow()[2], 0);
    }

    #[test]
    fn it_should_copy_a_u16_aray() {
        let data = &[1u16, 1, 1, 1];
        let memory = Memory::new(3);
        memory.copy_u16_vector(data, 1);
        assert_eq!(memory.0.borrow()[0], 0);
        assert_eq!(memory.0.borrow()[1], 281479271743489);
        assert_eq!(memory.0.borrow()[2], 0);
    }

    #[test]
    fn it_should_copy_a_u16() {
        let memory = Memory::new(3);
        memory.0.borrow_mut()[1] = 65537;
        memory.copy_u16(42, 1);
        assert_eq!(memory.0.borrow()[0], 0);
        assert_eq!(memory.0.borrow()[1], 65578);
        assert_eq!(memory.0.borrow()[2], 0);
    }

    #[test]
    fn it_should_copy_a_u32_aray() {
        let data = &[1u32, 1];
        let memory = Memory::new(3);
        memory.copy_u32_vector(data, 1);
        assert_eq!(memory.0.borrow()[0], 0);
        assert_eq!(memory.0.borrow()[1], 4294967297);
        assert_eq!(memory.0.borrow()[2], 0);
    }

    #[test]
    fn it_should_copy_a_u32() {
        let memory = Memory::new(3);
        memory.0.borrow_mut()[1] = 4294967297;
        memory.copy_u32(42, 1);
        assert_eq!(memory.0.borrow()[0], 0);
        assert_eq!(memory.0.borrow()[1], 4294967338);
        assert_eq!(memory.0.borrow()[2], 0);
    }

    #[test]
    fn it_should_copy_a_u64_aray() {
        let data = &[1u64];
        let memory = Memory::new(3);
        memory.copy_u64_vector(data, 1);
        assert_eq!(memory.0.borrow()[0], 0);
        assert_eq!(memory.0.borrow()[1], 1);
        assert_eq!(memory.0.borrow()[2], 0);
    }

    #[test]
    fn it_should_copy_a_u64() {
        let memory = Memory::new(3);
        memory.copy_u64(42, 1);
        assert_eq!(memory.0.borrow()[0], 0);
        assert_eq!(memory.0.borrow()[1], 42);
        assert_eq!(memory.0.borrow()[2], 0);
    }
}
