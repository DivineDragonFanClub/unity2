pub mod api;
pub mod assembly;
pub mod class;
pub mod method;

pub use class::{FieldInfo, Il2CppClass, Il2CppReflectionType, PropertyInfo, VirtualInvoke};
pub use method::{MethodInfo, OptionalMethod, ParameterInfo};

#[cfg(feature = "fe-engage")]
pub mod fe_engage {
    pub fn il2cpp_init_scan() -> usize {
        static OFFSET: ::std::sync::LazyLock<usize> = ::std::sync::LazyLock::new(|| {
            let text = lazysimd::scan::get_text();
            lazysimd::get_offset_neon(
                text,
                "fd 7b be a9 f3 0b 00 f9 fd 03 00 91 f3 03 00 aa ?? ?? ?? ?? ?? ?? ?? ?? c0 00 80 52 ?? ?? ?? ?? e0 03 13 aa ?? ?? ?? ?? f3 0b 40 f9 00 00 00 12 fd 7b c2 a8 c0 03 5f d6",
            )
            .expect("il2cpp_init pattern scan failed")
        });

        *OFFSET
    }
}

#[repr(C)]
pub union Il2CppTypeData {
    data: *const u8,
    class_index: i32,
    ty: &'static Il2CppType,
    array: *const u8,
    generic_parameter_index: i32,
    generic_class: *const (),
}

#[repr(C)]
pub struct Il2CppType {
    pub data: Il2CppTypeData,
    pub bits: u32,
}

unsafe impl Send for Il2CppType {}
unsafe impl Sync for Il2CppType {}

#[repr(C)]
pub struct Il2CppGenericClass {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct Il2CppGenericContainer {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct Il2CppRGCTXData {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct Il2CppDomain {
    _opaque: [u8; 0],
}

