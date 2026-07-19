use crate::map_decoder::layer::{LayerDecisions, LayerProbs};
use crate::map_decoder::pool::Pool;
use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub struct ActivePathProbs<'a>(Box<[LayerProbs<'a>]>);

impl<'a> ActivePathProbs<'a> {
    pub fn new(m: usize, pool: &'a Pool<[f64; 4]>) -> Self {
        let mut data = Vec::with_capacity(m);

        if 0 < m {
            data.push(LayerProbs::One([[0.0; 4]]));
        }
        for lambda in 1..m {
            data.push(LayerProbs::Shared(pool.get(lambda)));
        }

        Self(data.into_boxed_slice())
    }

    pub fn owned(m: usize) -> Self {
        let mut data = Vec::with_capacity(m);

        if 0 < m {
            data.push(LayerProbs::One([[0.0; 4]]));
        }
        for lambda in 1..m {
            data.push(LayerProbs::Owned(
                vec![[0.0; 4]; 1 << lambda].into_boxed_slice(),
            ));
        }

        Self(data.into_boxed_slice())
    }

    #[inline]
    pub fn borrow_layers_pair_discard(
        &mut self,
        lambda: usize,
    ) -> (&mut [[f64; 4]], Option<&[[f64; 4]]>) {
        if lambda + 1 < self.0.len() {
            let [layer, layer_next] = self.0.get_disjoint_mut([lambda, lambda + 1]).unwrap();
            (layer.borrow_mut_discard(), Some(layer_next))
        } else {
            (self.0[lambda].borrow_mut_discard(), None)
        }
    }
}

impl Index<usize> for ActivePathProbs<'_> {
    type Output = [[f64; 4]];

    #[inline(always)]
    fn index(&self, lambda: usize) -> &Self::Output {
        &self.0[lambda]
    }
}

#[derive(Clone)]
pub struct ActivePathDecisions<'a>(Box<[LayerDecisions<'a>]>);

impl<'a> ActivePathDecisions<'a> {
    pub fn new(m: usize, pool: &'a Pool<u8>) -> Self {
        let mut data = Vec::with_capacity(m);

        if 0 < m {
            data.push(LayerDecisions::One([0; 1]));
        }
        if 1 < m {
            data.push(LayerDecisions::Two([0; 2]));
        }
        if 2 < m {
            data.push(LayerDecisions::Four([0; 4]));
        }
        if 3 < m {
            data.push(LayerDecisions::Eight([0; 8]));
        }
        for lambda in 4..m {
            data.push(LayerDecisions::Shared(pool.get(lambda)));
        }

        Self(data.into_boxed_slice())
    }

    pub fn owned(m: usize) -> Self {
        let mut data = Vec::with_capacity(m);

        if 0 < m {
            data.push(LayerDecisions::One([0; 1]));
        }
        if 1 < m {
            data.push(LayerDecisions::Two([0; 2]));
        }
        if 2 < m {
            data.push(LayerDecisions::Four([0; 4]));
        }
        if 3 < m {
            data.push(LayerDecisions::Eight([0; 8]));
        }
        for lambda in 4..m {
            data.push(LayerDecisions::Owned(
                vec![0; 1 << lambda].into_boxed_slice(),
            ));
        }

        Self(data.into_boxed_slice())
    }

    pub fn borrow_layers_pair(&mut self, lambda: usize) -> (&[u8], &mut [u8]) {
        let [layer, layer_next] = self.0.get_disjoint_mut([lambda, lambda + 1]).unwrap();
        (layer, layer_next.borrow_mut_clone())
    }
}

impl Index<usize> for ActivePathDecisions<'_> {
    type Output = [u8];

    #[inline(always)]
    fn index(&self, lambda: usize) -> &Self::Output {
        &self.0[lambda]
    }
}

impl IndexMut<usize> for ActivePathDecisions<'_> {
    fn index_mut(&mut self, lambda: usize) -> &mut Self::Output {
        self.0[lambda].borrow_mut_clone()
    }
}
