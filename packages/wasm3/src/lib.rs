#![no_std]
#![warn(missing_docs)]
//! A rust wrapper for [WASM3](https://github.com/wasm3/wasm3).

extern crate alloc;

pub mod error;

mod environment;
pub use self::environment::Environment;
mod function;
pub use self::function::{CallContext, Function, RawCall};
mod macros;
mod module;
pub use self::module::{Instance, Module};
mod store;
pub use self::store::Store;
mod ty;
pub use ffi as wasm3_sys;

pub use self::ty::{WasmArg, WasmArgs, WasmType};
