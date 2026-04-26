use std::sync::OnceLock;

use crate::class::Class;
use crate::method::Method;
use crate::system::Il2CppString;
use crate::{Array, ClassIdentity};

use super::object::GameObject;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Scene {
    pub handle: i32,
}

impl Scene {
    pub fn is_valid(self) -> bool {
        static METHOD: OnceLock<Method<fn(i32) -> bool>> = OnceLock::new();
        let method = *METHOD.get_or_init(|| {
            Self::class()
                .method::<fn(i32) -> bool>("IsValidInternal")
                .expect("Scene.IsValidInternal missing")
        });
        method.call(self.handle)
    }

    pub fn is_loaded(self) -> bool {
        static METHOD: OnceLock<Method<fn(i32) -> bool>> = OnceLock::new();
        let method = *METHOD.get_or_init(|| {
            Self::class()
                .method::<fn(i32) -> bool>("GetIsLoadedInternal")
                .expect("Scene.GetIsLoadedInternal missing")
        });
        method.call(self.handle)
    }

    pub fn root_count(self) -> i32 {
        static METHOD: OnceLock<Method<fn(i32) -> i32>> = OnceLock::new();
        let method = *METHOD.get_or_init(|| {
            Self::class()
                .method::<fn(i32) -> i32>("GetRootCountInternal")
                .expect("Scene.GetRootCountInternal missing")
        });
        method.call(self.handle)
    }

    pub fn get_name(handle: i32) -> Option<Il2CppString> {
        static METHOD: OnceLock<Method<fn(i32) -> Il2CppString>> = OnceLock::new();
        let method = *METHOD.get_or_init(|| {
            Self::class()
                .method::<fn(i32) -> Il2CppString>("GetNameInternal")
                .expect("Scene.GetNameInternal missing")
        });
        let result = method.call(handle);
        if result.is_null() { None } else { Some(result) }
    }
}

impl ClassIdentity for Scene {
    const NAMESPACE: &'static str = "UnityEngine.SceneManagement";
    const NAME: &'static str = "Scene";

    fn class() -> Class {
        static CACHE: OnceLock<Class> = OnceLock::new();
        *CACHE.get_or_init(|| Class::lookup(Self::NAMESPACE, Self::NAME))
    }
}

#[unity2::class(namespace = "UnityEngine.SceneManagement")]
pub struct SceneManager {}

#[unity2::methods]
impl SceneManager {
    #[method(name = "GetActiveScene", args = 0)]
    pub fn get_active_scene() -> Scene;

    #[method(name = "get_sceneCount", args = 0)]
    pub fn scene_count() -> i32;

    #[method(name = "GetSceneAt", args = 1)]
    pub fn get_scene_at(index: i32) -> Scene;
}
