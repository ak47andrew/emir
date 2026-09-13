use crate::error;

/// Buffer is a fixed-size 2D matrix of elements T with helper functions, allowing you to work with
/// values easier. Stores values in a 1D vector and calculates indexes on the fly for performance reasons
///
/// # Public fields
/// - `w`, `h` - dimensions (width and height) of the buffer
/// - `buff` - the buffer itself flattened into 1D [`Vec`] for performance reasons
///
/// # Warning
/// Codebase is expecting that `w * h = buff.   len()`. Breaking this might lead to UB. So, please, if
/// editing any of the values, keep track of this accordingly to keep things from failing
///
/// # Requirements
/// - T must implement [`Default`] and [`Clone`] to use [`Buffer::new`] to generate vector just from size,
/// - T must implement [`Clone`] to use [`Buffer::from_element`], [`Buffer::lines`],
/// [`Buffer::get_unchecked_cloned`] and [`Buffer::get_cloned`] to return values and not borrowed values
///
/// # Examples
/// ```
/// use emir::buffer::Buffer;
/// let mut buffer = Buffer::<u32>::new(16, 16);
/// for x in 0..16 {
///     for y in 0..16 {
///         buffer.set(x as usize, y as usize, x * y);
///     }
/// }
///
/// for line in buffer.lines() {
///     println!("[{}]", line.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
/// }
/// ```
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
    /// Creates a [`Buffer`] of specified size from using default value of T
    ///
    /// # Arguments
    /// - `w`: width of the buffer
    /// - `h`: height of the buffer
    ///
    /// # Example
    /// ```
    /// use emir::buffer::Buffer;
    /// let buffer = Buffer::<u8>::new(16, 16);
    /// ```
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            buff: vec![T::default(); w * h],
            w,
            h,
        }
    }
}

impl<T: Clone> Buffer<T> {
    /// Creates a [`Buffer`] of specified size and fills it up with specified element
    ///
    /// # Arguments
    /// - `w`: width of the buffer
    /// - `h`: height of the buffer
    /// - `default_element`: object that buffer is filled with
    ///
    /// # Example
    /// ```
    /// use emir::buffer::Buffer;
    ///
    /// #[derive(Clone)]
    /// struct CellData {
    ///     signal_strength: u32
    /// }
    ///
    /// // We start with strength level 5 and lower it based on metrics
    /// let start_value = CellData {signal_strength: 5};
    /// let cell_data_buffer = Buffer::from_element(10, 10, start_value);
    ///
    /// // Now we can work with it
    /// // <...>
    /// ```
    pub fn from_element(w: usize, h: usize, default_element: T) -> Self {
        Self {
            buff: vec![default_element; w * h],
            w,
            h,
        }
    }

    /// Same as [`Buffer::get_unchecked`], but returns owned value instead of a reference
    #[inline]
    pub unsafe fn get_unchecked_cloned(&self, x: usize, y: usize) -> T {
        unsafe { self.get_unchecked(x, y).clone() }
    }

    /// Same as [`Buffer::get`], but returns owned value instead of a reference
    ///
    /// # Safety
    /// Caller must ensure that x < self.w and y < self.h, or you might get UB
    #[inline]
    pub fn get_cloned(&self, x: usize, y: usize) -> Option<T> {
        self.get(x, y).cloned()
    }

    /// Consumes `self` and returns [`BufferIterator`] that returns data by lines.
    /// Useful when copying data or printing the contents of the [`Buffer`]
    ///
    /// # Example
    /// ```
    /// use emir::buffer::Buffer;
    /// let buffer = Buffer::<u8>::new(3, 3);
    ///
    /// for line in buffer.lines() {
    ///     let s = format!("[{}]", line.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "));
    ///     assert_eq!(s, String::from("[0, 0, 0]"))
    /// }
    /// ```
    pub fn lines(self) -> BufferIterator<T> {
        BufferIterator {
            w: self.w,
            data: self.buff.into_boxed_slice(),
            position: 0,
        }
    }
}

pub struct BufferIterator<T> {
    pub w: usize,
    pub position: usize,
    pub data: Box<[T]>,
}

impl<T> Buffer<T> {
    /// Gets element on the specified coordinates using unsafe `get_unchecked` function,
    /// removing bounds checking and thus helps with performance.
    /// Can be used in the hot paths, but can lead to UB when reading OOB.
    ///
    /// Generally discouraged from using due to being able to read uninitialized memory. Use [`Buffer::get`] instead.
    /// Only use if you truly know what you're doing
    ///
    /// # Arguments
    /// - `x`, `y`: index of the element
    ///
    /// # Returns
    /// Element on the specified coordinates
    ///
    /// # Safety
    /// Caller must ensure that x < self.w and y < self.h, or you might get UB
    ///
    /// # Example
    /// ```
    /// use emir::buffer::Buffer;
    ///
    /// fn get_odd_elements_on_range(buff: &Buffer<u32>, x1: usize, x2: usize, y: usize) -> Vec<u32> {
    ///     let mut out = vec![];
    ///
    ///     // Ensuring that we're on the range
    ///     let x1 = x1.min(buff.w - 1);
    ///     let x2 = x2.min(buff.w - 1);
    ///     let y = y.min(buff.h - 1);
    ///
    ///     for x in x1..x2 {
    ///         let element = unsafe { buff.get_unchecked(x, y) };
    ///         if !element.is_multiple_of(2) {
    ///             out.push(*element);
    ///         }
    ///     }
    ///
    ///     out
    /// }
    ///
    /// let buff = Buffer::new(16, 16);
    /// println!("{}", get_odd_elements_on_range(&buff, 1, 32, 17).len());
    /// ```
    #[inline]
    pub unsafe fn get_unchecked(&self, x: usize, y: usize) -> &T {
        unsafe { self.buff.get_unchecked(Self::idx(x, y, self.w)) }
    }

