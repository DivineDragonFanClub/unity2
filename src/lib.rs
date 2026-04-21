// Lets the proc macros' `::unity2::…` paths resolve when they're invoked inside this crate
extern crate self as unity2;

use crate::il2cpp::Il2CppClass;

pub use unity_macro::*;

#[macro_export]
macro_rules! method_info {
    ($callback:expr, $parameters_count:expr $(,)?) => {{
        static __UNITY2_MI: ::std::sync::OnceLock<&'static $crate::MethodInfo> =
            ::std::sync::OnceLock::new();
        *__UNITY2_MI.get_or_init(|| {
            let __mi = ::std::boxed::Box::leak(::std::boxed::Box::new(
                $crate::MethodInfo::new(),
            ));
            __mi.method_ptr = $callback as *mut u8;
            __mi.parameters_count = $parameters_count;
            &*__mi
        })
    }};
}

mod backend_assertion;

pub mod class;
pub mod diag;
pub mod engine;
pub mod error;
pub mod il2cpp;
pub mod lookup;
pub mod method;
pub mod prelude;
pub mod system;

pub use class::Class;
pub use error::{Il2CppError, Il2CppResult};
pub use il2cpp::{FieldInfo, MethodInfo, PropertyInfo};
pub use method::{Method, MethodSignature};
pub use system::{Il2CppString, SystemType};

pub type OptionalMethod = ::core::option::Option<&'static ()>;

// Lets primitives participate in reflection without a manual class lookup
macro_rules! impl_primitive_class_identity {
    ($($rust:ty => $il2cpp:literal),* $(,)?) => {
        $(
            impl ClassIdentity for $rust {
                const NAMESPACE: &'static str = "System";
                const NAME: &'static str = $il2cpp;

                fn class() -> Class {
                    static CACHE: ::std::sync::OnceLock<Class> = ::std::sync::OnceLock::new();
                    *CACHE.get_or_init(|| Class::lookup(Self::NAMESPACE, Self::NAME))
                }
            }
        )*
    };
}

impl_primitive_class_identity! {
    bool => "Boolean",
    i8   => "SByte",
    u8   => "Byte",
    i16  => "Int16",
    u16  => "UInt16",
    i32  => "Int32",
    u32  => "UInt32",
    i64  => "Int64",
    u64  => "UInt64",
    f32  => "Single",
    f64  => "Double",
    char => "Char",
}

pub trait ClassIdentity: Copy {
    const NAMESPACE: &'static str;
    const NAME: &'static str;

    fn class() -> Class;
}

pub trait FromIlInstance: Sized + ClassIdentity {
    fn from_il_instance(instance: IlInstance) -> Self;
    
    fn instantiate() -> Option<Self> {
        let inst = unsafe { crate::il2cpp::api::object_new(Self::class().raw()) };
        if inst.is_null() {
            None
        } else {
            Some(Self::from_il_instance(inst))
        }
    }
}

pub trait Cast: SystemObject {
    #[inline]
    fn class(self) -> Class {
        Class::from_raw(object_get_class(self))
    }

    #[inline]
    fn is_null(self) -> bool {
        self.as_instance().is_null()
    }

    // Points this instance's klass field at `new_class`
    fn rebind_class(self, new_class: Class) {
        assert!(!self.as_instance().is_null(), "Cast::rebind_class on null instance");
        unsafe {
            let klass_slot =
                self.as_instance().as_ptr() as *mut *const crate::il2cpp::Il2CppClass;
            *klass_slot = new_class.raw() as *const _;
        }
    }

    // Clone this instance's class and rebind to it so overrides are per-instance
    fn override_class(self) -> Class {
        let cloned = self.class().clone_for_override();
        self.rebind_class(cloned);
        cloned
    }

    #[inline]
    fn is_instance_of<T: ClassIdentity>(self) -> bool {
        self.class().is_subclass_of::<T>()
    }

    // True when this instance's immediate parent class is exactly T, does not match T itself
    #[inline]
    fn is_direct_subclass_of<T: ClassIdentity>(self) -> bool {
        self.class().parent_is::<T>()
    }

    #[inline]
    fn try_cast<T: ClassIdentity + FromIlInstance>(self) -> Option<T> {
        if self.is_instance_of::<T>() {
            Some(T::from_il_instance(self.as_instance()))
        } else {
            None
        }
    }

    // Caller swears the runtime class is T or a subclass, fields and methods read garbage otherwise
    #[inline]
    unsafe fn cast<T: ClassIdentity + FromIlInstance>(self) -> T {
        T::from_il_instance(self.as_instance())
    }
}

impl<T: SystemObject> Cast for T {}

