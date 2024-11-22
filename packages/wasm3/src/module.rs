use alloc::{borrow::Cow, boxed::Box, ffi::CString, rc::Rc, string::String, vec::Vec};
use core::{
    cell::RefCell,
    ffi::{c_char, c_void, CStr},
    marker::PhantomData,
    ptr::{self, NonNull},
};

use ffi::M3Module;
use snafu::{IntoError, ResultExt, Snafu};

use crate::{
    environment::Environment,
    error::{Error, Result, Trap},
    function::{CallContext, Function, RawCall},
    store::{AsContext, Store, StoreContext, StoredData},
};

/// Failed to link a WASM function.
#[derive(Debug, Snafu)]
#[snafu(display("Failed to link function `{name}`"))]
pub struct ClosureLinkFailed {
    /// The source of the error.
    source: Error,
    /// The name of the function that was being linked.
    name: String,
}

#[derive(Debug)]
pub(crate) struct RawModule {
    pub inner: NonNull<ffi::M3Module>,
    pub data: Cow<'static, [u8]>,
}

impl Drop for RawModule {
    fn drop(&mut self) {
        unsafe { ffi::m3_FreeModule(self.inner.as_ptr()) };
    }
}

/// A parsed module which can be loaded into a [`Runtime`].
pub struct Module {
    raw: RawModule,
    env: Environment,
}

impl Module {
    /// Parses a wasm module from raw bytes.
    pub fn parse(env: &Environment, data: impl Into<Cow<'static, [u8]>>) -> Result<Self> {
        let data = data.into();
        assert!(data.len() <= !0u32 as usize);
        let mut module = ptr::null_mut();
        unsafe {
            Error::from_ffi(ffi::m3_ParseModule(
                env.as_ptr(),
                &mut module,
                data.as_ptr(),
                data.len() as u32,
            ))?;
        }
        let module = NonNull::new(module)
            .expect("module pointer is non-null after m3_ParseModule if result is not error");
        Ok(Module {
            raw: RawModule {
                inner: module,
                data,
            },
            env: env.clone(),
        })
    }

    pub(crate) fn as_ptr(&self) -> ffi::IM3Module {
        self.raw.inner.as_ptr()
    }

    pub(crate) fn into_raw(self) -> RawModule {
        self.raw
    }

    /// The environment this module was parsed in.
    pub fn environment(&self) -> &Environment {
        &self.env
    }
}

/// A loaded module belonging to a specific runtime. Allows for linking and looking up functions.
// needs no drop as loaded modules will be cleaned up by the runtime
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance<T>(StoredData<M3Module>, PhantomData<fn() -> T>);

impl<T> Instance<T> {
    /// Links the given function to the corresponding module and function name.
    /// This allows linking a more verbose function, as it gets access to the unsafe
    /// runtime parts. For easier use the [`make_func_wrapper`] should be used to create
    /// the unsafe facade for your function that then can be passed to this.
    ///
    /// For a simple API see [`Self::link_closure`] which takes a closure instead.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations:
    ///
    /// * a memory allocation failed
    /// * no function by the given name in the given module could be found
    /// * the function has been found but the signature did not match
    pub fn link_function<Args, Ret>(
        &mut self,
        store: &mut Store<T>,
        module_name: impl Into<Vec<u8>>,
        function_name: impl Into<Vec<u8>>,
        f: RawCall,
    ) -> Result<()>
    where
        Args: crate::WasmArgs,
        Ret: crate::WasmType,
    {
        let module_name_cstr = CString::new(module_name)?;
        let function_name_cstr = CString::new(function_name)?;
        let signature = function_signature::<Args, Ret>();

        unsafe {
            Error::from_ffi(ffi::m3_LinkRawFunction(
                self.0.get(&store.as_context())?.as_ptr(),
                module_name_cstr.as_ptr(),
                function_name_cstr.as_ptr(),
                signature.as_ptr(),
                Some(f),
            ))
        }
    }

