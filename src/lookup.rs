use crate::il2cpp::MethodInfo;
use crate::{Class, Il2CppError, Il2CppResult};

pub fn method_offset_by_name(
    namespace: &str,
    class_name: &str,
    method_name: &str,
    param_count: usize,
) -> Il2CppResult<usize> {
    let class = Class::try_lookup(namespace, class_name)?;
    method_offset_on_class(class, method_name, param_count)
}

pub fn method_offset_on_class(
    class: Class,
    method_name: &str,
    param_count: usize,
) -> Il2CppResult<usize> {
    let raw = class.raw();

    let matches = raw
        .get_methods()
        .iter()
        .filter(|m| {
            m.parameters_count as usize == param_count
                && m.get_name().as_deref() == Some(method_name)
        })
        .count();

    if matches > 1 {
        return Err(Il2CppError::AmbiguousMethod {
            class: format!("{}.{}", raw.get_namespace(), raw.get_name()),
            method: method_name.to_string(),
            param_count,
            overload_count: matches,
        });
    }

    let method = raw.get_method_from_name(method_name, param_count).ok_or_else(|| {
        Il2CppError::MissingMethod {
            class: format!("{}.{}", raw.get_namespace(), raw.get_name()),
            method: method_name.to_string(),
            param_count,
        }
    })?;

    Ok(method_ptr_to_offset(method.method_ptr))
}

pub fn method_offset_by_vtable_index(
    namespace: &str,
    class_name: &str,
    vtable_index: usize,
) -> Il2CppResult<usize> {
    let class = Class::try_lookup(namespace, class_name)?;
    method_offset_by_vtable_index_on_class(class, vtable_index)
}

pub fn method_offset_by_vtable_index_on_class(
    class: Class,
    vtable_index: usize,
) -> Il2CppResult<usize> {
    let raw = class.raw();
    let vtable = raw.get_vtable();
    let slot = vtable.get(vtable_index).ok_or_else(|| Il2CppError::VtableIndexOutOfRange {
        class: format!("{}.{}", raw.get_namespace(), raw.get_name()),
        index: vtable_index,
        vtable_len: vtable.len(),
    })?;

    Ok(method_ptr_to_offset(slot.method_ptr))
}

fn method_ptr_to_offset(method_ptr: *mut u8) -> usize {
    let text = lazysimd::scan::get_text();
    unsafe { (method_ptr as *const u8).offset_from(text.as_ptr()) as usize }
}

pub fn method_info_on_class(
    class: Class,
    method_name: &str,
    param_count: usize,
) -> Il2CppResult<&'static MethodInfo> {
    let raw = class.raw();

    let matches = raw
        .get_methods()
        .iter()
        .filter(|m| {
            m.parameters_count as usize == param_count
                && m.get_name().as_deref() == Some(method_name)
        })
        .count();

    if matches > 1 {
        return Err(Il2CppError::AmbiguousMethod {
            class: format!("{}.{}", raw.get_namespace(), raw.get_name()),
            method: method_name.to_string(),
            param_count,
            overload_count: matches,
        });
    }

    raw.get_method_from_name(method_name, param_count)
        .map(|mi| &*mi)
        .ok_or_else(|| Il2CppError::MissingMethod {
            class: format!("{}.{}", raw.get_namespace(), raw.get_name()),
            method: method_name.to_string(),
            param_count,
        })
}

pub fn method_info_by_vtable_index_on_class(
    class: Class,
    vtable_index: usize,
) -> Il2CppResult<&'static MethodInfo> {
    let raw = class.raw();
    let vtable = raw.get_vtable();
    let slot = vtable.get(vtable_index).ok_or_else(|| Il2CppError::VtableIndexOutOfRange {
        class: format!("{}.{}", raw.get_namespace(), raw.get_name()),
        index: vtable_index,
        vtable_len: vtable.len(),
    })?;
    Ok(slot.method_info)
}
