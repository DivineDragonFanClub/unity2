use lazysimd;

use super::{
    assembly::{CppVector, Il2CppAssembly, Il2CppImage},
    class::Il2CppClass,
    method::MethodInfo,
    Il2CppType,
};
use crate::system::SystemType;

#[lazysimd::from_pattern(
    "fd 7b be a9 f3 0b 00 f9 fd 03 00 91 f3 03 00 aa ?? ?? ?? ?? ?? ?? ?? ?? c0 00 80 52 ?? ?? ?? ?? e0 03 13 aa ?? ?? ?? ?? f3 0b 40 f9 00 00 00 12 fd 7b c2 a8 c0 03 5f d6"
)]
pub(crate) fn init(domain_name: *const i8) -> i32;

#[lazysimd::from_pattern(
    "ff c3 02 d1 fd 7b 05 a9 fd 43 01 91 fc 6f 06 a9 fa 67 07 a9 f8 5f 08 a9 f6 57 09 a9 f4 4f 0a a9 f8 03 00 aa 16 0f 43 f8 f9 03 02 aa f4 03 01 aa"
)]
pub(crate) fn class_from_name(
    image: &Il2CppImage,
    namespace: *const u8,
    name: *const u8,
) -> Option<&'static mut Il2CppClass>;

#[lazysimd::from_pattern(
    "ff 43 01 d1 fd 7b 01 a9 fd 43 00 91 f8 5f 02 a9 f6 57 03 a9 f4 4f 04 a9 08 c8 44 39 f3 03 03 2a f6 03 02 2a f4 03 01 aa f5 03 00 aa"
)]
pub(crate) fn get_method_from_name_flags(
    class: &Il2CppClass,
    method_name: *const u8,
    args_count: usize,
    flags: u32,
) -> Option<&'static mut MethodInfo>;

#[skyline::from_offset(0x42911c)]
pub(crate) fn assembly_getallassemblies() -> &'static CppVector<&'static Il2CppAssembly>;

#[lazysimd::from_pattern(
    "ff 03 01 d1 fd 7b 01 a9 fd 43 00 91 f5 13 00 f9 f4 4f 03 a9 ?? ?? ?? ?? ?? ?? ?? ?? a0 0f 00 f9 e0 03 13 aa ff 07 00 f9"
)]
pub(crate) fn type_get_object(ty: &Il2CppType) -> SystemType;

#[lazysimd::from_pattern(
    "ff 03 01 d1 fd 7b 01 a9 fd 43 00 91 f6 57 02 a9 f4 4f 03 a9 f3 03 00 aa e0 03 1f aa 68 2a 40 39 08 05 00 51 1f 75 00 71"
)]
pub(crate) fn class_from_il2cpptype(ty: &Il2CppType) -> Option<&'static mut Il2CppClass>;

// Required after resolving a class from a generic Il2CppType, finalizes field/method/vtable metadata
#[lazysimd::from_pattern(
    "fd 7b bd a9 f5 0b 00 f9 fd 03 00 91 f4 4f 02 a9 08 c8 44 39 08 03 10 37 ?? ?? ?? ?? ?? ?? ?? ?? f3 03 00 aa b5 0f 00 f9"
)]
pub(crate) fn class_init(class: &Il2CppClass);

// Sets the class header only, callers still need to invoke a .ctor method
#[lazysimd::from_pattern(
    "ff 43 01 d1 fd 7b 01 a9 fd 43 00 91 f7 13 00 f9 f6 57 03 a9 f4 4f 04 a9 08 c8 44 39 f3 03 00 aa e8 02 10 37"
)]
pub(crate) fn object_new(klass: &Il2CppClass) -> crate::IlInstance;

// kind 0 is Normal (scanned), 1 is Atomic (not scanned), class cloning uses Normal
#[skyline::from_offset(0x474370)]
pub(crate) fn gc_malloc_kind(size: usize, kind: u32) -> *mut u8;

#[lazysimd::from_pattern(
    "fd 7b be a9 f3 0b 00 f9 fd 03 00 91 f3 03 01 aa 21 00 80 52 e2 03 1f 2a ?? ?? ?? ?? e1 03 13 aa f3 0b 40 f9 fd 7b c2 a8"
)]
pub(crate) fn array_new<T: Copy>(
    element_typeinfo: &Il2CppClass,
    length: usize,
) -> crate::Array<T>;

#[lazysimd::from_pattern(
    "ff 03 01 d1 fd 7b 02 a9 fd 83 00 91 f4 4f 03 a9 f3 03 00 aa ?? ?? ?? ?? 01 7c 40 92 e8 23 00 91 e0 03 13 aa f4 23 00 91 ?? ?? ?? ?? e8 23 40 39 0b fd 41 d3 e9 0f 40 f9"
)]
pub(crate) fn string_new(c_str: *const u8) -> crate::Il2CppString;

// length 0 returns the s_EmptyString static instance, do NOT write into it
#[skyline::from_offset(0x44a168)]
pub(crate) fn string_new_size(length: i32, method_info: crate::OptionalMethod) -> crate::Il2CppString;

// Reversed so game-specific assemblies beat Unity and mscorlib
pub(crate) fn get_class_from_name(
    namespace: impl AsRef<str>,
    name: impl AsRef<str>,
) -> Option<&'static mut Il2CppClass> {
    super::assembly::get_assemblies().iter().rev().find_map(|assembly| {
        let namespace = std::ffi::CString::new(namespace.as_ref()).ok()?;
        let name = std::ffi::CString::new(name.as_ref()).ok()?;
        unsafe {
            class_from_name(
                assembly.image,
                namespace.as_ptr() as _,
                name.as_ptr() as _,
            )
        }
    })
}