#[repr(C)]
pub(crate) struct IlObject {
    pub class: *mut Il2CppClass,
    _monitor: *const (),
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct IlInstance(*mut IlObject);

impl IlInstance {
    #[inline]
    pub fn from_raw(ptr: *mut ()) -> Self {
        Self(ptr as *mut IlObject)
    }

    #[inline]
    pub fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    #[inline]
    pub fn as_ptr(self) -> *mut () {
        self.0 as *mut ()
    }

    #[inline]
    pub(crate) fn as_object_ptr(self) -> *mut IlObject {
        self.0
    }

    #[inline]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }

    // Pointer past the 16-byte header, into the object's fields
    #[inline]
    pub(crate) fn field_ptr(self, offset: usize) -> *mut u8 {
        unsafe { (self.0 as *mut u8).add(offset) }
    }
}

// Il2CppArraySize handle, nullable, only length and element accessors panic on null
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Array<T: Copy>(IlInstance, ::core::marker::PhantomData<T>);

unsafe impl<T: Copy> Send for Array<T> {}
unsafe impl<T: Copy> Sync for Array<T> {}

impl<T: Copy> From<Array<T>> for IlInstance {
    fn from(a: Array<T>) -> Self {
        a.0
    }
}

impl<T: Copy> AsRef<IlInstance> for Array<T> {
    fn as_ref(&self) -> &IlInstance {
        &self.0
    }
}

#[repr(C)]
pub(crate) struct InnerArray<T: Copy> {
    class: *const Il2CppClass,
    monitor: *const (),
    bounds: *const Il2CppArrayBounds,
    pub max_length: usize,
    pub m_items: [T; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Il2CppArrayBounds {
    length: usize,
    lower_bound: i32,
}

impl<T: Copy> Array<T> {
    #[inline]
    pub fn from_raw(ptr: *mut ()) -> Self {
        Self(IlInstance::from_raw(ptr), ::core::marker::PhantomData)
    }

    pub fn new(class: &Il2CppClass, length: usize) -> Option<Array<T>> {
        let arr = unsafe { crate::il2cpp::api::array_new::<T>(class, length) };
        if arr.is_null() {
            None
        } else {
            Some(arr)
        }
    }

    #[inline]
    fn inner(self) -> *const InnerArray<T> {
        self.0.as_ptr() as *const InnerArray<T>
    }

    #[inline]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }

    #[inline]
    pub fn items(self) -> *const T {
        unsafe { (*self.inner()).m_items.as_ptr() }
    }

    #[inline]
    pub fn max_length(self) -> usize {
        assert!(!self.is_null(), "Array::max_length on null array");
        unsafe { (*self.inner()).max_length }
    }

    #[inline]
    pub fn len(self) -> usize {
        self.max_length()
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn get(self, index: usize) -> T {
        let len = self.max_length();
        if index >= len {
            panic!("Array index {} out of bounds (len = {})", index, len);
        }
        unsafe { *self.items().add(index) }
    }
    
    #[inline]
    pub fn set(self, index: usize, value: T) {
        let len = self.max_length();
        if index >= len {
            panic!("Array index {} out of bounds (len = {})", index, len);
        }
        unsafe {
            let items = self.items() as *mut T;
            *items.add(index) = value;
        }
    }

    #[inline]
    pub fn iter(self) -> ArrayIter<T> {
        self.into_iter()
    }

    #[inline]
    pub fn as_slice<'a>(&'a self) -> &'a [T] {
        assert!(!self.is_null(), "Array::as_slice on null array");
        unsafe { ::core::slice::from_raw_parts(self.items(), self.max_length()) }
    }

    #[inline]
    pub fn as_mut_slice<'a>(&'a mut self) -> &'a mut [T] {
        assert!(!self.is_null(), "Array::as_mut_slice on null array");
        unsafe { ::core::slice::from_raw_parts_mut(self.items() as *mut T, self.max_length()) }
    }

    pub fn copy_from_slice(self, src: &[T]) {
        assert!(!self.is_null(), "Array::copy_from_slice on null array");
        let dst_len = self.max_length();
        assert!(
            src.len() <= dst_len,
            "Array::copy_from_slice src len {} exceeds array len {}",
            src.len(),
            dst_len
        );
        unsafe {
            ::core::ptr::copy_nonoverlapping(src.as_ptr(), self.items() as *mut T, src.len());
        }
    }
}

impl<T: Copy + ClassIdentity> Array<T> {
    pub fn of_len(length: usize) -> Option<Self> {
        Self::new(T::class().raw(), length)
    }
}

impl<T: Copy> IntoIterator for Array<T> {
    type Item = T;
    type IntoIter = ArrayIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let len = self.max_length();
        ArrayIter { arr: self, index: 0, len }
    }
}

pub struct ArrayIter<T: Copy> {
    arr: Array<T>,
    index: usize,
    len: usize,
}

