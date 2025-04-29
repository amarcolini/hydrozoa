#![allow(non_snake_case)]

use alloc::{ffi::CString, rc::Rc, string::String};
use core::str;

use anyhow::{Context, Result};
use vexide::{
    float::Float,
    io::{print, println},
    time::Instant,
};
use wasm3::{
    store::{AsContextMut, StoreContextMut},
    Function, Instance, Store,
};

use crate::{platform::flush_serial, Data};

pub fn link_teavm(store: &mut Store<Data>, instance: &mut Instance<Data>) -> Result<()> {
    let teavm = TeaVM {
        catch_exception: instance
            .find_function::<(), i32>(store, "teavm_catchException")
            .context("finding teavm interop function")?,
        allocate_string_array: wrap(&mut *store, &mut *instance, "teavm_allocateStringArray")?,
        object_array_data: wrap(&mut *store, &mut *instance, "teavm_objectArrayData")?,
        byte_array_data: wrap(&mut *store, &mut *instance, "teavm_byteArrayData")?,
        allocate_string: wrap(&mut *store, &mut *instance, "teavm_allocateString")?,
        string_data: wrap(&mut *store, &mut *instance, "teavm_stringData")?,
        array_length: wrap(&mut *store, &mut *instance, "teavm_arrayLength")?,
        short_array_data: wrap(&mut *store, &mut *instance, "teavm_shortArrayData")?,
        char_array_data: wrap(&mut *store, &mut *instance, "teavm_charArrayData")?,
        int_array_data: wrap(&mut *store, &mut *instance, "teavm_intArrayData")?,
        long_array_data: wrap(&mut *store, &mut *instance, "teavm_longArrayData")?,
        float_array_data: wrap(&mut *store, &mut *instance, "teavm_floatArrayData")?,
        double_array_data: wrap(&mut *store, &mut *instance, "teavm_doubleArrayData")?,
    };
    store.data_mut().teavm = Some(teavm);

    instance.link_closure(
        store,
        "teavm",
        "putwcharsOut",
        |mut ctx, (chars, count): (u32, u32)| {
            let mem = ctx.memory_mut();
            let string = str::from_utf8(&mem[chars as usize..(chars + count) as usize]).unwrap();
            print!("{string}");
            Ok(())
        },
    )?;

    instance.link_closure(
        store,
        "teavm",
        "putwcharsErr",
        |mut ctx, (chars, count): (u32, u32)| {
            let mem = ctx.memory_mut();
            let string = str::from_utf8(&mem[chars as usize..(chars + count) as usize]).unwrap();
            print!("{string}");
            Ok(())
        },
    )?;

    let epoch = Instant::now();

    instance.link_closure(store, "teavm", "currentTimeMillis", move |_ctx, ()| {
        let secs = epoch.elapsed().as_secs_f64();
        Ok(secs * 1000.0)
    })?;

    instance.link_closure(store, "teavm", "nanoTime", move |_ctx, ()| {
        let nanos = epoch.elapsed().as_nanos() as f64 / 1000000.0;
        Ok(nanos)
    })?;

    instance.link_closure(store, "teavmMath", "sin", move |_ctx, a: f64| Ok(a.sin()))?;
    instance.link_closure(store, "teavmMath", "cos", move |_ctx, a: f64| Ok(a.cos()))?;
    instance.link_closure(store, "teavmMath", "tan", move |_ctx, a: f64| Ok(a.tan()))?;
    instance.link_closure(store, "teavmMath", "asin", move |_ctx, a: f64| Ok(a.asin()))?;
    instance.link_closure(store, "teavmMath", "acos", move |_ctx, a: f64| Ok(a.acos()))?;
    instance.link_closure(store, "teavmMath", "atan", move |_ctx, a: f64| Ok(a.atan()))?;
    instance.link_closure(store, "teavmMath", "exp", move |_ctx, a: f64| Ok(a.exp()))?;
    instance.link_closure(store, "teavmMath", "log", move |_ctx, a: f64| Ok(a.ln()))?;
    instance.link_closure(store, "teavmMath", "sqrt", move |_ctx, a: f64| Ok(a.sqrt()))?;
    instance.link_closure(store, "teavmMath", "ceil", move |_ctx, a: f64| Ok(a.ceil()))?;
    instance.link_closure(store, "teavmMath", "floor", move |_ctx, a: f64| {
        Ok(a.floor())
    })?;
    instance.link_closure(
        store,
        "teavmMath",
        "pow",
        move |_ctx, (x, y): (f64, f64)| Ok(x.powf(y)),
    )?;
    instance.link_closure(
        store,
        "teavmMath",
        "atan2",
        move |_ctx, (y, x): (f64, f64)| Ok(y.atan2(x)),
    )?;


    instance.link_closure(store, "teavm", "logString", move |mut ctx, string: i32| {
        let string = get_string(&mut ctx, string);

        print!("{string}");

        Ok(())
    })?;

    instance.link_closure(store, "teavm", "logInt", move |_ctx, int: i32| {
        print!("{int}");
        Ok(())
    })?;

    instance.link_closure(store, "teavm", "logOutOfMemory", move |_ctx, ()| {
        println!("Out of memory");
        Ok(())
    })?;

    Ok(())
}

