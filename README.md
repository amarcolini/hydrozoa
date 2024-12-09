# Hydrozoa Runtime

> Run WebAssembly programs on VEX V5 brains

## About

Hydrozoa Runtime is the behind-the-scenes magic that makes it possible to use interpreted languages like Java and Kotlin on VEX Robots. It uses a WebAssembly interpreter that runs directly on the VEX V5 Brain to run programs, and defines a set of functions the program can use to interface with the VEX SDK.

## Building

Hydrozoa programs need the `hydrozoa.bin` runtime file to upload their code to a robot, which can be generated by building this repository.

To begin, ensure you have both the Rust programming language (`cargo`) and the ARM Embedded Toolchain (`arm-none-eabi-gcc`) available on your system.

This project also depends on `cargo-v5`, which you can install by running this command:

```shell
cargo install cargo-v5
```

Clone the repository with the `--recurse-submodules` flag to ensure you have its submodules as well:

```shell
git clone https://github.com/vexide/hydrozoa.git --recurse-submodules
```

Then, run this command to build the project:

```shell
cargo v5 build --release
```

The resulting `hydrozoa.bin` file is located in `./target/armv7a-vex-v5/release/`.
