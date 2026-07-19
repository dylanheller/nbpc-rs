pub struct DecisionHistory {
    n: usize,
    l: usize,
    nodes: Vec<u8>,
}

impl DecisionHistory {
    pub fn zeros(l: usize, n: usize) -> Self {
        assert!(l <= 64, "`l` must be <= 64");

        Self {
            n,
            l,
            nodes: vec![0; n * l],
        }
    }

    #[inline]
    fn uget(&self, i: usize, j: usize) -> (u8, u8) {
        let pos = i * self.l + j;
        let node = unsafe { *self.nodes.get_unchecked(pos) };
        (node >> 2, node & 0x3)
    }

    #[inline]
    pub fn set(&mut self, i: usize, j: usize, (parent_i, val): (u8, u8)) {
        assert!(
            i < self.n,
            "`i` is out of bounds for `DecisionHistory` with `n = {}`",
            self.n
        );
        assert!(
            j < self.l,
            "`j` is out of bounds for `DecisionHistory` with `l = {}`",
            self.l
        );
        assert!(parent_i < 64);
        assert!(val < 4);

        let pos = i * self.l + j;
        let node = (parent_i << 2) | val;

        unsafe { *self.nodes.get_unchecked_mut(pos) = node };
    }

    pub fn backtrack(&self, j_last: usize) -> Box<[u8]> {
        assert!(
            j_last < self.l,
            "`j_last` is out of bounds for `DecisionHistory` with `l = {}`",
            self.l
        );
        let mut path = Box::new_uninit_slice(self.n);

        let mut next_j = j_last;
        for i in (0..self.n).rev() {
            let (parent_i, val) = self.uget(i, next_j);
            path[i].write(val);
            next_j = usize::from(parent_i);
        }

        unsafe { path.assume_init() }
    }
}
