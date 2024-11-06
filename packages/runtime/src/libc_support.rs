use alloc::string::String;
use core::{
    alloc::Layout,
    ffi::{c_char, c_int, VaList, VaListImpl},
};

use hashbrown::HashMap;
use vexide::core::{
    println,
    sync::{LazyLock, Mutex},
};

// these really get more unhinged the more you read

#[allow(non_upper_case_globals)]
const max_align_t: usize = 16;

static LAYOUTS: LazyLock<Mutex<HashMap<usize, Layout>>> = LazyLock::new(Mutex::default);

#[no_mangle]
extern "C" fn calloc(nmemb: usize, size: usize) -> *mut u8 {
    let layout = Layout::from_size_align(size * nmemb, max_align_t).unwrap();
    if layout.size() == 0 {
        return core::ptr::null_mut();
    }

    let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
    if ptr.is_null() {
        return ptr;
    }

    let mut layouts = LAYOUTS.try_lock().unwrap();
    layouts.insert(ptr as usize, layout);

    ptr
}

#[no_mangle]
extern "C" fn free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    let mut layouts = LAYOUTS.try_lock().unwrap();
    let layout = layouts
        .remove(&(ptr as usize))
        .expect("double free detected");
    unsafe { alloc::alloc::dealloc(ptr, layout) };
}

#[no_mangle]
extern "C" fn realloc(ptr: *mut u8, size: usize) -> *mut u8 {
    if ptr.is_null() {
        return calloc(1, size);
    }

    let mut layouts = LAYOUTS.try_lock().unwrap();
    let layout = layouts
        .remove(&(ptr as usize))
        .expect("realloc on unknown pointer");
    let new_layout = Layout::from_size_align(size, layout.align()).unwrap();
    let new_ptr = unsafe { alloc::alloc::realloc(ptr, layout, new_layout.size()) };
    if new_ptr.is_null() {
        return new_ptr;
    }

    layouts.insert(new_ptr as usize, new_layout);

    new_ptr
}

#[no_mangle]
extern "C" fn strcmp(s1: *const u8, s2: *const u8) -> i32 {
    let mut i = 0;
    loop {
        let c1 = unsafe { *s1.add(i) };
        let c2 = unsafe { *s2.add(i) };
        if c1 == 0 && c2 == 0 {
            return 0;
        } else if c1 == 0 {
            return -1;
        } else if c2 == 0 {
            return 1;
        } else if c1 != c2 {
            return c1 as i32 - c2 as i32;
        }
        i += 1;
    }
}

#[no_mangle]
unsafe extern "C" fn vsnprintf(
    buffer: *mut u8,
    bufsz: usize,
    format: *const u8,
    args: VaList,
) -> c_int {
    let mut s = String::new();
    let bytes_written =
        printf_compat::format(format, args, printf_compat::output::fmt_write(&mut s));

    s.truncate(bufsz - 1);
    s.push('\0');

    let bytes_written = bytes_written.min(bufsz as i32 - 1);
    core::ptr::copy_nonoverlapping(s.as_ptr(), buffer, bytes_written as usize);
    bytes_written
}

#[no_mangle]
extern "C" fn __popcountsi2(a: i32) -> i32 {
    let x = a as u32;
    let x = x - ((x >> 1) & 0x55555555);
    let x = ((x >> 2) & 0x33333333) + (x & 0x33333333);
    let x = (x + (x >> 4)) & 0x0F0F0F0F;
    let x = x + (x >> 16);
    ((x + (x >> 8)) & 0x0000003F) as i32
}

#[no_mangle]
extern "C" fn __popcountdi2(a: i64) -> i64 {
    let x = a as u64;
    let x = x - ((x >> 1) & 0x5555555555555555);
    let x = ((x >> 2) & 0x3333333333333333) + (x & 0x3333333333333333);
    let x = (x + (x >> 4)) & 0x0F0F0F0F0F0F0F0F;
    let x = x + (x >> 32);
    let x = x + (x >> 16);
    ((x + (x >> 8)) & 0x0000007F) as i64
}
