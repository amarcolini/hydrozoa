#![no_main]
#![no_std]
#![feature(c_variadic)]

use alloc::borrow::ToOwned;
use core::{
    ffi::{c_char, CStr},
    ptr::addr_of,
    time::Duration,
};

use vexide::{core::program::exit, prelude::*};
use vexide_wasm_startup::{startup, CodeSignature, ProgramFlags, ProgramOwner, ProgramType};

extern crate alloc;

mod libc_support;

unsafe extern "C" {
    static __user_ram_end: c_char;
}

async fn main(_peripherals: Peripherals) {
    println!("Starting...");
    println!("Link Addr: 0x{:x?}", unsafe {
        vex_sdk::vexSystemLinkAddrGet()
    });
    sleep(Duration::from_secs(1)).await;

    // unsafe {
    //     let lib_start = addr_of!(__user_ram_end);
    //     let string = CStr::from_ptr(lib_start);
    //     println!("String: {string:?}");
    // }

    // sleep(Duration::from_secs(1)).await;

    let env = wasm3::Environment::new().expect("Unable to create environment");
    let rt = env.create_runtime(1024).expect("Unable to create runtime");
    let module = env
        .parse_module(include_bytes!("../../test-program/test_program.wasm").to_owned())
        .expect("Unable to parse module");

    let instance = rt.load_module(module).expect("Unable to load module");
    let func = instance
        .find_function::<(i32, i32), i32>("add")
        .expect("Unable to find function");
    println!("Wasm says that 3 + 6 is {}", func.call(3, 6).unwrap())
}

#[link_section = ".code_signature"]
#[used]
static CODE_SIGNATURE: CodeSignature = CodeSignature::new(
    ProgramType::User,
    ProgramOwner::Partner,
    ProgramFlags::empty(),
);

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    startup();
    block_on(main(Peripherals::take().unwrap()));
    exit();
}
