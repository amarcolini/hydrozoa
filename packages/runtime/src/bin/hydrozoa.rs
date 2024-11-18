#![no_main]
#![no_std]

use anyhow::Context;
use runtime::{platform, sdk, teavm, Data};
use vexide::{core::program::exit, prelude::*};
use vexide_wasm_startup::{startup, CodeSignature, ProgramFlags, ProgramOwner, ProgramType};
use wasm3::{Environment, Store};

extern crate alloc;

fn main(_peripherals: Peripherals) {
    let env = wasm3::Environment::new().expect("Unable to create environment");
    let mut store = env
        .create_store(8192, Data::default())
        .expect("Unable to create runtime");

    if let Err(mut err) = run(&env, &mut store) {
        if let Some(info) = store.take_error_info() {
            err = err.context(info);
        }
        println!("\nError: {:?}", err);
    }
}

fn run(env: &Environment, store: &mut Store<Data>) -> anyhow::Result<()> {
    let wasm_bytes = platform::read_user_program();
    let module = env
        .parse_module(wasm_bytes)
        .context("Unable to parse module")?;

    let mut instance = store.instantiate(module).context("Unable to load module")?;

    teavm::link_teavm(&mut *store, &mut instance).context("Unable to link teavm")?;
    sdk::link(&mut *store, &mut instance).context("Unable to link sdk")?;

    teavm::teamvm_main(&mut *store, &mut instance, &[]).context("Unable to run main")?;

    Ok(())
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
    main(Peripherals::take().unwrap());
    exit();
}
