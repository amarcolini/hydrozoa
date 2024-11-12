use wasm3::{
    error::{Error, Trap},
    Environment, Instance,
};

fn main() {
    let env = Environment::new().expect("Unable to create environment");
    let rt = env
        .create_store(1024 * 60)
        .expect("Unable to create runtime");
    let module = Instance::parse(
        &env,
        &include_bytes!("wasm/wasm_millis_to_seconds/wasm_millis_to_seconds.wasm")[..],
    )
    .expect("Unable to parse module");

    let mut module = rt.instantiate(module).expect("Unable to load module");
    module
        .link_closure("time", "millis", |_, ()| Err::<u64, _>(Trap::Abort))
        .expect("Unable to link closure");
    let func = module
        .find_function::<(), u64>("seconds")
        .expect("Unable to find function");

    let err = func.call().unwrap_err();
    match err {
        Error::Wasm3(e) if e.is_trap(Trap::Abort) => {
            println!("got expected error: {}", e);
        }
        _ => {
            panic!("unexpected error: {}", err)
        }
    }
}