    /// Links the given closure to the corresponding module and function name.
    /// This boxes the closure and therefor requires a heap allocation.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations:
    ///
    /// * a memory allocation failed
    /// * no function by the given name in the given module could be found
    /// * the function has been found but the signature did not match
    pub fn link_closure<Args, Ret, F>(
        &mut self,
        store: &mut Store<T>,
        module_name: &str,
        function_name: &str,
        closure: F,
    ) -> core::result::Result<(), ClosureLinkFailed>
    where
        Args: crate::WasmArgs,
        Ret: crate::WasmType,
        F: for<'cc> FnMut(CallContext<'cc, T>, Args) -> core::result::Result<Ret, Trap> + 'static,
    {
        struct UserData<T, F> {
            pub closure: F,
            pub data: Rc<RefCell<T>>,
        }

        unsafe extern "C" fn trampoline<Args, Ret, F, T>(
            runtime: ffi::IM3Runtime,
            ctx: ffi::IM3ImportContext,
            sp: *mut u64,
            _mem: *mut c_void,
        ) -> *const c_void
        where
            Args: crate::WasmArgs,
            Ret: crate::WasmType,
            F: for<'cc> FnMut(CallContext<'cc, T>, Args) -> core::result::Result<Ret, Trap>
                + 'static,
        {
            let runtime = NonNull::new(runtime)
                .expect("wasm3 calls imported functions with non-null runtime");
            let ctx = NonNull::new(ctx)
                .expect("wasm3 calls imported functions with non-null import context");
            let user_data = NonNull::new(ctx.as_ref().userdata as *mut UserData<T, F>)
                .expect("userdata passed to m3_LinkRawFunctionEx is non-null")
                .as_mut();

            let args = Args::pop_from_stack(sp.add(Ret::SIZE_IN_SLOT_COUNT));
            let ret =
                (user_data.closure)(CallContext::from_raw(runtime, user_data.data.clone()), args);
            let result = match ret {
                Ok(ret) => {
                    ret.push_on_stack(sp);
                    ffi::m3Err_none
                }
                Err(trap) => trap.as_cstr().as_ptr(),
            };
            result.cast()
        }

        let module_name_cstr =
            CString::new(module_name)
                .map_err(Error::from)
                .context(ClosureLinkFailedSnafu {
                    name: function_name,
                })?;
        let function_name_cstr =
            CString::new(function_name)
                .map_err(Error::from)
                .context(ClosureLinkFailedSnafu {
                    name: function_name,
                })?;
        let signature = function_signature::<Args, Ret>();

        let mut closure = Box::pin(UserData {
            closure,
            data: store.data_ref(),
        });

        let err = unsafe {
            Error::from_ffi(ffi::m3_LinkRawFunctionEx(
                self.0
                    .get(&store.as_context())
                    .context(ClosureLinkFailedSnafu {
                        name: function_name,
                    })?
                    .as_ptr(),
                module_name_cstr.as_ptr(),
                function_name_cstr.as_ptr(),
                signature.as_ptr(),
                Some(trampoline::<Args, Ret, F, T>),
                closure.as_mut().get_unchecked_mut() as *const UserData<T, F> as *const c_void,
            ))
        };

        if let Err(err) = err {
            if err != Error::FunctionNotFound {
                Err(err).context(ClosureLinkFailedSnafu {
                    name: function_name,
                })?;
            }
        }

        store.push_closure(closure);
        Ok(())
    }

    /// Looks up a function by the given name in this module.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations:
    ///
    /// * a memory allocation failed
    /// * no function by the given name in the given module could be found
    /// * the function has been found but the signature did not match
    pub fn find_function<Args, Ret>(
        &self,
        store: &Store<T>,
        function_name: &str,
    ) -> Result<Function<Args, Ret>>
    where
        Args: crate::WasmArgs,
        Ret: crate::WasmType,
    {
        let function = store.find_function(function_name)?;
        match function.instance(store)? {
            Some(instance) if instance == self.0.get(&store.as_context())? => Ok(function),
            _ => Err(Error::FunctionNotFound),
        }
    }

    /// The name of this module.
    pub fn name<'a>(&self, ctx: impl AsContext) -> Result<&'a str> {
        Ok(unsafe {
            CStr::from_ptr(ffi::m3_GetModuleName(
                self.0.get(&ctx.as_context())?.as_ptr(),
            ))
            .to_str()
            .expect("module name is not valid utf-8")
        })
    }

    //     /// Links wasi to this module.
    //     #[cfg(feature = "wasi")]
    //     pub fn link_wasi(&mut self) -> Result<()> {
    //         unsafe { Error::from_ffi_res(ffi::m3_LinkWASI(self.raw)) }
    //     }
}

