#[derive(Clone, Copy)]
pub struct Xoshiro128PlusPlus {
    s: [u32; 4],
}

impl Default for Xoshiro128PlusPlus {
    fn default() -> Self {
        Self { s: [1, 1, 1, 1] }
    }
}

impl Xoshiro128PlusPlus {
    #[inline(always)]
    pub const fn from_seed(s: [u32; 4]) -> Self {
        Self { s }
    }

    #[inline(always)]
    const fn next(&mut self) -> u32 {
        let res = self.s[0]
            .wrapping_add(self.s[3])
            .rotate_left(7)
            .wrapping_add(self.s[0]);

        let t = self.s[1] << 9;

        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];

        self.s[2] ^= t;

        self.s[3] = self.s[3].rotate_left(11);

        res
    }

    #[inline(always)]
    pub const fn random_bounded(&mut self, bound: u32) -> u32 {
        if bound == 0 {
            return 0;
        }

        let limit = (0_u32.wrapping_sub(bound)) % bound;

        let mut x = self.next();
        while x < limit {
            x = self.next();
        }

        x % bound
    }
}
