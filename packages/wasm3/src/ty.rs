#![allow(missing_docs, clippy::missing_safety_doc)]
use alloc::vec::Vec;
use core::ffi::c_char;

trait Sealed {}

/// Trait implemented by types that can be passed to and from wasm.
#[allow(private_bounds)]
pub trait WasmType: Sized + Sealed {
    const TYPE_INDEX: ffi::M3ValueType::Type;
    const SIZE_IN_SLOT_COUNT: usize;
    const SIGNATURE: u8;
    unsafe fn pop_from_stack(stack: *mut u64) -> Self;
    unsafe fn push_on_stack(self, stack: *mut u64);
}

/// Tait implemented by types that can be passed to wasm.
pub trait WasmArg: WasmType {}

/// Helper tait implemented by tuples to emulate "variadic generics".
#[allow(private_bounds)]
pub trait WasmArgs: Sealed {
    unsafe fn push_on_stack(self, stack: *mut u64);
    // required for closure linking
    unsafe fn pop_from_stack(stack: *mut u64) -> Self;
    fn validate_types(types: impl Iterator<Item = ffi::M3ValueType::Type>) -> bool;
    fn append_signature(buffer: &mut Vec<c_char>);
}

impl Sealed for i32 {}
impl WasmArg for i32 {}
impl WasmType for i32 {
    const TYPE_INDEX: ffi::M3ValueType::Type = ffi::M3ValueType::c_m3Type_i32;
    const SIZE_IN_SLOT_COUNT: usize = 1;
    const SIGNATURE: u8 = b'i';
    unsafe fn pop_from_stack(stack: *mut u64) -> Self {
        *(stack as *const i32)
    }
    unsafe fn push_on_stack(self, stack: *mut u64) {
        *(stack as *mut i32) = self;
    }
}

impl Sealed for u32 {}
impl WasmArg for u32 {}
impl WasmType for u32 {
    const TYPE_INDEX: ffi::M3ValueType::Type = ffi::M3ValueType::c_m3Type_i32;
    const SIZE_IN_SLOT_COUNT: usize = 1;
    const SIGNATURE: u8 = b'i';
    unsafe fn pop_from_stack(stack: *mut u64) -> Self {
        *(stack as *const u32)
    }
    unsafe fn push_on_stack(self, stack: *mut u64) {
        *(stack as *mut u32) = self;
    }
}

impl Sealed for bool {}
impl WasmArg for bool {}
impl WasmType for bool {
    const TYPE_INDEX: ffi::M3ValueType::Type = ffi::M3ValueType::c_m3Type_i32;
    const SIZE_IN_SLOT_COUNT: usize = 1;
    const SIGNATURE: u8 = b'i';
    unsafe fn pop_from_stack(stack: *mut u64) -> Self {
        *(stack as *const i32) != 0
    }
    unsafe fn push_on_stack(self, stack: *mut u64) {
        *(stack as *mut i32) = self as i32;
    }
}

impl Sealed for i64 {}
impl WasmArg for i64 {}
impl WasmType for i64 {
    const TYPE_INDEX: ffi::M3ValueType::Type = ffi::M3ValueType::c_m3Type_i64;
    const SIZE_IN_SLOT_COUNT: usize = 1;
    const SIGNATURE: u8 = b'I';
    unsafe fn pop_from_stack(stack: *mut u64) -> Self {
        *(stack as *const i64)
    }
    unsafe fn push_on_stack(self, stack: *mut u64) {
        *(stack as *mut i64) = self;
    }
}

impl Sealed for u64 {}
impl WasmArg for u64 {}
impl WasmType for u64 {
    const TYPE_INDEX: ffi::M3ValueType::Type = ffi::M3ValueType::c_m3Type_i64;
    const SIZE_IN_SLOT_COUNT: usize = 1;
    const SIGNATURE: u8 = b'I';
    unsafe fn pop_from_stack(stack: *mut u64) -> Self {
        *stack
    }
    unsafe fn push_on_stack(self, stack: *mut u64) {
        *stack = self;
    }
}

impl Sealed for f32 {}
impl WasmArg for f32 {}
impl WasmType for f32 {
    const TYPE_INDEX: ffi::M3ValueType::Type = ffi::M3ValueType::c_m3Type_f32;
    const SIZE_IN_SLOT_COUNT: usize = 1;
    const SIGNATURE: u8 = b'f';
    unsafe fn pop_from_stack(stack: *mut u64) -> Self {
        f32::from_ne_bytes((*(stack as *const u32)).to_ne_bytes())
    }
    unsafe fn push_on_stack(self, stack: *mut u64) {
        *(stack as *mut u32) = u32::from_ne_bytes(self.to_ne_bytes());
    }
}

impl Sealed for f64 {}
impl WasmArg for f64 {}
impl WasmType for f64 {
    const TYPE_INDEX: ffi::M3ValueType::Type = ffi::M3ValueType::c_m3Type_f64;
    const SIZE_IN_SLOT_COUNT: usize = 1;
    const SIGNATURE: u8 = b'F';
    unsafe fn pop_from_stack(stack: *mut u64) -> Self {
        f64::from_ne_bytes((*stack).to_ne_bytes())
    }
    unsafe fn push_on_stack(self, stack: *mut u64) {
        *stack = u64::from_ne_bytes(self.to_ne_bytes());
    }
}

