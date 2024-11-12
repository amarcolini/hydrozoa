use wasm3::{Environment, Instance};

fn main() {
    let env = Environment::new().expect("Unable to create environment");
    let rt = env
        .create_store(1024 * 60)
        .expect("Unable to create runtime");
    let module = Instance::parse(&env, &include_bytes!("wasm/wasm_add/wasm_add.wasm")[..])
        .expect("Unable to parse module");

    let module = rt.instantiate(module).expect("Unable to load module");
    let func = module
        .find_function::<(i64, i64), i64>("add")
        .expect("Unable to find function");
    println!("Wasm says that 3 + 6 is {}", func.call(3, 6).unwrap())
}
