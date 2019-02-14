use failure::Error;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Fail)]
pub(crate) enum AllocatorError {
    #[fail(display = "Not enough memory to allocate {}", intended)]
    NotEnoughMemory { intended: usize },
    #[fail(display = "Address {} not allocated", address)]
    AddressNotAllocated { address: usize },
    #[fail(display = "Trying to free address {} already freed", address)]
    AddressAlreadyFreed { address: usize },
}

pub(crate) struct Allocator {
    memory: Rc<RefCell<Vec<u64>>>,
    free_chunks: Vec<(usize, usize)>,
    allocated_spaces: HashMap<usize, usize>,
}

impl Allocator {
    pub(crate) fn new(memory: Rc<RefCell<Vec<u64>>>) -> Allocator {
        let capacity = memory.borrow().capacity();
        Allocator {
            memory,
            allocated_spaces: HashMap::new(),
            free_chunks: vec![(0, capacity)],
        }
    }

    pub(crate) fn malloc(&mut self, size: usize) -> Result<usize, Error> {
        let free_memory = self
            .free_chunks
            .iter()
            .map(|(from, to)| to.clone() - from.clone())
            .sum();
        if size > free_memory {
            Err(Error::from(AllocatorError::NotEnoughMemory {
                intended: size,
            }))
        } else {
            let space = self
                .free_chunks
                .iter()
                .rev()
                .enumerate()
                .find(|(_, (from, to))| (to.clone() - from.clone()) >= size)
                .map(|(i, (f, t))| (self.free_chunks.len() - i - 1, (f.clone(), t.clone())));
            match space {
                None => Err(Error::from(AllocatorError::NotEnoughMemory {
                    intended: size,
                })),
                Some((index, (from, to))) => {
                    self.free_chunks.remove(index);
                    if from + size < to {
                        self.insert_free_chunk_sorted((from + size, to));
                    }
                    self.allocated_spaces.insert(from, size);
                    Ok(from)
                }
            }
        }
    }

    pub(crate) fn free(&mut self, address: usize) -> Result<(), Error> {
        match self.allocated_spaces.get(&address).map(|v| v.clone()) {
            Some(space) => {
                self.add_free_space(address, address + space)?;
                self.allocated_spaces.remove(&address);
                Ok(())
            }
            None => Err(Error::from(AllocatorError::AddressNotAllocated { address })),
        }
    }

    fn add_free_space(&mut self, from: usize, to: usize) -> Result<(), Error> {
        let adjacent = self
            .free_chunks
            .iter()
            .enumerate()
            .find(|(_, (f, t))| f.clone() == to || from == t.clone())
            .map(|(i, (f, t))| (i, (f.clone(), t.clone())));
        match adjacent {
            Some((i, (f, t))) => {
                self.free_chunks.remove(i);
                self.insert_free_chunk_sorted(if f == to { (from, t) } else { (f, to) })
            }
            None => self.insert_free_chunk_sorted((from, to)),
        }
    }

    fn insert_free_chunk_sorted(&mut self, item: (usize, usize)) -> Result<(), Error> {
        match self
            .free_chunks
            .binary_search_by(|(f, t)| (item.1 - item.0).cmp(&(t - f)))
        {
            Ok(_) => Err(Error::from(AllocatorError::AddressAlreadyFreed {
                address: item.0,
            })),
            Err(pos) => {
                self.free_chunks.insert(pos, item);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::allocator::Allocator;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: NotEnoughMemory { intended: 3 }"
    )]
    fn it_should_error_if_trying_to_allocate_more_space_than_memory_capacity() {
        let memory = Rc::new(RefCell::new(Vec::with_capacity(2)));
        let mut allocator = Allocator::new(memory);
        allocator.malloc(3).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: NotEnoughMemory { intended: 1 }"
    )]
    fn it_should_error_if_trying_to_allocate_more_space_than_available() {
        let memory = Rc::new(RefCell::new(Vec::with_capacity(2)));
        let mut allocator = Allocator::new(memory);
        allocator.malloc(2).unwrap();
        allocator.malloc(1).unwrap();
    }

    #[test]
    fn it_should_return_the_first_address_available() {
        let memory = Rc::new(RefCell::new(Vec::with_capacity(2)));
        let mut allocator = Allocator::new(memory);
        assert_eq!(allocator.malloc(1).unwrap(), 0);
        assert_eq!(allocator.malloc(1).unwrap(), 1);
    }

    #[test]
    fn it_should_correctly_free_memory() {
        let memory = Rc::new(RefCell::new(Vec::with_capacity(2)));
        let mut allocator = Allocator::new(memory);
        let address = allocator.malloc(2).unwrap();
        allocator.free(address).unwrap();
        assert_eq!(allocator.malloc(2).unwrap(), 0);
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: AddressNotAllocated { address: 1 }"
    )]
    fn it_should_fail_when_freeing_unallocated_space() {
        let memory = Rc::new(RefCell::new(Vec::with_capacity(2)));
        let mut allocator = Allocator::new(memory);
        allocator.malloc(2).unwrap();
        allocator.free(1).unwrap();
    }

    #[test]
    fn it_should_defragment_memory() {
        let memory = Rc::new(RefCell::new(Vec::with_capacity(5)));
        let mut allocator = Allocator::new(memory);
        let address1 = allocator.malloc(2).unwrap();
        let address2 = allocator.malloc(2).unwrap();
        allocator.free(address1).unwrap();
        allocator.free(address2).unwrap();
        allocator.malloc(4).unwrap();
        allocator.malloc(1).unwrap();
    }

    #[test]
    fn it_should_allocate_from_the_smallest_chunk_possible() {
        let memory = Rc::new(RefCell::new(Vec::with_capacity(5)));
        let mut allocator = Allocator::new(memory);
        let address1 = allocator.malloc(2).unwrap();
        let address2 = allocator.malloc(2).unwrap();
        allocator.free(address1).unwrap();
        allocator.free(address2).unwrap();
        allocator.malloc(1).unwrap();
        allocator.malloc(4).unwrap();
    }
}
