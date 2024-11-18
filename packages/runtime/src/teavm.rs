#![allow(non_snake_case)]

use alloc::{boxed::Box, rc::Rc, string::String, sync::Arc};
use core::str;

use anyhow::Context;
use vexide::core::{print, println, sync::OnceLock, time::Instant};
use wasm3::{
    error::Result,
    store::{AsContextMut, StoreContextMut},
    Function, Instance, Store, WasmArg, WasmType,
};

use crate::Data;

pub fn link_teavm(store: &mut Store<Data>, instance: &mut Instance<Data>) -> anyhow::Result<()> {
    let teavm_catchException = instance
        .find_function::<(), i32>(store, "teavm_catchException")
        .context("teavm_catchException")?;
    let teavm_allocateStringArray = wrap(
        instance
            .find_function::<i32, i32>(store, "teavm_allocateStringArray")
            .context("teavm_allocateStringArray")?,
        teavm_catchException,
    );
    let teavm_objectArrayData = wrap(
        instance
            .find_function::<i32, i32>(store, "teavm_objectArrayData")
            .context("teavm_objectArrayData")?,
        teavm_catchException,
    );
    let teavm_byteArrayData = wrap(
        instance
            .find_function::<i32, i32>(store, "teavm_byteArrayData")
            .context("teavm_byteArrayData")?,
        teavm_catchException,
    );
    let teavm_allocateString = wrap(
        instance
            .find_function::<i32, i32>(store, "teavm_allocateString")
            .context("teavm_allocateString")?,
        teavm_catchException,
    );
    let teavm_stringData = wrap(
        instance
            .find_function::<i32, i32>(store, "teavm_stringData")
            .context("teavm_stringData")?,
        teavm_catchException,
    );
    let teavm_arrayLength = wrap(
        instance
            .find_function::<i32, i32>(store, "teavm_arrayLength")
            .context("teavm_arrayLength")?,
        teavm_catchException,
    );

    let teavm = TeaVM {
        catch_exception: teavm_catchException,
        allocate_string_array: Rc::new(teavm_allocateStringArray),
        object_array_data: Rc::new(teavm_objectArrayData),
        byte_array_data: Rc::new(teavm_byteArrayData),
        allocate_string: Rc::new(teavm_allocateString),
        string_data: Rc::new(teavm_stringData),
        array_length: Rc::new(teavm_arrayLength),
    };
    store.data_mut().teavm = Some(teavm);

    _ = instance
        .link_closure(
            store,
            "teavm",
            "putwcharsOut",
            |mut ctx, (chars, count): (u32, u32)| {
                let mem = ctx.memory_mut();
                let string =
                    str::from_utf8(&mem[chars as usize..(chars + count) as usize]).unwrap();
                print!("{string}");
                Ok(())
            },
        )
        .context("putwcharsOut");

    _ = instance
        .link_closure(
            store,
            "teavm",
            "putwcharsErr",
            |mut ctx, (chars, count): (u32, u32)| {
                let mem = ctx.memory_mut();
                let string =
                    str::from_utf8(&mem[chars as usize..(chars + count) as usize]).unwrap();
                print!("{string}");
                Ok(())
            },
        )
        .context("putwcharsErr");

    let epoch = Instant::now();

    _ = instance
        .link_closure(store, "teavm", "currentTimeMillis", move |_ctx, ()| {
            let secs = epoch.elapsed().as_secs_f64();
            Ok(secs * 1000.0)
        })
        .context("currentTimeMillis");

    _ = instance
        .link_closure(store, "teavm", "logString", move |mut ctx, string: i32| {
            let teavm = ctx.data().teavm.clone().unwrap();
            let array_ptr = (teavm.string_data)(ctx.as_context_mut(), string).unwrap() as usize;
            let len =
                (teavm.array_length)(ctx.as_context_mut(), array_ptr as i32).unwrap() as usize;
            let bytes = len * size_of::<u16>();

            let memory = ctx.memory();
            let array = &memory[array_ptr..array_ptr + bytes];
            let string = String::from_utf16_lossy(bytemuck::cast_slice(array));

            print!("{string}");

            Ok(())
        })
        .context("logString");

    _ = instance
        .link_closure(store, "teavm", "logInt", move |_ctx, int: i32| {
            print!("{int}");
            Ok(())
        })
        .context("logInt");

    _ = instance
        .link_closure(store, "teavm", "logOutOfMemory", move |_ctx, ()| {
            println!("Out of memory");
            Ok(())
        })
        .context("logOutOfMemory");

    Ok(())
}

fn wrap<T: WasmArg, R: WasmType>(
    func: Function<T, R>,
    catch: Function<(), i32>,
) -> impl Fn(StoreContextMut<Data>, T) -> Result<R> {
    move |mut ctx, args| {
        let result = func.call(&mut ctx, args)?;
        let exception = catch.call(&mut ctx)?;
        if exception != 0 {
            panic!("Java code threw an exception");
        }
        Ok(result)
    }
}

type TeaVMDataGetter = dyn Fn(StoreContextMut<Data>, i32) -> Result<i32>;

#[derive(Clone)]
pub struct TeaVM {
    pub catch_exception: Function<(), i32>,
    pub allocate_string_array: Rc<TeaVMDataGetter>,
    pub object_array_data: Rc<TeaVMDataGetter>,
    pub byte_array_data: Rc<TeaVMDataGetter>,
    pub allocate_string: Rc<TeaVMDataGetter>,
    pub string_data: Rc<TeaVMDataGetter>,
    pub array_length: Rc<TeaVMDataGetter>,
}

pub fn teamvm_main(
    store: &mut Store<Data>,
    instance: &mut Instance<Data>,
    args: &[&str],
) -> anyhow::Result<()> {
    let teavm = store.data().teavm.clone().unwrap();

    // all this to make a (String[] args)
    let java_args = (teavm.allocate_string_array)(store.as_context_mut(), args.len() as i32)
        .context("allocating String[]")? as usize;
    let args_bytes = args.len() * size_of::<i32>();
    for (i, &arg) in args.iter().enumerate() {
        let java_arg = (teavm.allocate_string)(store.as_context_mut(), arg.len() as i32)
            .context("allocating String")?;
        let string_data = (teavm.string_data)(store.as_context_mut(), java_arg)
            .context("getting String bytes")?;
        let arg_address = (teavm.object_array_data)(store.as_context_mut(), string_data)
            .context("getting data from String bytes")? as usize;
        let arg_bytes = arg.len() * size_of::<u16>();

        let memory = store.memory_mut();
        let arg_slice: &mut [u16] =
            bytemuck::cast_slice_mut(&mut memory[arg_address..arg_address + arg_bytes]);
        for (i, byte) in arg.encode_utf16().enumerate() {
            arg_slice[i] = byte;
        }

        let args_data: &mut [i32] =
            bytemuck::cast_slice_mut(&mut memory[java_args..java_args + args_bytes]);
        args_data[i] = java_arg;
    }

    let start = instance
        .find_function::<i32, ()>(store, "start")
        .context("getting start function")?;
    start
        .call(&mut *store, java_args as i32)
        .context("calling start function")?;

    let exception = teavm
        .catch_exception
        .call(&mut *store)
        .context("checking start function's exceptions")?;
    if exception != 0 {
        panic!("Java code threw an exception");
    }

    Ok(())
}
