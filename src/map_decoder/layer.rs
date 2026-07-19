use crate::map_decoder::pool::BufHandle;
use std::ops::Deref;

#[derive(Clone)]
pub enum LayerDecisions<'a> {
    One([u8; 1]),
    Two([u8; 2]),
    Four([u8; 4]),
    Eight([u8; 8]),
    Owned(Box<[u8]>),
    Shared(BufHandle<'a, u8>),
}

impl Deref for LayerDecisions<'_> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::One(layer) => layer,
            Self::Two(layer) => layer,
            Self::Four(layer) => layer,
            Self::Eight(layer) => layer,
            Self::Owned(layer) => layer,
            Self::Shared(buf_handle) => buf_handle,
        }
    }
}

impl LayerDecisions<'_> {
    pub fn borrow_mut_clone(&mut self) -> &mut [u8] {
        match self {
            Self::One(layer) => layer,
            Self::Two(layer) => layer,
            Self::Four(layer) => layer,
            Self::Eight(layer) => layer,
            Self::Owned(layer) => layer,
            Self::Shared(buf_handle) => buf_handle.borrow_mut_clone(),
        }
    }
}

#[derive(Clone)]
pub enum LayerProbs<'a> {
    One([[f64; 4]; 1]),
    Owned(Box<[[f64; 4]]>),
    Shared(BufHandle<'a, [f64; 4]>),
}

impl Deref for LayerProbs<'_> {
    type Target = [[f64; 4]];

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::One(layer) => layer,
            Self::Owned(layer) => layer,
            Self::Shared(buf_handle) => buf_handle,
        }
    }
}

impl LayerProbs<'_> {
    #[inline]
    pub fn borrow_mut_discard(&mut self) -> &mut [[f64; 4]] {
        match self {
            Self::One(layer) => layer,
            Self::Owned(layer) => layer,
            Self::Shared(buf_handle) => buf_handle.borrow_mut_discard(),
        }
    }
}
