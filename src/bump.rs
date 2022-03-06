use super::alloc_api::*;
use super::pod::*;
use super::unwrap;
use alloc::alloc::{alloc, dealloc, Layout};
use core::cell::Cell;
use std::ptr::NonNull;
use std::{cmp, mem, ptr, slice, str};

#[derive(Clone, Copy)]
struct Bump {
    ptr: NonNull<u8>,
    current: NonNull<u8>,
    layout: Layout,
}

const DANGLING: NonNull<u8> = NonNull::dangling();

const EMPTY_BUMP: Bump = Bump {
    ptr: DANGLING,
    current: DANGLING,
    layout: unsafe { Layout::from_size_align_unchecked(0, 1) },
};

impl Bump {
    fn new(layout: Layout) -> Bump {
        let ptr = unsafe { alloc(layout) };
        let ptr = unwrap(NonNull::new(ptr));

        return Bump {
            ptr,
            current: ptr,
            layout,
        };
    }

    fn alloc(&mut self, layout: Layout) -> Option<*mut u8> {
        if self.layout.size() == 0 {
            return None;
        }

        if layout.align() > 8 {
            panic!("Not handled");
        }

        let required_offset = self.current.as_ptr().align_offset(layout.align());
        if required_offset == usize::MAX {
            return None;
        }

        unsafe {
            let alloc_begin = self.current.as_ptr() as usize + required_offset;
            let alloc_end = alloc_begin + layout.size();
            let bump_end = self.ptr.as_ptr() as usize + self.layout.size();

            if alloc_end <= bump_end {
                self.current = NonNull::new_unchecked(alloc_end as *mut u8);

                return Some(alloc_begin as *mut u8);
            }

            return None;
        }
    }
}

pub struct BucketList {
    allocations: Cell<Pod<Bump>>,
    index: Cell<usize>,
}

#[derive(Clone, Copy)]
pub struct BucketListMark {
    index: usize,
    current: NonNull<u8>,
}

impl BucketList {
    pub const DEFAULT_BUCKET_SIZE: usize = 2 * 1024 * 1024;

    #[inline(always)]
    pub fn new() -> Self {
        return Self {
            allocations: Cell::new(Pod::new()),
            index: Cell::new(0),
        };
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let layout = match Layout::from_size_align(capacity, 8) {
            Ok(layout) => layout,
            Err(e) => panic!("failed to make Layout"),
        };

        let mut allocations = Pod::new();

        let bump = Bump::new(layout);

        allocations.push(bump);

        return Self {
            allocations: Cell::new(allocations),
            index: Cell::new(0),
        };
    }

    pub fn save(&self) -> BucketListMark {
        let allocations = self.allocations.replace(Pod::new());
        let index = self.index.get();

        if let Some(&bump) = allocations.get(index) {
            self.allocations.replace(allocations);

            return BucketListMark {
                index,
                current: bump.current,
            };
        }

        self.allocations.replace(allocations);

        return BucketListMark {
            index,
            current: DANGLING,
        };
    }

    pub unsafe fn set(&mut self, mark: BucketListMark) {
        let mut allocations = self.allocations.replace(Pod::new());

        let bump = match allocations.get_mut(mark.index) {
            Some(b) => b,
            None => return,
        };

        bump.current = bump.ptr;

        for bump in &mut allocations[(mark.index + 1)..(self.index.get() + 1)] {
            bump.current = bump.ptr;
        }

        self.index.set(mark.index);
        self.allocations.replace(allocations);
    }

    pub fn scoped<'a>(&'a mut self) -> ScopedBump<'a> {
        let mark = self.save();

        return ScopedBump { mark, alloc: self };
    }
}

unsafe impl Send for BucketList {}

impl Drop for BucketList {
    fn drop(&mut self) {
        let allocations = self.allocations.replace(Pod::new());

        for bump in allocations {
            unsafe {
                dealloc(bump.ptr.as_ptr(), bump.layout);
            }
        }
    }
}

unsafe impl Allocator for BucketList {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let mut index = self.index.get();
        let mut allocations = self.allocations.replace(Pod::new());

        let size = std::cmp::max(layout.size(), Self::DEFAULT_BUCKET_SIZE);
        let bump_layout = unsafe { Layout::from_size_align_unchecked(size, layout.align()) };

        if allocations.len() == 0 {
            let bump = Bump::new(bump_layout);
            allocations.push(bump);
        }

        let ptr = allocations[index].alloc(layout).unwrap_or_else(|| {
            let mut bump = Bump::new(bump_layout);
            let ptr = unwrap(bump.alloc(layout));
            index += 1;

            allocations.push(bump);

            return ptr;
        });

        let slice = unsafe { core::slice::from_raw_parts_mut(ptr, layout.size()) };
        let ptr = NonNull::new(slice).ok_or(AllocError)?;

        self.allocations.replace(allocations);
        self.index.set(index);

        return Ok(ptr);
    }

    // deallocation doesn't do anything
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {}
}

impl AllocStat for BucketList {
    fn total_used(&self) -> usize {
        let allocations = self.allocations.replace(Pod::new());

        let mut used = 0;
        for bump in allocations.iter() {
            let current = bump.current.as_ptr() as usize;
            let begin = bump.ptr.as_ptr() as usize;

            used += current - begin;
        }

        self.allocations.replace(allocations);

        return used;
    }

    fn total_capacity(&self) -> usize {
        let allocations = self.allocations.replace(Pod::new());

        let mut capacity = 0;

        for bump in allocations.iter() {
            capacity += bump.layout.size();
        }

        self.allocations.replace(allocations);

        return capacity;
    }
}

pub struct ScopedBump<'a> {
    mark: BucketListMark,
    alloc: &'a mut BucketList,
}

impl<'a> ScopedBump<'a> {
    pub fn chain<'b>(&'b mut self) -> ScopedBump<'b> {
        let mark = self.alloc.save();

        return ScopedBump {
            mark,
            alloc: &mut self.alloc,
        };
    }
}

impl<'a> Drop for ScopedBump<'a> {
    fn drop(&mut self) {
        unsafe {
            self.alloc.set(self.mark);
        }
    }
}
unsafe impl<'a> Allocator for ScopedBump<'a> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        return self.alloc.allocate(layout);
    }

    // deallocation doesn't do anything
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {}
}

impl AllocStat for ScopedBump<'_> {
    fn total_used(&self) -> usize {
        let allocations = self.alloc.allocations.replace(Pod::new());

        let mut used = 0;

        if allocations.len() != 0 {
            for bump in &allocations[(self.mark.index + 1)..] {
                let current = bump.current.as_ptr() as usize;
                let begin = bump.ptr.as_ptr() as usize;

                used += current - begin;
            }

            let bump = allocations[self.mark.index];
            let current = self.mark.current.as_ptr() as usize;
            let begin = bump.ptr.as_ptr() as usize;

            used -= current - begin;
        }

        self.alloc.allocations.replace(allocations);

        return used;
    }

    fn total_capacity(&self) -> usize {
        let allocations = self.alloc.allocations.replace(Pod::new());

        let mut capacity = 0;

        if allocations.len() != 0 {
            for bump in &allocations[(self.mark.index + 1)..] {
                capacity += bump.layout.size();
            }

            let bump = allocations[self.mark.index];
            let current = self.mark.current.as_ptr() as usize;
            let begin = bump.ptr.as_ptr() as usize;

            capacity -= current - begin;
        }

        self.alloc.allocations.replace(allocations);

        return capacity;
    }
}
