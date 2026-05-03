use std::ffi::CStr;
use std::sync::OnceLock;

use crate::il2cpp::method::ParameterInfo;
use crate::il2cpp::{FieldInfo, Il2CppClass, Il2CppType, MethodInfo};
use crate::{Class, ClassIdentity, FromIlInstance};

pub trait InjectedClass: ClassIdentity + FromIlInstance {
    type Parent: ClassIdentity;
    const EXTRA_BYTES: u32 = 0;

    fn class_builder() -> ClassBuilder<Self::Parent>;

    fn cache() -> &'static OnceLock<Class>;

    fn fill_cache(class: Class) {
        let _ = Self::cache().set(class);
    }
}

pub struct InjectedFieldDescriptor {
    pub name: &'static CStr,
    pub ty: &'static Il2CppType,
    pub offset: u32,
}

pub struct InjectedParameterDescriptor {
    pub name: &'static CStr,
    pub ty: &'static Il2CppType,
}

pub struct InjectedMethodDescriptor {
    pub name: &'static CStr,
    pub method_ptr: *mut u8,
    pub return_type: &'static Il2CppType,
    pub parameters: Vec<InjectedParameterDescriptor>,
}

unsafe impl Send for InjectedMethodDescriptor {}
unsafe impl Sync for InjectedMethodDescriptor {}

pub struct ClassBuilder<P: ClassIdentity> {
    namespace: &'static CStr,
    name: &'static CStr,
    extra_bytes: u32,
    overrides: Vec<(String, *mut u8)>,
    added_methods: Vec<InjectedMethodDescriptor>,
    added_fields: Vec<InjectedFieldDescriptor>,
    _parent: ::core::marker::PhantomData<P>,
}

impl<P: ClassIdentity> ClassBuilder<P> {
    pub fn new(namespace: &'static CStr, name: &'static CStr) -> Self {
        Self {
            namespace,
            name,
            extra_bytes: 0,
            overrides: Vec::new(),
            added_methods: Vec::new(),
            added_fields: Vec::new(),
            _parent: ::core::marker::PhantomData,
        }
    }

    pub fn extra_bytes(mut self, bytes: u32) -> Self {
        self.extra_bytes = bytes;
        self
    }

    pub fn override_virtual(mut self, name: impl Into<String>, method_ptr: *mut u8) -> Self {
        self.overrides.push((name.into(), method_ptr));
        self
    }

    pub fn add_methods(mut self, methods: Vec<InjectedMethodDescriptor>) -> Self {
        self.added_methods.extend(methods);
        self
    }

    pub fn add_fields(mut self, fields: Vec<InjectedFieldDescriptor>) -> Self {
        self.added_fields.extend(fields);
        self
    }

    pub fn build(self) -> Class {
        let parent = P::class();
        let cloned = parent.clone_for_override();

        cloned.set_name(self.name, self.namespace);
        cloned.set_instance_size(parent.instance_size() + self.extra_bytes);

        install_type_hierarchy(cloned, parent);

        for (slot_name, fn_ptr) in self.overrides {
            install_override(cloned, &slot_name, fn_ptr);
        }

        if !self.added_fields.is_empty() {
            install_fields(cloned, self.added_fields);
        }

        if !self.added_methods.is_empty() {
            install_methods(cloned, self.added_methods);
        }

        cloned
    }
}

fn install_type_hierarchy(class: Class, parent: Class) {
    let parent_raw = parent.raw();
    let parent_depth = parent_raw._2.type_hierarchy_depth as usize;
    let parent_hierarchy: &[&'static Il2CppClass] = unsafe {
        if parent_raw._2.type_hierarchy.is_null() || parent_depth == 0 {
            &[]
        } else {
            ::core::slice::from_raw_parts(parent_raw._2.type_hierarchy, parent_depth)
        }
    };

    let mut new_hierarchy: Vec<&'static Il2CppClass> = parent_hierarchy.to_vec();
    new_hierarchy.push(class.raw());

    let leaked: &'static [&'static Il2CppClass] = Box::leak(new_hierarchy.into_boxed_slice());
    unsafe {
        let raw_ptr = class.raw() as *const Il2CppClass as *mut Il2CppClass;
        ::core::ptr::write_volatile(
            ::core::ptr::addr_of_mut!((*raw_ptr)._2.type_hierarchy),
            leaked.as_ptr(),
        );
        ::core::ptr::write_volatile(
            ::core::ptr::addr_of_mut!((*raw_ptr)._2.type_hierarchy_depth),
            leaked.len() as u8,
        );
    }
}