impl Sealed for () {}
impl WasmType for () {
    const TYPE_INDEX: ffi::M3ValueType::Type = ffi::M3ValueType::c_m3Type_none;
    const SIZE_IN_SLOT_COUNT: usize = 0;
    const SIGNATURE: u8 = b'v';
    unsafe fn pop_from_stack(_: *mut u64) -> Self {}
    unsafe fn push_on_stack(self, _: *mut u64) {}
}

impl WasmArgs for () {
    unsafe fn push_on_stack(self, _: *mut u64) {}
    unsafe fn pop_from_stack(_: *mut u64) -> Self {}
    fn validate_types(mut types: impl Iterator<Item = ffi::M3ValueType::Type>) -> bool {
        types.next().is_none()
    }
    fn append_signature(_buffer: &mut Vec<c_char>) {}
}

/// Unary functions
impl<T> WasmArgs for T
where
    T: WasmArg,
{
    unsafe fn push_on_stack(self, stack: *mut u64) {
        WasmType::push_on_stack(self, stack);
    }
    unsafe fn pop_from_stack(stack: *mut u64) -> Self {
        WasmType::pop_from_stack(stack)
    }
    fn validate_types(mut types: impl Iterator<Item = ffi::M3ValueType::Type>) -> bool {
        types.next().map(|ty| ty == T::TYPE_INDEX).unwrap_or(false) && types.next().is_none()
    }
    fn append_signature(buffer: &mut Vec<c_char>) {
        buffer.push(T::SIGNATURE as c_char);
    }
}

macro_rules! args_impl {
    ($($types:ident),*) => { args_impl!(@rec [$($types,)*] []); };
    (@rec [] [$($types:ident,)*]) => { args_impl!(@do_impl $($types,)*); };
    (@rec [$head:ident, $($tail:ident,)*] [$($types:ident,)*]) => {
        args_impl!(@do_impl $($types,)*);
        args_impl!(@rec [$($tail,)*] [$($types,)* $head,]);
    };
    (@do_impl) => {/* catch the () case, since its implementation differs slightly */};
    (@do_impl $($types:ident,)*) => {
        impl<$($types,)*> Sealed for ($($types,)*) {}
        #[allow(clippy::mixed_read_write_in_expression)]
        #[allow(unused_assignments)]
        impl<$($types,)*> WasmArgs for ($($types,)*)
        where $($types: WasmArg,)* {
                    unsafe fn push_on_stack(self, mut stack: *mut u64) {
                #[allow(non_snake_case)]
                let ($($types,)*) = self;

                $(
                    $types.push_on_stack(stack);
                    stack = stack.add($types::SIZE_IN_SLOT_COUNT);
                )*
            }
                    unsafe fn pop_from_stack(mut stack: *mut u64) -> Self {
                ($(
                    {
                        let val = $types::pop_from_stack(stack);
                        stack = stack.add($types::SIZE_IN_SLOT_COUNT);
                        val
                    },
                )*)
            }
                    fn validate_types(mut types: impl Iterator<Item=ffi::M3ValueType::Type>) -> bool {
                $(
                    types.next().map(|ty| ty == $types::TYPE_INDEX).unwrap_or(false) &&
                )*
                types.next().is_none()
            }
                    fn append_signature(buffer: &mut Vec<c_char>) {
                $(
                    buffer.push($types::SIGNATURE as c_char);
                )*
            }
        }
    };
}
args_impl!(A, B, C, D, E, F, G, H, J, K, L, M, N, O, P, Q);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_validate_types_single() {
        assert!(f64::validate_types(
            [ffi::M3ValueType::c_m3Type_f64,].iter().cloned()
        ));
    }

    #[test]
    fn test_validate_types_single_fail() {
        assert!(!f32::validate_types(
            [ffi::M3ValueType::c_m3Type_f64,].iter().cloned()
        ));
    }

    #[test]
    fn test_validate_types_double() {
        assert!(<(f64, u32)>::validate_types(
            [
                ffi::M3ValueType::c_m3Type_f64,
                ffi::M3ValueType::c_m3Type_i32,
            ]
            .iter()
            .cloned()
        ));
    }

    #[test]
    fn test_validate_types_double_fail() {
        assert!(!<(f32, u64)>::validate_types(
            [
                ffi::M3ValueType::c_m3Type_i64,
                ffi::M3ValueType::c_m3Type_f32,
            ]
            .iter()
            .cloned()
        ));
    }

    #[test]
    fn test_validate_types_quintuple() {
        assert!(<(f64, u32, i32, i64, f32)>::validate_types(
            [
                ffi::M3ValueType::c_m3Type_f64,
                ffi::M3ValueType::c_m3Type_i32,
                ffi::M3ValueType::c_m3Type_i32,
                ffi::M3ValueType::c_m3Type_i64,
                ffi::M3ValueType::c_m3Type_f32,
            ]
            .iter()
            .cloned()
        ));
    }

    #[test]
    fn test_validate_types_quintuple_fail() {
        assert!(!<(f64, u32, i32, i64, f32)>::validate_types(
            [
                ffi::M3ValueType::c_m3Type_i32,
                ffi::M3ValueType::c_m3Type_i64,
                ffi::M3ValueType::c_m3Type_i32,
                ffi::M3ValueType::c_m3Type_f32,
                ffi::M3ValueType::c_m3Type_f64,
            ]
            .iter()
            .cloned()
        ));
    }
}
