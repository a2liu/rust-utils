#[macro_export]
macro_rules! const_assert {
    ($cond:expr) => {
        // Causes overflow if condition is false
        let _ = [(); 0 - (!($cond) as usize)];
    };
    ($($xs:expr),+) => {
        const_assert!($($xs)&&+);
    };
    ($($xs:expr);+ $(;)*) => {
        const_assert!($($xs),+);
    };
}

#[cfg(debug_assertions)]
pub fn expect<V, E>(res: Result<V, E>) -> V
where
    E: core::fmt::Debug,
{
    return res.unwrap();
}

#[cfg(not(debug_assertions))]
pub fn expect<V, E>(res: Result<V, E>) -> V {
    let err = match res {
        Ok(v) => return v,
        Err(err) => err,
    };

    panic!("Expected value");
}

pub fn unwrap<V>(opt: Option<V>) -> V {
    if let Some(v) = opt {
        return v;
    }

    panic!("Expected value");
}

#[derive(Clone, Copy)]
pub struct CopyRange<U = usize>
where
    U: Copy,
{
    pub start: U,
    pub end: U,
}

#[inline(always)]
pub fn r<U>(start: U, end: U) -> CopyRange<U>
where
    U: Copy,
{
    return CopyRange { start, end };
}

impl CopyRange<usize> {
    #[inline(always)]
    pub fn len(&self) -> usize {
        return self.end - self.start;
    }
}

impl CopyRange<u32> {
    #[inline(always)]
    pub fn len(&self) -> u32 {
        return self.end - self.start;
    }
}

impl<U> core::fmt::Debug for CopyRange<U>
where
    U: core::fmt::Display + Copy,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        return write!(f, "{}..{}", self.start, self.end);
    }
}

pub trait SliceIndex<T>: Clone + core::fmt::Debug {
    type IndexResult: ?Sized;

    fn index(self, data: &[T]) -> Option<&Self::IndexResult>;

    fn index_mut(self, data: &mut [T]) -> Option<&mut Self::IndexResult>;
}

impl<T> SliceIndex<T> for u8 {
    type IndexResult = T;

    #[inline(always)]
    fn index(self, data: &[T]) -> Option<&Self::IndexResult> {
        return data.get(self as usize);
    }

    #[inline(always)]
    fn index_mut(self, data: &mut [T]) -> Option<&mut Self::IndexResult> {
        return data.get_mut(self as usize);
    }
}

impl<T> SliceIndex<T> for u16 {
    type IndexResult = T;

    #[inline(always)]
    fn index(self, data: &[T]) -> Option<&Self::IndexResult> {
        return data.get(self as usize);
    }

    #[inline(always)]
    fn index_mut(self, data: &mut [T]) -> Option<&mut Self::IndexResult> {
        return data.get_mut(self as usize);
    }
}

impl<T> SliceIndex<T> for u32 {
    type IndexResult = T;

    #[inline(always)]
    fn index(self, data: &[T]) -> Option<&Self::IndexResult> {
        return data.get(self as usize);
    }

    #[inline(always)]
    fn index_mut(self, data: &mut [T]) -> Option<&mut Self::IndexResult> {
        return data.get_mut(self as usize);
    }
}

impl<T> SliceIndex<T> for usize {
    type IndexResult = T;

    #[inline(always)]
    fn index(self, data: &[T]) -> Option<&Self::IndexResult> {
        return data.get(self);
    }

    #[inline(always)]
    fn index_mut(self, data: &mut [T]) -> Option<&mut Self::IndexResult> {
        return data.get_mut(self);
    }
}

impl<T> SliceIndex<T> for CopyRange<u32> {
    type IndexResult = [T];

    #[inline(always)]
    fn index(self, data: &[T]) -> Option<&Self::IndexResult> {
        return data.get((self.start as usize)..(self.end as usize));
    }

    #[inline(always)]
    fn index_mut(self, data: &mut [T]) -> Option<&mut Self::IndexResult> {
        return data.get_mut((self.start as usize)..(self.end as usize));
    }
}

impl<T> SliceIndex<T> for CopyRange<usize> {
    type IndexResult = [T];

    #[inline(always)]
    fn index(self, data: &[T]) -> Option<&Self::IndexResult> {
        return data.get(self.start..self.end);
    }

    #[inline(always)]
    fn index_mut(self, data: &mut [T]) -> Option<&mut Self::IndexResult> {
        return data.get_mut(self.start..self.end);
    }
}

impl<T> SliceIndex<T> for core::ops::Range<u32> {
    type IndexResult = [T];

    #[inline(always)]
    fn index(self, data: &[T]) -> Option<&Self::IndexResult> {
        return data.get((self.start as usize)..(self.end as usize));
    }

    #[inline(always)]
    fn index_mut(self, data: &mut [T]) -> Option<&mut Self::IndexResult> {
        return data.get_mut((self.start as usize)..(self.end as usize));
    }
}

impl<T> SliceIndex<T> for core::ops::Range<usize> {
    type IndexResult = [T];

    #[inline(always)]
    fn index(self, data: &[T]) -> Option<&Self::IndexResult> {
        return data.get(self);
    }

    #[inline(always)]
    fn index_mut(self, data: &mut [T]) -> Option<&mut Self::IndexResult> {
        return data.get_mut(self);
    }
}

impl<T> SliceIndex<T> for core::ops::RangeTo<usize> {
    type IndexResult = [T];

    #[inline(always)]
    fn index(self, data: &[T]) -> Option<&Self::IndexResult> {
        return data.get(self);
    }

    #[inline(always)]
    fn index_mut(self, data: &mut [T]) -> Option<&mut Self::IndexResult> {
        return data.get_mut(self);
    }
}

impl<T> SliceIndex<T> for core::ops::RangeFrom<usize> {
    type IndexResult = [T];

    #[inline(always)]
    fn index(self, data: &[T]) -> Option<&Self::IndexResult> {
        return data.get(self);
    }

    #[inline(always)]
    fn index_mut(self, data: &mut [T]) -> Option<&mut Self::IndexResult> {
        return data.get_mut(self);
    }
}

impl<T> SliceIndex<T> for core::ops::RangeFull {
    type IndexResult = [T];

    #[inline(always)]
    fn index(self, data: &[T]) -> Option<&Self::IndexResult> {
        return data.get(self);
    }

    #[inline(always)]
    fn index_mut(self, data: &mut [T]) -> Option<&mut Self::IndexResult> {
        return data.get_mut(self);
    }
}

pub const fn const_cond(cond: bool, if_true: usize, if_false: usize) -> usize {
    (cond as usize) * if_true + (!cond as usize) * if_false
}

pub const fn const_max(a: usize, b: usize) -> usize {
    const_cond(a > b, a, b)
}
