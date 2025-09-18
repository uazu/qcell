use alloc::vec::Vec;

pub struct RwChain<'a, O> {
    pub(crate) owner: &'a mut O,
    pub(crate) addrs: Vec<usize>,
}

impl<'a, O> RwChain<'a, O> {
    pub(crate) fn new(owner: &'a mut O) -> Self {
        Self {
            owner,
            addrs: Vec::new(),
        }
    }

    pub(crate) fn push_addr<C: ?Sized>(&mut self, cell: &C) {
        self.addrs.push(cell as *const C as *const () as usize)
    }

    pub(crate) fn contains_addr<C: ?Sized>(&self, cell: &C) -> bool {
        self.addrs
            .contains(&(cell as *const C as *const () as usize))
    }
}
