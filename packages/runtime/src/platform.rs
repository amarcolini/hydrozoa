use core::ptr;

const LINKED_FILE: *const u32 = 0x7800000 as *const u32;

pub fn read_user_program() -> &'static [u8] {
    unsafe {
        let len = ptr::read_volatile(LINKED_FILE);
        let file_base: *const u8 = LINKED_FILE.offset(1).cast();
        core::slice::from_raw_parts(file_base, len as usize)
    }
}

pub fn flush_serial() {
    while unsafe { vex_sdk::vexSerialWriteFree(1) < 2048 } {
        unsafe {
            vex_sdk::vexTasksRun();
        }
    }
}
