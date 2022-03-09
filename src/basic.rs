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
