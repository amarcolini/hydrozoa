#![no_std]
#![warn(missing_docs)]
//! A rust wrapper for [WASM3](https://github.com/wasm3/wasm3).

extern crate alloc;

pub mod error;

pub mod environment;
pub use self::environment::Environment;
pub mod function;
pub use self::function::{CallContext, Function, RawCall};
pub mod macros;
mod module;
pub use self::module::{Instance, Module};
pub mod store;
pub use self::store::Store;
pub mod ty;
pub use ffi as wasm3_sys;

pub use self::ty::{WasmArg, WasmArgs, WasmType};
