#![no_std]
#![feature(c_variadic)]

extern crate alloc;

mod libc_support;
pub mod platform;
pub mod sdk;
pub mod teavm;
