use std::cell::RefCell;
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

    pub(crate) fn get(&self) -> &Rc<RefCell<Vec<u64>>> {
        &self.0
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
}

#[cfg(test)]
mod tests {
    use crate::Memory;

    #[test]
    fn it_should_copy_a_u8_aray() {
        let data = &[1u8, 1, 1, 1, 1, 1, 1, 1];
        let memory = Memory::new(3);
        memory.copy_u8_vector(data, 1);
        assert_eq!(memory.get().borrow()[0], 0);
        assert_eq!(memory.get().borrow()[1], 72340172838076673);
        assert_eq!(memory.get().borrow()[2], 0);
    }

    #[test]
    fn it_should_copy_a_u8() {
        let memory = Memory::new(3);
        memory.get().borrow_mut()[1] = 257;
        memory.copy_u8(42, 1);
        assert_eq!(memory.get().borrow()[0], 0);
        assert_eq!(memory.get().borrow()[1], 298);
        assert_eq!(memory.get().borrow()[2], 0);
    }

    #[test]
    fn it_should_copy_a_u16_aray() {
        let data = &[1u16, 1, 1, 1];
        let memory = Memory::new(3);
        memory.copy_u16_vector(data, 1);
        assert_eq!(memory.get().borrow()[0], 0);
        assert_eq!(memory.get().borrow()[1], 281479271743489);
        assert_eq!(memory.get().borrow()[2], 0);
    }

    #[test]
    fn it_should_copy_a_u16() {
        let memory = Memory::new(3);
        memory.get().borrow_mut()[1] = 65537;
        memory.copy_u16(42, 1);
        assert_eq!(memory.get().borrow()[0], 0);
        assert_eq!(memory.get().borrow()[1], 65578);
        assert_eq!(memory.get().borrow()[2], 0);
    }

    #[test]
    fn it_should_copy_a_u32_aray() {
        let data = &[1u32, 1];
        let memory = Memory::new(3);
        memory.copy_u32_vector(data, 1);
        assert_eq!(memory.get().borrow()[0], 0);
        assert_eq!(memory.get().borrow()[1], 4294967297);
        assert_eq!(memory.get().borrow()[2], 0);
    }

    #[test]
    fn it_should_copy_a_u32() {
        let memory = Memory::new(3);
        memory.get().borrow_mut()[1] = 4294967297;
        memory.copy_u32(42, 1);
        assert_eq!(memory.get().borrow()[0], 0);
        assert_eq!(memory.get().borrow()[1], 4294967338);
        assert_eq!(memory.get().borrow()[2], 0);
    }

    #[test]
    fn it_should_copy_a_u64_aray() {
        let data = &[1u64];
        let memory = Memory::new(3);
        memory.copy_u64_vector(data, 1);
        assert_eq!(memory.get().borrow()[0], 0);
        assert_eq!(memory.get().borrow()[1], 1);
        assert_eq!(memory.get().borrow()[2], 0);
    }

    #[test]
    fn it_should_copy_a_u64() {
        let memory = Memory::new(3);
        memory.copy_u64(42, 1);
        assert_eq!(memory.get().borrow()[0], 0);
        assert_eq!(memory.get().borrow()[1], 42);
        assert_eq!(memory.get().borrow()[2], 0);
    }
}
