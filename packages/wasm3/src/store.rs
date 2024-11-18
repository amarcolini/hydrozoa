use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    ffi::CString,
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};
use core::{
    cell::{Ref, RefCell, RefMut, UnsafeCell},
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
    pub fn new<D>(store: &StoreContext<D>, raw: NonNull<T>) -> Self {
        Self {
            raw,
            store_id: store.id(),
        }
    }

    pub fn get<D>(&self, ctx: &StoreContext<D>) -> Result<NonNull<T>> {
        ensure!(ctx.id() == self.store_id, StoreMismatchSnafu);
        Ok(self.raw)
    }
}

pub trait AsContextMut: AsContext {
    fn as_context_mut(&mut self) -> StoreContextMut<'_, Self::Data>;
}

impl<T: AsContextMut> AsContextMut for &mut T {
    fn as_context_mut(&mut self) -> StoreContextMut<'_, Self::Data> {
        (*self).as_context_mut()
    }
}

pub trait AsContext {
    type Data;
    fn as_context(&self) -> StoreContext<'_, Self::Data>;
}

impl<T: AsContext> AsContext for &T {
    type Data = T::Data;
    fn as_context(&self) -> StoreContext<'_, Self::Data> {
        (**self).as_context()
    }
}

impl<T: AsContext> AsContext for &mut T {
    type Data = T::Data;
    fn as_context(&self) -> StoreContext<'_, Self::Data> {
        (**self).as_context()
    }
}

/// A runtime context for wasm3 modules.
#[derive(Debug)]
pub struct Store<T: 'static> {
    raw: NonNull<ffi::M3Runtime>,
    data: Rc<RefCell<T>>,
    environment: Environment,
    // holds all linked closures so that they properly get disposed of when runtime drops
    closures: Vec<PinnedAnyClosure>,
    // holds all backing data of loaded modules as they have to be kept alive for the module's lifetime
    pub(crate) modules: Vec<RawModule>,
}

impl<T> Store<T> {
    /// Creates a new runtime with the given stack size in slots.
    ///
    /// # Errors
    ///
    /// This function will error on memory allocation failure.
    pub fn new(environment: &Environment, stack_size: u32, data: T) -> Result<Self> {
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
            data: Rc::new(RefCell::new(data)),
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
    pub fn instantiate(&mut self, module: Module) -> Result<Instance<T>> {
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

    /// Returns a reference to the data associated with this context.
    pub fn data(&self) -> Ref<'_, T> {
        self.data.borrow()
    }

    /// Returns a mutable reference to the data associated with this context.
    pub fn data_mut(&mut self) -> RefMut<'_, T> {
        self.data.borrow_mut()
    }

    /// Returns a description of the last error that occurred in this runtime.
    pub fn take_error_info(&mut self) -> Option<String> {
        let mut info = unsafe { mem::zeroed() };
        unsafe { ffi::m3_GetErrorInfo(self.as_ptr(), &mut info) };
        if info.message.is_null() {
            None
        } else {
            let message = unsafe { CStr::from_ptr(info.message).to_string_lossy().to_string() };
            if message.is_empty() {
                None
            } else {
                Some(message)
            }
        }
    }
}

impl<T> Store<T> {
    pub(crate) fn push_closure(&mut self, closure: PinnedAnyClosure) {
        self.closures.push(closure);
    }

    pub(crate) fn as_ptr(&self) -> ffi::IM3Runtime {
        self.raw.as_ptr()
    }

    pub(crate) fn data_ref(&self) -> Rc<RefCell<T>> {
        self.data.clone()
    }
}

impl<T> Drop for Store<T> {
    fn drop(&mut self) {
        for mut module in mem::take(&mut self.modules) {
            module.data = Cow::Borrowed(&[]);
            mem::forget(module);
        }
        unsafe { ffi::m3_FreeRuntime(self.raw.as_ptr()) };
    }
}

impl<T> AsContext for Store<T> {
    type Data = T;
    fn as_context(&self) -> StoreContext<'_, T> {
        StoreContext::new(self.raw, self.data.clone())
    }
}

impl<T> AsContextMut for Store<T> {
    fn as_context_mut(&mut self) -> StoreContextMut<'_, T> {
        StoreContextMut::new(self.raw, self.data.clone())
    }
}

#[derive(Debug)]
pub struct StoreContextMut<'a, T> {
    raw: NonNull<ffi::M3Runtime>,
    data: Rc<RefCell<T>>,
    scope: PhantomData<&'a mut ()>,
}

impl<T> StoreContextMut<'_, T> {
    pub(crate) fn new(raw: NonNull<ffi::M3Runtime>, data: Rc<RefCell<T>>) -> Self {
        Self {
            raw,
            data,
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

    /// Returns a reference to the data associated with this context.
    pub fn data(&self) -> Ref<'_, T> {
        self.data.borrow()
    }

    /// Returns a mutable reference to the data associated with this context.
    pub fn data_mut(&mut self) -> RefMut<'_, T> {
        self.data.borrow_mut()
    }

    pub(crate) fn as_ptr(&self) -> ffi::IM3Runtime {
        self.raw.as_ptr()
    }
}

impl<T> AsContext for StoreContextMut<'_, T> {
    type Data = T;
    fn as_context(&self) -> StoreContext<'_, T> {
        StoreContext::new(self.raw, self.data.clone())
    }
}

impl<T> AsContextMut for StoreContextMut<'_, T> {
    fn as_context_mut(&mut self) -> StoreContextMut<'_, T> {
        Self::new(self.raw, self.data.clone())
    }
}

pub struct StoreContext<'a, T> {
    raw: NonNull<ffi::M3Runtime>,
    data: Rc<RefCell<T>>,
    scope: PhantomData<&'a ()>,
}

impl<'a, T> StoreContext<'a, T> {
    pub(crate) fn new(raw: NonNull<ffi::M3Runtime>, data: Rc<RefCell<T>>) -> Self {
        Self {
            raw,
            data,
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

    /// Returns a reference to the data associated with this context.
    pub fn data(&self) -> Ref<'_, T> {
        self.data.borrow()
    }

    pub(crate) fn as_ptr(&self) -> ffi::IM3Runtime {
        self.raw.as_ptr()
    }

    pub(crate) fn id(&self) -> usize {
        self.raw.as_ptr() as usize
    }
}

impl<T> AsContext for StoreContext<'_, T> {
    type Data = T;
    fn as_context(&self) -> StoreContext<'_, T> {
        Self::new(self.raw, self.data.clone())
    }
}

#[test]
fn create_and_drop_rt() {
    let env = Environment::new().expect("env alloc failure");
    assert!(Store::new(&env, 1024 * 64, ()).is_ok());
}
