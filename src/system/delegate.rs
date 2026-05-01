use crate::il2cpp::MethodInfo;
use crate::{Array, ClassIdentity, FromIlInstance, Il2CppString, IlInstance, IntPtr, SystemType};

#[unity2::class(namespace = "System")]
pub struct Action {}

#[unity2::methods]
impl Action {
    #[method(name = "Invoke", args = 0)]
    fn invoke(self);

    #[method(name = ".ctor", args = 2)]
    fn ctor(self, target: IlInstance, method_info: *const MethodInfo);
}

impl Action {
    pub fn from_raw_parts(
        target: IlInstance,
        method_info: &'static MethodInfo,
    ) -> Option<Self> {
        let instance = <Self as FromIlInstance>::instantiate()?;
        instance.ctor(target, method_info as *const _);
        Some(instance)
    }
}

#[unity2::class(namespace = "System", name = "Action`1")]
pub struct Action1<A: ClassIdentity> {}

#[unity2::methods]
impl<A: ClassIdentity> Action1<A> {
    #[method(name = "Invoke")]
    fn invoke(self, arg: A);

    #[method(name = ".ctor", args = 2)]
    fn ctor(self, target: IlInstance, method_info: *const MethodInfo);
}

impl<A: ClassIdentity> Action1<A> {
    pub fn from_raw_parts(
        target: IlInstance,
        method_info: &'static MethodInfo,
    ) -> Option<Self> {
        let instance = <Self as FromIlInstance>::instantiate()?;
        instance.ctor(target, method_info as *const _);
        Some(instance)
    }
}

#[unity2::class(namespace = "System", name = "Func`1")]
pub struct Func1<R: ClassIdentity> {}

#[unity2::methods]
impl<R: ClassIdentity> Func1<R> {
    #[method(name = "Invoke", args = 0)]
    fn invoke(self) -> R;

    #[method(name = ".ctor", args = 2)]
    fn ctor(self, target: IlInstance, method_info: *const MethodInfo);
}

impl<R: ClassIdentity> Func1<R> {
    pub fn from_raw_parts(
        target: IlInstance,
        method_info: &'static MethodInfo,
    ) -> Option<Self> {
        let instance = <Self as FromIlInstance>::instantiate()?;
        instance.ctor(target, method_info as *const _);
        Some(instance)
    }
}

#[unity2::class(namespace = "System", name = "Func`2")]
pub struct Func2<A: ClassIdentity, R: ClassIdentity> {}

#[unity2::methods]
impl<A: ClassIdentity, R: ClassIdentity> Func2<A, R> {
    #[method(name = "Invoke")]
    fn invoke(self, arg: A) -> R;

    #[method(name = ".ctor", args = 2)]
    fn ctor(self, target: IlInstance, method_info: *const MethodInfo);
}

impl<A: ClassIdentity, R: ClassIdentity> Func2<A, R> {
    pub fn from_raw_parts(
        target: IlInstance,
        method_info: &'static MethodInfo,
    ) -> Option<Self> {
        let instance = <Self as FromIlInstance>::instantiate()?;
        instance.ctor(target, method_info as *const _);
        Some(instance)
    }
}

#[unity2::class(namespace = "System", name = "Delegate")]
pub struct Delegate {
    method_ptr: IntPtr,
    invoke_impl: IntPtr,
    method: IntPtr,
    delegate_trampoline: IntPtr,
    extra_arg: IntPtr,
    method_code: IntPtr,
    method_info: MethodInfo,
    original_method_info: MethodInfo,
    data: DelegateData,
    method_is_virtual: bool,
}

#[unity2::methods]
impl Delegate {
    #[method(name = "get_Method")]
    fn get_method(self) -> MethodInfo;
}

#[unity2::class(namespace = "System", name = "DelegateData")]
pub struct DelegateData {
    pub target_type: SystemType,
    pub method_name: Il2CppString,
    pub curried_first_arg: bool,
}

#[unity2::methods]
impl DelegateData {
    #[method(name = ".ctor")]
    fn ctor(self);
}

#[unity2::class(namespace = "System", name = "MulticastDelegate")]
#[parent(Delegate)]
pub struct MulticastDelegate {
    pub delegates: Array<Delegate>,
}