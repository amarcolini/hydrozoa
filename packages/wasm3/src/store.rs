use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    ffi::CString,
    string::{String, ToString},
    vec::Vec,
};
use core::{
    cell::UnsafeCell,
    ffi::CStr,
    hash::Hash,
    marker::PhantomData,
    mem,
    pin::Pin,
    ptr::{self, NonNull},
    slice,
};

use ffi::M3ErrorInfo;
use snafu::ensure;

use crate::{
    environment::Environment,
    error::{Error, ModuleLoadEnvMismatchSnafu, Result, StoreMismatchSnafu},
    function::Function,
    module::{Instance, Module, RawModule},
};

type PinnedAnyClosure = Pin<Box<dyn core::any::Any + 'static>>;

#[derive(Debug)]
pub(crate) struct StoredData<T> {
    raw: NonNull<T>,
    store_id: usize,
}

impl<T> Clone for StoredData<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for StoredData<T> {}

impl<T> PartialEq for StoredData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw && self.store_id == other.store_id
    }
}
impl<T> Eq for StoredData<T> {}

impl<T> Hash for StoredData<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
        self.store_id.hash(state);
    }
}

impl<T> StoredData<T> {
    pub fn new(store: &StoreContext, raw: NonNull<T>) -> Self {
        Self {
            raw,
            store_id: store.id(),
        }
    }

    pub fn get(&self, ctx: &StoreContext) -> Result<NonNull<T>> {
        ensure!(ctx.id() == self.store_id, StoreMismatchSnafu);
        Ok(self.raw)
    }
}

pub trait AsContextMut {
    fn as_context_mut(&mut self) -> StoreContextMut<'_>;
}

impl<T: AsContextMut> AsContextMut for &mut T {
    fn as_context_mut(&mut self) -> StoreContextMut<'_> {
        (*self).as_context_mut()
    }
}

pub trait AsContext {
    fn as_context(&self) -> StoreContext<'_>;
}

impl<T: AsContext> AsContext for &T {
    fn as_context(&self) -> StoreContext<'_> {
        (**self).as_context()
    }
}

impl<T: AsContext> AsContext for &mut T {
    fn as_context(&self) -> StoreContext<'_> {
        (**self).as_context()
    }
}

/// A runtime context for wasm3 modules.
#[derive(Debug)]
pub struct Store {
    raw: NonNull<ffi::M3Runtime>,
    environment: Environment,
    // holds all linked closures so that they properly get disposed of when runtime drops
    closures: Vec<PinnedAnyClosure>,
    // holds all backing data of loaded modules as they have to be kept alive for the module's lifetime
    pub(crate) modules: Vec<RawModule>,
}

impl Store {
    /// Creates a new runtime with the given stack size in slots.
    ///
    /// # Errors
    ///
    /// This function will error on memory allocation failure.
    pub fn new(environment: &Environment, stack_size: u32) -> Result<Self> {
        unsafe {
            NonNull::new(ffi::m3_NewRuntime(
                environment.as_ptr(),
                stack_size,
                ptr::null_mut(),
            ))
        }
        .ok_or_else(Error::malloc_error)
        .map(|raw| Store {
            raw,
            environment: environment.clone(),
            closures: Vec::new(),
            modules: Vec::new(),
        })
    }

    /// Loads a parsed module, returning its instance if successful.
    ///
    /// # Errors
    ///
    /// This function will error if the module's environment differs from the one this runtime uses.
    pub fn instantiate(&mut self, module: Module) -> Result<Instance> {
        if &self.environment != module.environment() {
            ModuleLoadEnvMismatchSnafu.fail()
        } else {
            let raw_mod = module.into_raw();
            unsafe {
                Error::from_ffi(ffi::m3_LoadModule(
                    self.raw.as_ptr(),
                    raw_mod.inner.as_ptr(),
                ))?
            };

            let instance = unsafe { Instance::from_raw(&self.as_context(), raw_mod.inner) };

            self.modules.push(raw_mod);
            Ok(instance)
        }
    }

    /// Looks up a function by the given name in the loaded modules of this runtime.
    /// See [`Module::find_function`] for possible error cases.
    ///
    /// [`Module::find_function`]: ../module/struct.Module.html#method.find_function
    pub fn find_function<ARGS, RET>(&self, name: &str) -> Result<Function<ARGS, RET>>
    where
        ARGS: crate::WasmArgs,
        RET: crate::WasmType,
    {
        let mut func_raw: ffi::IM3Function = core::ptr::null_mut();
        let func_name_cstr = CString::new(name)?;
        unsafe {
            Error::from_ffi(ffi::m3_FindFunction(
                &mut func_raw as *mut ffi::IM3Function,
                self.as_ptr(),
                func_name_cstr.as_ptr(),
            ))?;
        }
        let func = NonNull::new(func_raw).ok_or(Error::FunctionNotFound)?;
        unsafe { Function::from_raw(&self.as_context(), func) }
    }

    /// Returns the raw memory of this runtime.
    pub fn memory(&self) -> &[u8] {
        self.as_context().memory()
    }

    /// Returns the raw memory of this runtime.
    pub fn memory_mut(&mut self) -> &mut [u8] {
        let mut len: u32 = 0;
        let data = unsafe { ffi::m3_GetMemory(self.as_ptr(), &mut len, 0) };
        unsafe { slice::from_raw_parts_mut(data, len as usize) }
    }

