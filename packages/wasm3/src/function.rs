use core::{
    cmp::{Eq, PartialEq},
    ffi::{c_void, CStr},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ptr::{self, NonNull},
    slice, str,
};

use ffi::M3Function;
use snafu::ensure;

use crate::{
    error::{Error, Result, StoreMismatchSnafu},
    store::{AsContext, AsContextMut, Store, StoreContext, StoreContextMut, StoredData},
    Instance, WasmArg, WasmArgs, WasmType,
};

/// Calling Context for a host function.
pub struct CallContext<'cc> {
    raw: NonNull<ffi::M3Runtime>,
    _pd: PhantomData<fn(&'cc ()) -> &'cc ()>,
}

impl<'cc> CallContext<'cc> {
    pub(crate) fn from_raw(raw: NonNull<ffi::M3Runtime>) -> CallContext<'cc> {
        CallContext {
            raw,
            _pd: PhantomData,
        }
    }

    /// Returns the raw memory of the runtime associated with this context.
    ///
    /// # Safety
    ///
    /// The returned pointer may get invalidated when wasm function objects are called due to reallocations.
    pub fn memory(&self) -> &[u8] {
        self.as_context().memory()
    }

    /// Returns the raw memory of the runtime associated with this context.
    ///
    /// # Safety
    ///
    /// The returned pointer may get invalidated when wasm function objects are called due to reallocations.
    pub fn memory_mut(&mut self) -> &mut [u8] {
        let mut memory_size = 0u32;
        let data = unsafe { ffi::m3_GetMemory(self.raw.as_ptr(), &mut memory_size, 0) };
        unsafe { slice::from_raw_parts_mut(data, memory_size as usize) }
    }
}

impl AsContext for CallContext<'_> {
    fn as_context(&self) -> StoreContext<'_> {
        StoreContext::new(self.raw)
    }
}

impl AsContextMut for CallContext<'_> {
    fn as_context_mut(&mut self) -> StoreContextMut<'_> {
        StoreContextMut::new(self.raw)
    }
}

// redefine of ffi::RawCall without the Option<T> around it
/// Type of a raw host function for wasm3.
pub type RawCall = unsafe extern "C" fn(
    runtime: ffi::IM3Runtime,
    ctx: ffi::IM3ImportContext,
    _sp: *mut u64,
    _mem: *mut c_void,
) -> *const c_void;

/// A callable wasm3 function.
/// This has a generic `call` function for up to 26 parameters emulating an overloading behaviour without having to resort to tuples.
/// These are hidden to not pollute the documentation.
#[derive(Debug, Copy, Clone)]
pub struct Function<Args, Ret> {
    raw: StoredData<M3Function>,
    _pd: PhantomData<fn(Args) -> Ret>,
}

impl<Args, Ret> Eq for Function<Args, Ret> {}
impl<Args, Ret> PartialEq for Function<Args, Ret> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<Args, Ret> Hash for Function<Args, Ret> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<Args, Ret> Function<Args, Ret>
where
    Args: WasmArgs,
    Ret: WasmType,
{
    /// The name of this function.
    pub fn name(&self, ctx: impl AsContext) -> Result<&str> {
        unsafe {
            let name = ffi::m3_GetFunctionName(self.raw.get(&ctx.as_context())?.as_ptr());
            let cstr = CStr::from_ptr(name);
            Ok(cstr.to_str().expect("function name is not valid utf-8"))
        }
    }

    /// The module containing this function.
    pub fn instance(&self, ctx: impl AsContext) -> Result<Option<Instance>> {
        let ctx = ctx.as_context();
        let module = unsafe { ffi::m3_GetFunctionModule(self.raw.get(&ctx)?.as_ptr()) };
        Ok(NonNull::new(module).map(|module| unsafe { Instance::from_raw(&ctx, module) }))
    }
}

