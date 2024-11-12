//! Error related functionality of wasm3.
use alloc::ffi::NulError;
use core::{cmp, ffi::CStr, fmt, ptr};

use snafu::Snafu;

/// Result alias that uses [`Error`].
pub type Result<T> = core::result::Result<T, Error>;
/// Result alias that uses [`Trap`].
pub type TrappedResult<T> = core::result::Result<T, Trap>;

/// A wasm trap.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Trap {
    /// Out of bounds memory access
    OutOfBoundsMemoryAccess,
    /// Division by zero
    DivisionByZero,
    /// Integer overflow
    IntegerOverflow,
    /// Integer conversion
    IntegerConversion,
    /// Indirect call type mismatch
    IndirectCallTypeMismatch,
    /// Table index out of range
    TableIndexOutOfRange,
    /// Exit
    Exit,
    /// Abort
    Abort,
    /// Unreachable
    Unreachable,
    /// Stack overflow
    StackOverflow,
}

impl Trap {
    /// Get the error message as a C string.
    pub fn as_cstr(self) -> &'static CStr {
        let ptr = unsafe {
            match self {
                Trap::OutOfBoundsMemoryAccess => ffi::m3Err_trapOutOfBoundsMemoryAccess,
                Trap::DivisionByZero => ffi::m3Err_trapDivisionByZero,
                Trap::IntegerOverflow => ffi::m3Err_trapIntegerOverflow,
                Trap::IntegerConversion => ffi::m3Err_trapIntegerConversion,
                Trap::IndirectCallTypeMismatch => ffi::m3Err_trapIndirectCallTypeMismatch,
                Trap::TableIndexOutOfRange => ffi::m3Err_trapTableIndexOutOfRange,
                Trap::Exit => ffi::m3Err_trapExit,
                Trap::Abort => ffi::m3Err_trapAbort,
                Trap::Unreachable => ffi::m3Err_trapUnreachable,
                Trap::StackOverflow => ffi::m3Err_trapStackOverflow,
            }
        };

        unsafe { CStr::from_ptr(ptr) }
    }

    /// Get the error message as a string.
    pub fn as_str(&self) -> &'static str {
        self.as_cstr()
            .to_str()
            .expect("expected wasm3 trap to be valid utf-8")
    }
}

impl cmp::PartialEq<Wasm3Error> for Trap {
    fn eq(&self, err: &Wasm3Error) -> bool {
        ptr::eq(err.0.as_ptr(), self.as_cstr().as_ptr())
    }
}

impl core::error::Error for Trap {}
impl fmt::Display for Trap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error returned by wasm3.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Wasm3Error(pub &'static CStr);

impl Wasm3Error {
    /// Check whether this error is the specified trap.
    pub fn eq_trap(self, trap: Trap) -> bool {
        ptr::eq(trap.as_cstr().as_ptr(), self.0.as_ptr())
    }

    /// Get the error message as a string.
    pub fn as_str(&self) -> &'static str {
        self.0
            .to_str()
            .expect("expected wasm3 error to be valid utf-8")
    }
}

impl cmp::PartialEq<Trap> for Wasm3Error {
    fn eq(&self, trap: &Trap) -> bool {
        self.eq_trap(*trap)
    }
}

impl core::error::Error for Wasm3Error {}
impl fmt::Display for Wasm3Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error returned by wasm3-rs.
#[derive(Clone, Debug, Snafu, PartialEq, Eq)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    /// An error originating from wasm3 itself may or may not be a trap.
    #[snafu(transparent)]
    Wasm3 {
        /// The source of the error.
        source: Wasm3Error,
    },
    /// A function has been found but its signature didn't match.
    InvalidFunctionSignature,
    /// The specified function could not be found.
    FunctionNotFound,
    /// The specified module could not be found.
    ModuleNotFound,
    /// The modules environment did not match the runtime's environment.
    ModuleLoadEnvMismatch,
    /// The specified store did not match the store the data was created with.
    StoreMismatch,
    /// A null byte was found in a string.
    #[snafu(transparent)]
    Nul {
        /// The source of the error.
        source: NulError,
    },
}

impl Error {
    /// # Safety
    ///
    /// `ptr` must be a valid pointer to a null-terminated string.
    pub(crate) unsafe fn from_ffi(ptr: ffi::M3Result) -> Result<()> {
        if ptr.is_null() {
            Ok(())
        } else if unsafe { ptr == ffi::m3Err_functionLookupFailed } {
            FunctionNotFoundSnafu.fail()
        } else {
            unsafe { Err(Wasm3Error(CStr::from_ptr(ptr)).into()) }
        }
    }

    pub(crate) fn malloc_error() -> Self {
        unsafe { Self::from_ffi(ffi::m3Err_mallocFailed).unwrap_err() }
    }
}