/// Copies a UTF16 string out of the JVM's memory and into a Rust [`String`].
pub fn get_string(ctx: &mut wasm3::CallContext<Data>, string: i32) -> String {
    let teavm = ctx.data().teavm.clone().unwrap();

    // get pointer & length of the utf16 buffer java stores strings in
    let array = (teavm.string_data)(ctx.as_context_mut(), string).unwrap();
    let len = (teavm.array_length)(ctx.as_context_mut(), array).unwrap() as usize;
    let bytes = len * size_of::<u16>();
    let array_addr = (teavm.char_array_data)(ctx.as_context_mut(), array).unwrap() as usize;

    let memory = ctx.memory();
    let array = &memory[array_addr..array_addr + bytes];
    String::from_utf16_lossy(bytemuck::cast_slice(array))
}

/// Copies a UTF16 string out of the JVM's memory and into a Rust [`CString`].
pub fn get_cstring(ctx: &mut wasm3::CallContext<Data>, string: i32) -> CString {
    CString::new(get_string(ctx, string)).unwrap()
}

type TeaVMDataGetter = dyn Fn(StoreContextMut<Data>, i32) -> Result<i32>;

fn wrap(
    store: &mut Store<Data>,
    instance: &mut Instance<Data>,
    func: &str,
) -> Result<Rc<TeaVMDataGetter>> {
    let teavm_catchException = instance
        .find_function::<(), i32>(store, "teavm_catchException")
        .context("finding teavm interop function")?;
    let func = instance
        .find_function::<i32, i32>(store, func)
        .context("finding teavm interop function")?;

    Ok(Rc::new(move |mut ctx, args| {
        let result = func.call(&mut ctx, args)?;
        let exception = teavm_catchException.call(&mut ctx)?;
        if exception != 0 {
            panic!("Java code threw an exception");
        }
        Ok(result)
    }))
}

#[derive(Clone)]
pub struct TeaVM {
    pub catch_exception: Function<(), i32>,
    pub allocate_string_array: Rc<TeaVMDataGetter>,
    pub object_array_data: Rc<TeaVMDataGetter>,
    pub byte_array_data: Rc<TeaVMDataGetter>,
    pub short_array_data: Rc<TeaVMDataGetter>,
    pub char_array_data: Rc<TeaVMDataGetter>,
    pub int_array_data: Rc<TeaVMDataGetter>,
    pub long_array_data: Rc<TeaVMDataGetter>,
    pub float_array_data: Rc<TeaVMDataGetter>,
    pub double_array_data: Rc<TeaVMDataGetter>,
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

    flush_serial();

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