impl<Args, Ret> Function<Args, Ret>
where
    Args: WasmArgs,
    Ret: WasmType,
{
    fn validate_sig(raw: NonNull<M3Function>) -> bool {
        let num_args = unsafe { ffi::m3_GetArgCount(raw.as_ptr()) };
        let args = (0..num_args).map(|i| unsafe { ffi::m3_GetArgType(raw.as_ptr(), i) });
        if !Args::validate_types(args) {
            return false;
        }

        let num_rets = unsafe { ffi::m3_GetRetCount(raw.as_ptr()) };
        match num_rets {
            0 => Ret::TYPE_INDEX == ffi::M3ValueType::c_m3Type_none,
            1 => {
                let ret = unsafe { ffi::m3_GetRetType(raw.as_ptr(), 0) };
                Ret::TYPE_INDEX == ret
            }
            _ => false,
        }
    }

    #[inline]
    pub(crate) unsafe fn from_raw(store: &StoreContext, raw: NonNull<M3Function>) -> Result<Self> {
        if !Self::validate_sig(raw) {
            return Err(Error::InvalidFunctionSignature);
        }
        Ok(Function {
            raw: StoredData::new(store, raw),
            _pd: PhantomData,
        })
    }

    fn get_call_result(&self, raw: NonNull<M3Function>) -> Result<Ret> {
        unsafe {
            let mut ret = core::mem::MaybeUninit::<Ret>::uninit();
            let result = ffi::m3_GetResultsV(raw.as_ptr(), ret.as_mut_ptr());
            Error::from_ffi(result)?;
            Ok(ret.assume_init())
        }
    }
}

macro_rules! func_call_impl {
    ($($types:ident),*) => { func_call_impl!(@rec [$($types,)*] []); };
    (@rec [] [$($types:ident,)*]) => { func_call_impl!(@do_impl $($types,)*); };
    (@rec [$head:ident, $($tail:ident,)*] [$($types:ident,)*]) => {
        func_call_impl!(@do_impl $($types,)*);
        func_call_impl!(@rec [$($tail,)*] [$($types,)* $head,]);
    };
    (@do_impl) => {};
    (@do_impl $($types:ident,)*) => {
    #[doc(hidden)] // this really pollutes the documentation
        impl<$($types,)* Ret> Function<($($types,)*), Ret>
        where
            Ret: WasmType,
            ($($types,)*): WasmArgs,
        {
            #[inline]
            #[allow(non_snake_case, clippy::too_many_arguments)]
            pub fn call(&self, mut ctx: impl AsContextMut, $($types: $types),*) -> Result<Ret> {
                let ctx = ctx.as_context_mut();
                let raw = self.raw.get(&ctx.as_context())?;
                let result = unsafe { ffi::m3_CallV(raw.as_ptr(), $($types,)*) };
                unsafe { Error::from_ffi(result)?; }
                self.get_call_result(raw)
            }
        }
    };
}
func_call_impl!(A, B, C, D, E, F, G, H, J, K, L, M, N, O, P, Q);

impl<ARG, Ret> Function<ARG, Ret>
where
    Ret: WasmType,
    ARG: WasmArg,
{
    /// Calls this function with the given parameter.
    /// This is implemented with variable arguments depending on the functions Args type.
    #[inline]
    pub fn call(&self, mut ctx: impl AsContextMut, arg: ARG) -> Result<Ret> {
        let ctx = ctx.as_context_mut();
        let raw = self.raw.get(&ctx.as_context())?;
        let result = unsafe { ffi::m3_CallV(raw.as_ptr(), arg) };
        unsafe {
            Error::from_ffi(result)?;
        }
        self.get_call_result(raw)
    }
}

impl<Ret> Function<(), Ret>
where
    Ret: WasmType,
{
    /// Calls this function.
    /// This is implemented with variable arguments depending on the functions Args type.
    #[inline]
    pub fn call(&self, mut ctx: impl AsContextMut) -> Result<Ret> {
        let ctx = ctx.as_context_mut();
        let raw = self.raw.get(&ctx.as_context())?;
        let result = unsafe { ffi::m3_CallV(raw.as_ptr()) };
        unsafe {
            Error::from_ffi(result)?;
        }
        self.get_call_result(raw)
    }
}
