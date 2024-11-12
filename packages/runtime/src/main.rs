#![no_main]
#![no_std]

use alloc::borrow::ToOwned;
use core::{
    ffi::{c_char, CStr},
    ptr::addr_of,
    time::Duration,
};

use runtime::{
    platform,
    teavm::{self, teamvm_main},
};
use vexide::{core::program::exit, prelude::*};
use vexide_wasm_startup::{startup, CodeSignature, ProgramFlags, ProgramOwner, ProgramType};

extern crate alloc;

async fn main(_peripherals: Peripherals) {
    let wasm_bytes = platform::read_user_program();

    let env = wasm3::Environment::new().expect("Unable to create environment");
    let mut store = env.create_store(4096).expect("Unable to create runtime");
    let module = env
        .parse_module(wasm_bytes)
        .expect("Unable to parse module");

    let mut instance = store.instantiate(module).expect("Unable to load module");
    teavm::link_teavm(&mut store, &mut instance).expect("Unable to link teavm");
    teavm::teamvm_main(&mut store, &mut instance, &[]).expect("Unable to run main");
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
