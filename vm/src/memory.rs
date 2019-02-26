use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct Memory(Rc<RefCell<Vec<u64>>>);

impl Memory {
    pub(crate) fn new(capacity: usize) -> Memory {
        let memory = Rc::new(RefCell::new(Vec::with_capacity(capacity)));
        {
            let mut raw_memory = memory.borrow_mut();
            for _ in 0..capacity - 1 {
                raw_memory.push(0);
            }
        }
        Memory(memory)
    }

    pub(crate) fn get(&self) -> &Rc<RefCell<Vec<u64>>> {
        &self.0
    }

    pub(crate) fn copy_u8_vector(&self, vector: &[u8], address: usize) {
        let data_len = (vector.len() as f64 / 8f64).ceil() as usize;
        let data: &[u64] =
            unsafe { std::slice::from_raw_parts(vector.as_ptr() as *const u64, data_len) };
        self.0.borrow_mut()[address..(address + data.len())].copy_from_slice(data);
    }
}

#[cfg(test)]
mod tests {
    use crate::Memory;

    #[test]
    fn it_should_copy_a_u8_aray() {
        let data = &[1u8, 1, 1, 1, 1, 1, 1, 1];
        let memory = Memory::new(2);
        memory.copy_u8_vector(data, 0);
        assert_eq!(memory.get().borrow()[0], 72340172838076673);
    }
}
