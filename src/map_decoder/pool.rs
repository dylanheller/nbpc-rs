use std::{cell::RefCell, iter, ops::Deref, rc::Rc};

pub struct Pool<T>(RefCell<Vec<Vec<Rc<[T]>>>>);

impl<T> Pool<T> {
    #[inline]
    pub fn new(m: usize, l: usize) -> Self {
        Self(RefCell::new(
            (0..m).map(|_| Vec::with_capacity(l)).collect(),
        ))
    }

    #[inline]
    pub fn put(&self, old_buf: Rc<[T]>, m: usize) {
        if Rc::strong_count(&old_buf) == 1 {
            self.0.borrow_mut()[m].push(old_buf);
        }
    }
}

impl<T: Clone + Copy + Default> Pool<T> {
    #[inline]
    pub fn get(&self, m: usize) -> BufHandle<'_, T> {
        let m_u8 = u8::try_from(m).unwrap();
        let buf = self.0.borrow_mut()[m]
            .pop()
            .unwrap_or_else(|| Rc::from_iter(iter::repeat_n(T::default(), 1 << m)));

        BufHandle {
            m: m_u8,
            buf: Some(buf),
            pool: self,
        }
    }
}

#[derive(Clone)]
pub struct BufHandle<'p, T> {
    m: u8,
    buf: Option<Rc<[T]>>,
    pool: &'p Pool<T>,
}

impl<T> Deref for BufHandle<'_, T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.buf.as_ref().unwrap_unchecked() }
    }
}

impl<T: Clone + Copy + Default> BufHandle<'_, T> {
    #[inline]
    pub fn borrow_mut_discard(&mut self) -> &mut [T] {
        if Rc::strong_count(self.buf.as_ref().unwrap()) > 1 {
            *self = self.pool.get(self.m as usize);
        }
        let rc = unsafe { self.buf.as_mut().unwrap_unchecked() };
        Rc::get_mut(rc).unwrap()
    }

    pub fn borrow_mut_clone(&mut self) -> &mut [T] {
        if Rc::strong_count(self.buf.as_ref().unwrap()) > 1 {
            let mut new = self.pool.get(self.m as usize);
            Rc::get_mut(new.buf.as_mut().unwrap())
                .unwrap()
                .copy_from_slice(unsafe { self.buf.as_ref().unwrap_unchecked() });
            *self = new;
        }

        Rc::get_mut(self.buf.as_mut().unwrap()).unwrap()
    }
}

impl<T> Drop for BufHandle<'_, T> {
    #[inline]
    fn drop(&mut self) {
        let buf = unsafe { Option::take(&mut self.buf).unwrap_unchecked() };
        self.pool.put(buf, self.m as usize);
    }
}