    /// Returns a description of the last error that occurred in this runtime.
    pub fn take_error_info(&mut self) -> Option<String> {
        let mut info = unsafe { mem::zeroed() };
        unsafe { ffi::m3_GetErrorInfo(self.as_ptr(), &mut info) };
        if info.message.is_null() {
            None
        } else {
            let message = unsafe { CStr::from_ptr(info.message).to_string_lossy().to_string() };
            Some(message)
        }
    }
}

impl Store {
    pub(crate) fn push_closure(&mut self, closure: PinnedAnyClosure) {
        self.closures.push(closure);
    }

    pub(crate) fn as_ptr(&self) -> ffi::IM3Runtime {
        self.raw.as_ptr()
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        for mut module in mem::take(&mut self.modules) {
            module.data = Cow::Borrowed(&[]);
            mem::forget(module);
        }
        unsafe { ffi::m3_FreeRuntime(self.raw.as_ptr()) };
    }
}

impl AsContext for Store {
    fn as_context(&self) -> StoreContext<'_> {
        StoreContext::new(self.raw)
    }
}

impl AsContextMut for Store {
    fn as_context_mut(&mut self) -> StoreContextMut<'_> {
        StoreContextMut::new(self.raw)
    }
}

#[derive(Debug)]
pub struct StoreContextMut<'a> {
    raw: NonNull<ffi::M3Runtime>,
    scope: PhantomData<&'a mut ()>,
}

impl StoreContextMut<'_> {
    pub(crate) fn new(raw: NonNull<ffi::M3Runtime>) -> Self {
        Self {
            raw,
            scope: PhantomData,
        }
    }

    /// Looks up a function by the given name in the loaded modules of this runtime.
    /// See [`Module::find_function`] for possible error cases.
    ///
    /// [`Module::find_function`]: ../module/struct.Module.html#method.find_function
    pub fn find_function<ARGS, RET>(&self, name: &str) -> Result<Function<ARGS, RET>>
    where
        ARGS: crate::WasmArgs,
        RET: crate::WasmType,
    {
        self.as_context().find_function(name)
    }

    /// Returns the raw memory of this runtime.
    pub fn memory(&self) -> &[u8] {
        self.as_context().memory()
    }

    /// Returns the raw memory of this runtime.
    pub fn memory_mut(&mut self) -> &mut [u8] {
        let mut len: u32 = 0;
        let data = unsafe { ffi::m3_GetMemory(self.as_ptr(), &mut len, 0) };
        unsafe { slice::from_raw_parts_mut(data, len as usize) }
    }

    pub(crate) fn as_ptr(&self) -> ffi::IM3Runtime {
        self.raw.as_ptr()
    }
}

impl AsContext for StoreContextMut<'_> {
    fn as_context(&self) -> StoreContext<'_> {
        StoreContext::new(self.raw)
    }
}

impl AsContextMut for StoreContextMut<'_> {
    fn as_context_mut(&mut self) -> StoreContextMut<'_> {
        Self::new(self.raw)
    }
}

pub struct StoreContext<'a> {
    raw: NonNull<ffi::M3Runtime>,
    scope: PhantomData<&'a ()>,
}

impl<'a> StoreContext<'a> {
    pub(crate) fn new(raw: NonNull<ffi::M3Runtime>) -> Self {
        Self {
            raw,
            scope: PhantomData,
        }
    }

    /// Looks up a function by the given name in the loaded modules of this runtime.
    /// See [`Module::find_function`] for possible error cases.
    ///
    /// [`Module::find_function`]: ../module/struct.Module.html#method.find_function
    pub fn find_function<ARGS, RET>(&self, name: &str) -> Result<Function<ARGS, RET>>
    where
        ARGS: crate::WasmArgs,
        RET: crate::WasmType,
    {
        let mut func_raw: ffi::IM3Function = core::ptr::null_mut();
        let func_name_cstr = CString::new(name)?;
        unsafe {
            Error::from_ffi(ffi::m3_FindFunction(
                &mut func_raw as *mut ffi::IM3Function,
                self.as_ptr(),
                func_name_cstr.as_ptr(),
            ))?;
        }
        let func = NonNull::new(func_raw).ok_or(Error::FunctionNotFound)?;
        unsafe { Function::from_raw(self, func) }
    }

    /// Returns the raw memory of this runtime.
    pub fn memory(&self) -> &'a [u8] {
        let mut len: u32 = 0;
        let data = unsafe { ffi::m3_GetMemory(self.as_ptr(), &mut len, 0) };
        unsafe { slice::from_raw_parts(data, len as usize) }
    }

    pub(crate) fn as_ptr(&self) -> ffi::IM3Runtime {
        self.raw.as_ptr()
    }

    pub(crate) fn id(&self) -> usize {
        self.raw.as_ptr() as usize
    }
}

impl AsContext for StoreContext<'_> {
    fn as_context(&self) -> StoreContext<'_> {
        Self::new(self.raw)
    }
}

#[test]
fn create_and_drop_rt() {
    let env = Environment::new().expect("env alloc failure");
    assert!(Store::new(&env, 1024 * 64).is_ok());
}
