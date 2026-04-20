// Runtime helpers the macro calls from per-method LazyLock<usize> offsets

use crate::{Class, Il2CppError};

// Panics on missing class or method, or on ambiguous overload, fails loudly on first call during LazyLock init
pub fn method_offset_by_name(
    namespace: &str,
    class_name: &str,
    method_name: &str,
    param_count: usize,
) -> usize {
    let class = Class::lookup(namespace, class_name).raw();

    let matches = class
        .get_methods()
        .iter()
        .filter(|m| {
            m.parameters_count as usize == param_count
                && m.get_name().as_deref() == Some(method_name)
        })
        .count();

    if matches > 1 {
        panic!(
            "{}",
            Il2CppError::AmbiguousMethod {
                class: format!("{}.{}", namespace, class_name),
                method: method_name.to_string(),
                param_count,
                overload_count: matches,
            }
        );
    }

    let method = class
        .get_method_from_name(method_name, param_count)
        .unwrap_or_else(|| {
            panic!(
                "{}",
                Il2CppError::MissingMethod {
                    class: format!("{}.{}", namespace, class_name),
                    method: method_name.to_string(),
                    param_count,
                }
            )
        });

    method_ptr_to_offset(method.method_ptr)
}

pub fn method_offset_by_vtable_index(
    namespace: &str,
    class_name: &str,
    vtable_index: usize,
) -> usize {
    let class = Class::lookup(namespace, class_name).raw();

    let vtable = class.get_vtable();
    let slot = vtable.get(vtable_index).unwrap_or_else(|| {
        panic!(
            "{}",
            Il2CppError::VtableIndexOutOfRange {
                class: format!("{}.{}", namespace, class_name),
                index: vtable_index,
                vtable_len: vtable.len(),
            }
        )
    });

    method_ptr_to_offset(slot.method_ptr)
}

fn method_ptr_to_offset(method_ptr: *mut u8) -> usize {
    let text = lazysimd::scan::get_text();
    unsafe { (method_ptr as *const u8).offset_from(text.as_ptr()) as usize }
}
