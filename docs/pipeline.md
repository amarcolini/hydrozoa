# How Hydrozoa runs a program

## Step 1: Language build tools

Hydrozoa relies on a language's own build tools to create a WebAssembly (WASM) binary that it can load when the user starts their program. We chose WASM because of its wide language support - everything from Rust to JavaScript can be compiled to it. However, the VEX brain does not understand WASM bytecode out of the box, so we include a seperate runtime which does.

In the case of Java, the language was designed to target the Java Virtual Machine (JVM) rather than WASM. This means we need to use a custom compiler, which is, at the moment, [TeaVM](https://teavm.org). If you are using the Hydrozoa Gradle plugin, TeaVM's setup & invocation is handled automatically while running the `build` or `upload` tasks.

## Step 2: Uploading

Hydrozoa programs are bigger than C++ or Rust programs because they include a full interpreter and bindings to the VEX SDK, not to mention the actual user code itself. Thus, the upload process is split into two files: the runtime and the user program. The runtime contains everything that usually doesn't need to be reuploaded, which means the interpreter, VEX startup code, bindings to the VEX SDK, and the TeaVM support library. It current exists on the brain as a single `libmutliv_runtime.bin` file, also this is subject to change. In contrast, the user program contains WASM-formatted data that the runtime is expected to load. Unlike the runtime, there can be multiple versions of a user program in different program slots. While uploading a program with the `hydrozoa` CLI, these upload progress for the runtime is marked with `lib` and the progress of the user program is marked with `bin`.

There tend to be very limited size constraints when uploading to the brain. The `wasm-opt` tool from Binaryen can help shrink the user code when running it like so:

```shell
wasm-opt -Oz -o optimized.wasm robot.wasm
```

This is not run automatically during a Gradle upload because then Binaryen would need to be installed manually by the user.

## Step 3: Startup

Hydrozoa uses a version of vexide-startup which has a modified linker script to support VEXos file linking. Before the program starts, VEXos loads the runtime to `0x03800000` and the user program to `0x07800000`. It then begins execution at `0x03800020`, which is handled by vexide-startup. After vexide-startup passes execution to the runtime, Hydrozoa reads the program from memory, initializes the interpreter (which is current wasm3), and begins execution.

### User program format

VEXos does not include the length of a file when linking it. Thus, the CLI prepends the user program with an integer equal to the program's length before uploading it. The runtime knows about this format and uses it to retrieve the linked wasm file as a Rust `&[u8]` slice.

```txt
        0x07800000
            │
            ▼
┌──────────┐┌───────┐┌───────────────────┐
│End of    ││Program││WASM user program  │
│Runtime   ││Header ││(length: value of  │
│          ││       ││program header).   │
│          ││(u32)  ││                   │
│          ││       ││                   │
│          ││       ││                   │
│          ││       ││                   │
│          ││       ││                   │
└──────────┘└───────┘└───────────────────┘
            ▲
            │
```

## Step 4: Execution

Running a program on a VEX is only useful if it can do something with the VEX hardware, so Hydrozoa's runtime provides functions that WASM programs can import and use to access peripherals.

### The "vex" module

Functions imported from the `"vex"` module are used to access VEX peripherals and control the VEX robot. They are generally direct recreations of VEX SDK functions from the `vex-sdk` Rust crate and are likely similar to those defined in `libv5rts.a`.

For example, the following Rust program could theoretically be compiled to WASM, and when run under Hydrozoa, would draw a square to the VEX's display and then immediately exit:

```rs
#[link(wasm_import_module = "vex")]
unsafe extern "C" {
    fn vexDisplayForegroundColor(col: u32);
    fn vexDisplayRectFill(x1: i32, y1: i32, x2: i32, y2: i32);
}

#[unsafe(no_mangle)]
extern "C" fn start() {
    unsafe {
        vexDisplayForegroundColor(0xFFFFFF);
        vexDisplayRectFill(20, 20, 120, 120);
    }
}
```

While most signatures exported from the `"vex"` module are one-to-one with their `vex-sdk` counterpart, ones that deal in pointers have been slightly modified. For example, the functions that take a `V5_DeviceT` now take a `u32`. If the WASM program attempts to cast this number to a pointer and dereference it, the operation will fail, because WASM is executed in a memory sandbox.

WASM programs *must* call the `vexTasksRun` function from the `"vex"` module at least one time every 2 milliseconds in order to update sensors, perform basic serial I/O, and ensure the correct operation of the runtime.

```rs
#[link(wasm_import_module = "vex")]
unsafe extern "C" {
    fn vexDeviceGetByIndex(index: u32) -> u32;
    fn vexDeviceMotorVoltageSet(device: u32, voltage: i32);
    fn vexTasksRun();
}

#[unsafe(no_mangle)]
extern "C" fn start() {
    unsafe {
        // Get the motor on smart port 1
        let device_handle: u32 = vexDeviceGetByIndex(0);
        // Set it to 12 volts (full power)
        vexDeviceMotorVoltageSet(device_handle, 12 * 1000);
        // Loop so that the program doesn't exit
        loop {
            vexTasksRun();
        }
    }
}
```