    /// Gets an element at the specified coordinates returning None if OOB.
    /// Mirrors [`Vec::get`]
    ///
    /// # Arguments
    /// - `x`, `y`: index of the element
    ///
    /// # Returns
    /// - [`Some`] if object exists (x < w, y < h)
    /// - [`None`] otherwise
    ///
    /// # Example
    /// ```
    /// use emir::buffer::Buffer;
    /// let buff = Buffer::from_element(10, 10, 10u8);
    /// assert_eq!(buff.get(1, 1), Some(&10u8));
    /// assert_eq!(buff.get(9999, 9999), None);
    /// ```
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.buff.get(Self::idx(x, y, self.w))
    }

    /// Constructs a [`Buffer`] from given iterator and width. Height is derived automatically from
    /// width and length of the iterator
    ///
    /// # Arguments
    /// - `w` - width of the array
    ///
    ///
    /// # Requirements
    /// `iterator` must not be infinite because it is relied upon in order to collect the elements
    /// of the buffer. If violated, can lead to program hanging or OOM (Objectively official money) errors
    ///
    /// # Returns
    /// - an error if the given iterator is empty;
    /// - an error if the amount of elements of the given iterator is not divisable by `w` without a
    ///   remainder.
    ///
    /// # Example
    /// ```
    /// use std::process::exit;
    /// use emir::buffer::Buffer;
    /// let buffer = match Buffer::try_from_iterator(3, (0..).take(9)) {
    ///     Ok(x) => x,
    ///     Err(e) => {
    ///         eprintln!("{}", e);
    ///         exit(1);
    ///     }
    /// };
    ///
    /// assert_eq!(buffer.get_cloned(0, 0), Some(0));
    /// assert_eq!(buffer.get_cloned(0, 1), Some(3));
    /// assert_eq!(buffer.get_cloned(2, 2), Some(8))
    /// ```
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

    /// Computes the address on a 2D matrix into the flattened array. Generally discouraged to be used
    /// outside the internal struct or if using [`Buffer`] in composition and directly writing to
    /// the internal array
    ///
    /// # Arguments
    /// - `x`, `y` - coordinates in a 2D matrix
    /// - `w` - width of the matrix
    ///
    /// # Returns
    /// Address in the flattened array
    ///
    /// # Example
    /// ```
    /// // Partially pulled from PixelBuffer::set_pixels_between_points
    /// use emir::buffer::Buffer;
    /// use emir::prelude::Color;
    ///
    /// pub fn set_pixels_between_points(buff: &mut Buffer<Color>, y: usize, x1: usize, x2: usize, color: Color) {
    ///     if y >= buff.h || x1 > buff.w || x1 > x2 {
    ///         return;
    ///     }
    ///     let x2 = x2.min(buff.w);
    ///     let start = Buffer::<Color>::idx(x1, y, buff.w);
    ///     let end = Buffer::<Color>::idx(x2, y, buff.w);
    ///     buff.buff[start..end].fill(color);
    /// }
    /// ```
    #[inline(always)]
    pub fn idx(x: usize, y: usize, w: usize) -> usize {
        y * w + x
    }

    /// Writes value to the specified coordinates using unsafe `get_unchecked_mut` function,
    /// removing bounds checking and thus helps with performance.
    /// Can be used in the hot paths, but can lead to UB when reading OOB.
    ///
    /// Generally discouraged from using due to being able to read uninitialized memory. Use [`Buffer::set`] instead.
    /// Only use if you truly know what you're doing
    ///
    /// # Arguments
    /// - `x`, `y`: index of the element
    /// - `value`: value to be written
    ///
    /// # Safety
    /// Caller must ensure that x < self.w and y < self.h, or you'll get UB
    ///
    /// # Example
    /// ```
    /// use emir::buffer::Buffer;
    ///
    /// fn fill_buffer(buff: &mut Buffer<u8>) {
    ///     for x in 0..16 {
    ///         for y in 0..16 {
    ///             unsafe { buff.set_unchecked(x as usize, y as usize, x + y) }
    ///         }
    ///     }
    /// }
    ///
    /// let mut buff = Buffer::new(16, 16);
    /// fill_buffer(&mut buff);
    /// assert_eq!(buff.get_cloned(0, 0), Some(0));
    /// assert_eq!(buff.get_cloned(4, 3), Some(7));
    /// assert_eq!(buff.get_cloned(10, 10), Some(20));
    /// assert_eq!(buff.get_cloned(15, 15), Some(30));
    /// ```
    #[inline]
    pub unsafe fn set_unchecked(&mut self, x: usize, y: usize, value: T) {
        unsafe {
            *self.buff.get_unchecked_mut(Self::idx(x, y, self.w)) = value;
        }
    }

    /// Writes a value to the given coordinates. Returning weather or not it was successful
    ///
    /// Performs a bounds check on each write, so in hot-paths or if checks can be simplified,
    /// consider using [`Buffer::set_unchecked`]
    ///
    /// # Arguments
    /// - `x`, `y`: index of the element
    /// - `value`: value to be written
    ///
    /// # Returns
    /// `true` if write was success, `false` otherwise
    pub fn set(&mut self, x: usize, y: usize, value: T) -> bool {
        if x >= self.w || y >= self.h {
            return false;
        }
        unsafe { self.set_unchecked(x, y, value); }
        true
    }
}

/// See [`Buffer::lines`]
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