impl<T: Copy> Iterator for ArrayIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.index < self.len {
            let v = unsafe { *self.arr.items().add(self.index) };
            self.index += 1;
            Some(v)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<T: Copy> ExactSizeIterator for ArrayIter<T> {}

pub trait SystemObject: Copy {
    fn as_instance(self) -> IlInstance;
}

impl<T> SystemObject for T
where
    T: Copy + Into<IlInstance>,
{
    fn as_instance(self) -> IlInstance {
        self.into()
    }
}

fn parent_chain(class: &Il2CppClass) -> impl Iterator<Item = &Il2CppClass> {
    const MAX_DEPTH: usize = 16;

    let mut cur: Option<&Il2CppClass> = Some(class);
    let mut steps = 0usize;
    std::iter::from_fn(move || {
        let this = cur?;
        steps += 1;
        if steps >= MAX_DEPTH {
            cur = None;
            return Some(this);
        }
        let parent_ptr: *const Il2CppClass = this._1.parent;
        cur = if parent_ptr.is_null() || std::ptr::eq(parent_ptr, this as *const _) {
            None
        } else {
            Some(unsafe { &*parent_ptr })
        };
        Some(this)
    })
}

pub fn get_properties(class: &Il2CppClass) -> Vec<&PropertyInfo> {
    let mut out = Vec::new();
    for c in parent_chain(class) {
        for p in c.get_properties() {
            out.push(p);
        }
    }
    out
}

pub fn get_fields(class: &Il2CppClass) -> Vec<&FieldInfo> {
    let mut out = Vec::new();
    for c in parent_chain(class) {
        for f in c.get_fields() {
            out.push(f);
        }
    }
    out
}

pub fn object_get_class<'a>(obj: impl SystemObject) -> &'a Il2CppClass {
    let instance = obj.as_instance();
    assert!(!instance.is_null(), "object_get_class on null reference");
    unsafe { &*(*instance.as_object_ptr()).class }
}

pub fn il2cpp_enum_names(enum_class: Class) -> Option<Vec<String>> {
    use std::sync::OnceLock;

    static GET_NAMES: OnceLock<Method<fn(system::SystemType) -> Array<Il2CppString>>> = OnceLock::new();

    let get_names = *GET_NAMES.get_or_init(|| {
        Class::lookup("System", "Enum")
            .method::<fn(system::SystemType) -> Array<Il2CppString>>("GetNames")
            .expect("System.Enum.GetNames not found in IL2CPP metadata")
    });

    let sys_type = system::SystemType::from_il2cpp_type(enum_class.raw().get_type())?;
    let names_array = get_names.call(sys_type);

    if names_array.is_null() {
        return None;
    }

    let len = names_array.len();

    let mut out = Vec::with_capacity(len);

    for i in 0..len {
        out.push(names_array.get(i).to_rust_string());
    }
    Some(out)
}

pub fn class_get_field_from_name<'a>(class: &'a Il2CppClass, name: &str) -> &'a FieldInfo {
    for c in parent_chain(class) {
        for f in c.get_fields() {
            if f.get_name().as_deref() == Some(name) {
                return f;
            }
        }
    }
    panic!(
        "{}",
        Il2CppError::MissingField {
            class: class.get_name(),
            field: name.to_string(),
        }
    )
}

pub fn field_get_value<Ty: Copy>(obj: impl SystemObject, field: &FieldInfo) -> Ty {
    field_get_value_at_offset(obj, field.offset as usize)
}

pub fn field_set_value<Ty: Copy>(obj: impl SystemObject, field: &FieldInfo, value: Ty) {
    field_set_value_at_offset(obj, field.offset as usize, value);
}

// Inherited fields keep their byte offset across the hierarchy, so a single cached offset works for every subclass
pub fn field_get_value_at_offset<Ty: Copy>(obj: impl SystemObject, offset: usize) -> Ty {
    let instance = obj.as_instance();
    unsafe { *(instance.field_ptr(offset) as *const Ty) }
}

pub fn field_set_value_at_offset<Ty: Copy>(obj: impl SystemObject, offset: usize, value: Ty) {
    let instance = obj.as_instance();
    unsafe {
        *(instance.field_ptr(offset) as *mut Ty) = value;
    }
}

// Static fields share the offset convention with instance fields but sit in the class's static-fields block
pub fn static_field_get_value_at_offset<Ty: Copy>(class: Class, offset: usize) -> Ty {
    let base = class.raw().static_fields as *const u8;
    unsafe { *(base.add(offset) as *const Ty) }
}

pub fn static_field_set_value_at_offset<Ty: Copy>(class: Class, offset: usize, value: Ty) {
    let base = class.raw().static_fields as *mut u8;
    unsafe {
        *(base.add(offset) as *mut Ty) = value;
    }
}
