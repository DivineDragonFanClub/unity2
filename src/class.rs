use std::sync::OnceLock;

use crate::il2cpp::{api, FieldInfo, Il2CppClass, MethodInfo, PropertyInfo};
use crate::method::Method;
use crate::system::SystemType;
use crate::{Array, ClassIdentity};

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Class {
    inner: &'static Il2CppClass,
}

impl Class {
    #[inline]
    pub fn from_raw(inner: &'static Il2CppClass) -> Self {
        Self { inner }
    }

    // Dotted names like `MapUnitCommandMenu.DanceMenuItem` split on the last `.` and walk nested_types
    pub fn lookup(namespace: &str, name: &str) -> Self {
        Self::try_lookup(namespace, name)
            .unwrap_or_else(|e| panic!("{}", e))
    }

    pub fn try_lookup(namespace: &str, name: &str) -> crate::Il2CppResult<Self> {
        if let Some((outer, inner)) = name.rsplit_once('.') {
            let outer_class = Self::try_lookup(namespace, outer)?;
            return outer_class
                .raw()
                .get_nested_types()
                .iter()
                .find(|nt| nt.get_name() == inner)
                .map(|&c| Self::from_raw(c))
                .ok_or_else(|| crate::Il2CppError::MissingClass {
                    namespace: namespace.to_string(),
                    name: name.to_string(),
                });
        }
        Il2CppClass::from_name(namespace, name)
            .map(|c| Self { inner: &*c })
            .ok_or_else(|| crate::Il2CppError::MissingClass {
                namespace: namespace.to_string(),
                name: name.to_string(),
            })
    }

    #[inline]
    pub fn raw(self) -> &'static Il2CppClass {
        self.inner
    }

    pub fn name(self) -> String {
        self.inner.get_name()
    }

    pub fn namespace(self) -> String {
        self.inner.get_namespace()
    }

    pub fn parent(self) -> Option<Class> {
        let parent = self.inner._1.parent;

        if std::ptr::eq(parent as *const _, self.inner as *const _) {
            None
        } else {
            Some(Class { inner: parent })
        }
    }

    // Flat walk over the cached type_hierarchy slice
    pub fn hierarchy(self) -> impl Iterator<Item = Class> {
        self.inner
            .get_class_hierarchy()
            .iter()
            .map(|&c| Class::from_raw(c))
    }

    pub fn is<T: ClassIdentity>(self) -> bool {
        self == T::class()
    }

    // Immediate parent match only, does not walk the hierarchy
    pub fn parent_is<T: ClassIdentity>(self) -> bool {
        self.parent().is_some_and(|p| p.is::<T>())
    }

    pub fn interfaces(self) -> impl Iterator<Item = Class> {
        self.inner
            .get_implemented_interfaces()
            .iter()
            .map(|&c| Class::from_raw(c))
    }

    // Matches il2cpp_class_is_assignable_from, hierarchy walk + flattened interfaces
    pub fn is_subclass_of<T: ClassIdentity>(self) -> bool {
        let target = T::class();
        self.hierarchy().any(|c| c == target) || self.interfaces().any(|i| i == target)
    }

    // Declared-only, walk hierarchy().flat_map(|c| c.declared_fields()) for inherited too
    pub fn declared_fields(self) -> &'static [FieldInfo] {
        self.inner.get_fields()
    }

    pub fn declared_methods(self) -> &'static [&'static MethodInfo] {
        self.inner.get_methods()
    }

    pub fn declared_properties(self) -> &'static [PropertyInfo] {
        self.inner.get_properties()
    }
}

impl Class {
    #[inline]
    pub fn of<T: ClassIdentity>() -> Class {
        T::class()
    }

    // GC-managed class clone, instances pointing at the clone via klass keep it alive
    pub fn clone_for_override(self) -> Class {
        // Class header ends at 0x138, vtable entries follow, 16 bytes each
        const HEADER_SIZE: usize = 0x138;
        const VIRTUAL_INVOKE_SIZE: usize = ::core::mem::size_of::<crate::il2cpp::VirtualInvoke>();

        let src = self.raw();
        let size = HEADER_SIZE + VIRTUAL_INVOKE_SIZE * src._2.vtable_count as usize; // all my homies HATe pointer math

        unsafe {
            // kind = 0 is Normal scanned allocation
            let dest = crate::il2cpp::api::gc_malloc_kind(size, 0);

            ::core::ptr::copy_nonoverlapping(
                src as *const Il2CppClass as *const u8,
                dest,
                size,
            );

            Class::from_raw(&*(dest as *const Il2CppClass))
        }
    }

    /// Only safe on a clone_for_override result, writing in the shared class's vtable affects every instance
    pub fn override_virtual_method(
        self,
        name: &str,
        method_info: &'static crate::il2cpp::MethodInfo,
    ) -> Option<crate::il2cpp::VirtualInvoke> {
        let class = self.raw();

        let slot = class
            .get_vtable()
            .iter()
            .position(|v| v.get_name().as_deref() == Some(name))?;

        unsafe {
            let vtable_ptr = class.get_vtable().as_ptr() as *mut crate::il2cpp::VirtualInvoke;
            let entry = &mut *vtable_ptr.add(slot);
            let old = *entry;

            entry.method_ptr = method_info.method_ptr;
            entry.method_info = method_info;

            Some(old)
        }
    }

    // Resolves a generic class definition (e.g. List`1) against concrete type args with reflection
    pub fn make_generic(self, type_args: &[Class]) -> Option<Class> {
        let args_array: Array<SystemType> =
            Array::new(SystemType::class().raw(), type_args.len())?;
            
        for (i, arg) in type_args.iter().enumerate() {
            let ty = SystemType::from_il2cpp_type(arg.raw().get_type())?;
            args_array.set(i, ty);
        }

        let generic_type = SystemType::from_il2cpp_type(self.raw().get_type())?;

        static MAKE_GENERIC_TYPE: OnceLock<
            Method<fn(SystemType, Array<SystemType>) -> SystemType>,
        > = OnceLock::new();

        let method = MAKE_GENERIC_TYPE.get_or_init(|| {
            Class::lookup("System", "RuntimeType")
                .method::<fn(SystemType, Array<SystemType>) -> SystemType>(
                    "MakeGenericType",
                )
                .expect("System.RuntimeType.MakeGenericType not found in IL2CPP metadata")
        });

        let result = method.call(generic_type, args_array);

        if result.is_null() {
            return None;
        }

        let result_type = result.il2cpp_type();
        let result_class = unsafe { api::class_from_il2cpptype(result_type) }?;

        unsafe { api::class_init(result_class) };
        
        Some(Class::from_raw(result_class))
    }
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.inner as *const _, other.inner as *const _)
    }
}
impl Eq for Class {}
