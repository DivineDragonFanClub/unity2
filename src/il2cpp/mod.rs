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
    pub data: *const u8,
    pub class_index: i32,
    pub ty: &'static Il2CppType,
    pub array: *const u8,
    pub generic_parameter_index: i32,
    pub generic_class: *const (),
}

#[repr(C)]
pub struct Il2CppType {
    pub data: Il2CppTypeData,
    pub bits: u32,
}

unsafe impl Send for Il2CppType {}
unsafe impl Sync for Il2CppType {}

impl Il2CppType {
    #[inline]
    pub fn type_enum(&self) -> u8 {
        ((self.bits >> 16) & 0xff) as u8
    }

    #[inline]
    pub fn byref(&self) -> bool {
        ((self.bits >> 29) & 1) != 0
    }

    #[inline]
    pub fn valuetype(&self) -> bool {
        ((self.bits >> 31) & 1) != 0
    }

    #[inline]
    pub fn data_as_usize(&self) -> usize {
        unsafe { self.data.data as usize }
    }
}

pub const TYPE_VOID: u8 = 0x01;
pub const TYPE_BOOLEAN: u8 = 0x02;
pub const TYPE_CHAR: u8 = 0x03;
pub const TYPE_I1: u8 = 0x04;
pub const TYPE_U1: u8 = 0x05;
pub const TYPE_I2: u8 = 0x06;
pub const TYPE_U2: u8 = 0x07;
pub const TYPE_I4: u8 = 0x08;
pub const TYPE_U4: u8 = 0x09;
pub const TYPE_I8: u8 = 0x0a;
pub const TYPE_U8: u8 = 0x0b;
pub const TYPE_R4: u8 = 0x0c;
pub const TYPE_R8: u8 = 0x0d;
pub const TYPE_STRING: u8 = 0x0e;
pub const TYPE_PTR: u8 = 0x0f;
pub const TYPE_BYREF: u8 = 0x10;
pub const TYPE_VALUETYPE: u8 = 0x11;
pub const TYPE_CLASS: u8 = 0x12;
pub const TYPE_VAR: u8 = 0x13;
pub const TYPE_ARRAY: u8 = 0x14;
pub const TYPE_GENERICINST: u8 = 0x15;
pub const TYPE_TYPEDBYREF: u8 = 0x16;
pub const TYPE_I: u8 = 0x18;
pub const TYPE_U: u8 = 0x19;
pub const TYPE_FNPTR: u8 = 0x1b;
pub const TYPE_OBJECT: u8 = 0x1c;
pub const TYPE_SZARRAY: u8 = 0x1d;
pub const TYPE_MVAR: u8 = 0x1e;

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