impl<T> Instance<T> {
    pub(crate) unsafe fn from_raw(store: &StoreContext<T>, raw: NonNull<M3Module>) -> Self {
        Instance(StoredData::new(store, raw), PhantomData)
    }
}

fn function_signature<Args, Ret>() -> Vec<c_char>
where
    Args: crate::WasmArgs,
    Ret: crate::WasmType,
{
    let mut signature = <Vec<c_char>>::new();
    signature.push(Ret::SIGNATURE as c_char);
    signature.push(b'(' as c_char);
    Args::append_signature(&mut signature);
    signature.push(b')' as c_char);
    signature.push(b'\0' as c_char);
    signature
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::TrappedResult, make_func_wrapper};

    make_func_wrapper!(mul_u32_and_f32_wrap: mul_u32_and_f32(a: u32, b: f32) -> f64);
    fn mul_u32_and_f32(a: u32, b: f32) -> f64 {
        (a as f64) * (b as f64)
    }

    make_func_wrapper!(hello_wrap: hello() -> TrappedResult<()>);
    fn hello() -> TrappedResult<()> {
        Ok(())
    }

    const TEST_BIN: &[u8] = include_bytes!("../tests/wasm_test_bins/wasm_test_bins.wasm");
    const STACK_SIZE: u32 = 1_000;

    #[test]
    fn module_parse() {
        let env = Environment::new().expect("env alloc failure");
        let fib32 = [
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x06, 0x01, 0x60, 0x01, 0x7f,
            0x01, 0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x66, 0x69, 0x62, 0x00,
            0x00, 0x0a, 0x1f, 0x01, 0x1d, 0x00, 0x20, 0x00, 0x41, 0x02, 0x49, 0x04, 0x40, 0x20,
            0x00, 0x0f, 0x0b, 0x20, 0x00, 0x41, 0x02, 0x6b, 0x10, 0x00, 0x20, 0x00, 0x41, 0x01,
            0x6b, 0x10, 0x00, 0x6a, 0x0f, 0x0b,
        ];
        let _ = Instance::parse(&env, &fib32[..]).unwrap();
    }

    #[test]
    fn test_link_functions() {
        let env = Environment::new().expect("env alloc failure");
        let runtime = Store::new(&env, STACK_SIZE, ()).expect("runtime init failure");
        let mut module = runtime.parse_and_instantiate(TEST_BIN).unwrap();
        module
            .link_function::<(u32, f32), f64>("env", "mul_u32_and_f32", mul_u32_and_f32_wrap)
            .unwrap();
        module
            .link_function::<(), ()>("env", "hello", hello_wrap)
            .unwrap();
    }

    #[test]
    fn test_link_closures() {
        let env = Environment::new().expect("env alloc failure");
        let runtime = Store::new(&env, STACK_SIZE, ()).expect("runtime init failure");
        let mut module = runtime.parse_and_instantiate(TEST_BIN).unwrap();
        module
            .link_closure(
                "env",
                "mul_u32_and_f32",
                |_ctx, args: (u32, f32)| -> TrappedResult<f64> {
                    Ok(mul_u32_and_f32(args.0, args.1))
                },
            )
            .unwrap();
        module
            .link_closure("env", "hello", |_ctx, _args: ()| -> TrappedResult<()> {
                hello()
            })
            .unwrap();
    }
}
