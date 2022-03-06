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
pub struct CopyRange {
    pub start: usize,
    pub end: usize,
}

pub const fn r(start: usize, end: usize) -> CopyRange {
    return CopyRange { start, end };
}

impl CopyRange {
    #[inline(always)]
    pub fn len(&self) -> usize {
        return self.end - self.start;
    }
}

impl core::fmt::Debug for CopyRange {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        return write!(f, "{}..{}", self.start, self.end);
    }
}
