pub mod api;
pub mod assembly;
pub mod class;
pub mod method;

pub use class::{FieldInfo, Il2CppClass, Il2CppReflectionType, PropertyInfo, VirtualInvoke};
pub use method::{MethodInfo, OptionalMethod, ParameterInfo};

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

#[unsafe(no_mangle)]
pub extern "C" fn il2cpp_init(domain_name: *const i8) -> i32 {
    unsafe { api::init(domain_name) }
}
