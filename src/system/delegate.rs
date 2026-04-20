use crate::il2cpp::MethodInfo;
use crate::{ClassIdentity, FromIlInstance, IlInstance};

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