fn install_fields(class: Class, descriptors: Vec<InjectedFieldDescriptor>) {
    const FIELD_ATTRIBUTE_PUBLIC: u32 = 0x0006;

    let parent_fields: &[FieldInfo] = class.raw().get_fields();
    let mut all_fields: Vec<FieldInfo> = parent_fields.to_vec();

    for desc in descriptors {
        let mut ty: Il2CppType =
            unsafe { ::core::ptr::read(desc.ty as *const Il2CppType) };
        ty.bits = (ty.bits & !0xFFFF) | FIELD_ATTRIBUTE_PUBLIC;
        let leaked_ty: &'static Il2CppType = Box::leak(Box::new(ty));

        let fi = FieldInfo::new_synthetic(
            desc.name.as_ptr() as *const u8,
            leaked_ty,
            class.raw(),
            desc.offset as i32,
            0,
        );
        all_fields.push(fi);
    }

    let leaked_array: &'static [FieldInfo] = Box::leak(all_fields.into_boxed_slice());
    unsafe {
        let raw_ptr = class.raw() as *const Il2CppClass as *mut Il2CppClass;
        ::core::ptr::write_volatile(
            ::core::ptr::addr_of_mut!((*raw_ptr)._1.fields),
            leaked_array.as_ptr(),
        );
        ::core::ptr::write_volatile(
            ::core::ptr::addr_of_mut!((*raw_ptr)._2.field_count),
            leaked_array.len() as u16,
        );
    }
}

fn install_override(class: Class, slot_name: &str, method_ptr: *mut u8) {
    let donor: &MethodInfo = {
        let vi = class.raw().get_virtual_method(slot_name).unwrap_or_else(|| {
            panic!(
                "ClassBuilder: virtual method `{}` not found on {}.{}",
                slot_name,
                class.namespace(),
                class.name()
            )
        });
        vi.method_info
    };

    let mut new_mi: MethodInfo = *donor;
    new_mi.method_ptr = method_ptr;
    let leaked: &'static MethodInfo = Box::leak(Box::new(new_mi));

    class.override_virtual_method(slot_name, leaked);
}

fn install_methods(class: Class, descriptors: Vec<InjectedMethodDescriptor>) {
    let parent_methods: &[&'static MethodInfo] = class.raw().get_methods();
    let mut all_methods: Vec<&'static MethodInfo> = parent_methods.to_vec();

    fn pick_invoker_donor(
        parent_methods: &[&'static MethodInfo],
        params_count: u8,
        return_is_void: bool,
    ) -> *const u8 {
        if let Some(m) = parent_methods.iter().find(|m| {
            m.parameters_count == params_count
                && method_is_void_return(m) == return_is_void
                && !m.invoker_method.is_null()
        }) {
            return m.invoker_method;
        }
        parent_methods
            .iter()
            .find(|m| !m.invoker_method.is_null())
            .map(|m| m.invoker_method)
            .unwrap_or(::core::ptr::null())
    }

    fn method_is_void_return(m: &MethodInfo) -> bool {
        if m.return_type.is_null() {
            return false;
        }
        unsafe {
            let ty = &*(m.return_type as *const Il2CppType);
            ty.type_enum() == crate::il2cpp::TYPE_VOID
        }
    }

    for desc in descriptors {
        let parameters_count = desc.parameters.len() as u8;

        let parameters_ptr: *const ParameterInfo = if desc.parameters.is_empty() {
            ::core::ptr::null()
        } else {
            let infos: Vec<ParameterInfo> = desc
                .parameters
                .iter()
                .enumerate()
                .map(|(i, p)| ParameterInfo {
                    name: p.name.as_ptr() as *const u8,
                    position: i as i32,
                    token: 0,
                    parameter_type: p.ty,
                })
                .collect();
            let leaked: &'static [ParameterInfo] = Box::leak(infos.into_boxed_slice());
            leaked.as_ptr()
        };

        let return_is_void = desc.return_type.type_enum() == crate::il2cpp::TYPE_VOID;
        let invoker_donor =
            pick_invoker_donor(parent_methods, parameters_count, return_is_void);

        let mut mi = MethodInfo::new();
        mi.method_ptr = desc.method_ptr;
        mi.invoker_method = invoker_donor;
        mi.name = desc.name.as_ptr() as *const u8;
        mi.class = Some(class.raw());
        mi.return_type = desc.return_type as *const Il2CppType as *const u8;
        mi.parameters = parameters_ptr;
        mi.parameters_count = parameters_count;
        mi.flags = 0x0006;
        mi.slot = u16::MAX;

        let leaked: &'static MethodInfo = Box::leak(Box::new(mi));
        all_methods.push(leaked);
    }

    let leaked_array: &'static [&'static MethodInfo] = Box::leak(all_methods.into_boxed_slice());
    unsafe {
        let raw_ptr = class.raw() as *const Il2CppClass as *mut Il2CppClass;
        ::core::ptr::write_volatile(
            ::core::ptr::addr_of_mut!((*raw_ptr)._1.methods),
            leaked_array.as_ptr(),
        );
        ::core::ptr::write_volatile(
            ::core::ptr::addr_of_mut!((*raw_ptr)._2.method_count),
            leaked_array.len() as u16,
        );
    }
}
