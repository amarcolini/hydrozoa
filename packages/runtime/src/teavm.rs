#![allow(non_snake_case)]

use alloc::string::String;
use core::str;

use vexide::core::{print, println, time::Instant};
use wasm3::{error::Trap, Function, Instance, Store, WasmArg, WasmArgs, WasmType};

pub fn link_teavm(store: &mut Store, instance: &mut Instance) -> wasm3::error::Result<()> {
    let teavm_stringData = instance.find_function::<i32, i32>(store, "teavm_stringData")?;
    let teavm_arrayLength = instance.find_function::<i32, i32>(store, "teavm_arrayLength")?;

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

    let epoch = Instant::now();

    instance.link_closure(store, "teavm", "currentTimeMillis", move |_ctx, ()| {
        let secs = epoch.elapsed().as_secs_f64();
        Ok(secs * 1000.0)
    })?;

    instance.link_closure(store, "teavm", "logString", move |mut ctx, string: i32| {
        let array_ptr = teavm_stringData.call(&mut ctx, string).unwrap() as usize;
        let len = teavm_arrayLength.call(&mut ctx, array_ptr as i32).unwrap() as usize;
        let bytes = len * size_of::<u16>();

        let memory = ctx.memory();
        let array = &memory[array_ptr..array_ptr + bytes];
        let string = String::from_utf16_lossy(bytemuck::cast_slice(array));

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

fn wrap<T: WasmArg, R: WasmType>(
    func: Function<T, R>,
    catch: Function<(), i32>,
) -> impl Fn(&mut Store, T) -> wasm3::error::Result<R> {
    move |store, args| {
        let result = func.call(&mut *store, args)?;
        let exception = catch.call(&mut *store)?;
        if exception != 0 {
            panic!("Java code threw an exception");
        }
        Ok(result)
    }
}

pub fn teamvm_main(
    store: &mut Store,
    instance: &mut Instance,
    args: &[&str],
) -> wasm3::error::Result<()> {
    let teavm_catchException = instance.find_function::<(), i32>(store, "teavm_catchException")?;
    let teavm_allocateStringArray = wrap(
        instance.find_function::<i32, i32>(store, "teavm_allocateStringArray")?,
        teavm_catchException,
    );
    let teavm_objectArrayData = wrap(
        instance.find_function::<i32, i32>(store, "teavm_objectArrayData")?,
        teavm_catchException,
    );
    let teavm_allocateString = wrap(
        instance.find_function::<i32, i32>(store, "teavm_allocateString")?,
        teavm_catchException,
    );
    let teavm_stringData = wrap(
        instance.find_function::<i32, i32>(store, "teavm_stringData")?,
        teavm_catchException,
    );

    // all this to make a (String[] args)

    let java_args = teavm_allocateStringArray(&mut *store, args.len() as i32)? as usize;
    let args_bytes = args.len() * size_of::<i32>();
    for (i, &arg) in args.iter().enumerate() {
        let java_arg = teavm_allocateString(&mut *store, arg.len() as i32)?;
        let string_data = teavm_stringData(&mut *store, java_arg)?;
        let arg_address = teavm_objectArrayData(&mut *store, string_data)? as usize;
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

    let start = instance.find_function::<i32, ()>(store, "start")?;
    start.call(&mut *store, java_args as i32)?;

    let exception = teavm_catchException.call(&mut *store)?;
    if exception != 0 {
        panic!("Java code threw an exception");
    }

    Ok(())
}
