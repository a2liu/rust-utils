// Long-term
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_macros)]
#![allow(unused_braces)]
#![allow(non_upper_case_globals)]
// Short-term allows
/* */
#![allow(unused_imports)]
#![allow(unused_mut)]
/* */

extern crate alloc;

#[cfg(not(debug_assertions))]
macro_rules! panic {
    ( $( $arg:tt )+ ) => {{
        #[cfg(target_arch = "wasm32")]
        core::arch::wasm32::unreachable();

        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            core::ptr::write_volatile(core::ptr::null_mut(), 0);

            core::hint::unreachable_unchecked()
        }
    }};
}

#[cfg(not(debug_assertions))]
macro_rules! unreachable {
    ( $( $arg:tt )* ) => {{
        panic!()
    }};
}

#[macro_use]
mod basic;

mod alloc_api;

#[macro_use]
mod pod;

mod bump;
mod hashref;

pub use alloc_api::*;
pub use basic::*;
pub use bump::*;
pub use hashref::*;
pub use pod::*;
