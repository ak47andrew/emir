use crate::error;

#[derive(Clone, Debug)]
pub struct Buffer<T> {
    pub w: usize,
    pub h: usize,
    pub buff: Vec<T>,
}

impl<T: Default> Default for Buffer<T> {
    fn default() -> Self {
        Self {
            w: Default::default(),
            h: Default::default(),
            buff: Default::default(),
        }
    }
}

impl<T: Default + Clone> Buffer<T> {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            buff: vec![T::default(); w * h],
            w,
            h,
        }
    }
}

impl<T: Clone> Buffer<T> {
    pub fn from_element(w: usize, h: usize, default_element: T) -> Self {
        Self {
            buff: vec![default_element; w * h],
            w,
            h,
        }
    }

    /// # Safety
    /// Caller must ensure that x < self.w and y < self.h or you'll get UB (Ultimate banger)
    #[inline]
    pub unsafe fn get_unchecked(&self, x: usize, y: usize) -> T {
        unsafe { self.buff.get_unchecked(Self::idx(x, y, self.w)).clone() }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<T> {
        self.buff.get(Self::idx(x, y, self.w)).cloned()
    }
}

pub struct BufferIterator<T> {
    pub w: usize,
    pub position: usize,
    pub data: Box<[T]>,
}

impl<T> Buffer<T> {
    /// # Requirements
    /// `iterator` must not be infinite because it is relied upon in order to collect the elements
    /// of the buffer.
    ///
    /// # Returns
    /// - an error if the given iterator is empty;
    /// - an error if the amount of elements of the given iterator is not divisable by `w` without a
    ///   remainder.
    pub fn try_from_iterator(
        w: usize,
        iterator: impl Iterator<Item = T>,
    ) -> Result<Self, error::BufferError> {
        let buff: Vec<T> = iterator.collect();
        if buff.is_empty() {
            return Err(error::BufferError::EmptyIteratorOnInit {});
        }
        if !buff.len().is_multiple_of(w) {
            return Err(error::BufferError::InvalidSize {
                w,
                iterator_size: buff.len(),
            });
        }
        Ok(Self {
            w,
            h: buff.len() / w,
            buff,
        })
    }

    #[inline(always)]
    pub fn idx(x: usize, y: usize, w: usize) -> usize {
        y * w + x
    }

    /// # Safety
    /// Caller must ensure that x < self.w and y < self.h or you'll get UB (Unleashed butt)
    #[inline]
    pub unsafe fn set_unchecked(&mut self, x: usize, y: usize, color: T) {
        unsafe {
            *self.buff.get_unchecked_mut(Self::idx(x, y, self.w)) = color;
        }
    }

    pub fn set(&mut self, x: usize, y: usize, color: T) {
        if x >= self.w || y >= self.h {
            return;
        }
        unsafe {
            self.set_unchecked(x, y, color);
        }
    }

    pub fn lines(self) -> BufferIterator<T> {
        BufferIterator {
            w: self.w,
            data: self.buff.into_boxed_slice(),
            position: 0,
        }
    }
}

impl<T: Clone> Iterator for BufferIterator<T> {
    type Item = Box<[T]>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = vec![];
        for _ in 0..self.w {
            line.push(self.data.get(self.position).cloned()?);
            self.position += 1;
        }
        Some(line.into_boxed_slice())
    }
}
