// UnityEngine.Object and descendants, distinct from System.Object

#![allow(unused_imports)]

use crate::class::Class;
use crate::system::Il2CppString;
use crate::{ClassIdentity, IlInstance};

use super::color::Color;
use super::vector::{Quaternion, Vector3};

#[unity2::class(namespace = "UnityEngine")]
pub struct Object {}

#[unity2::methods]
impl Object {
    #[method(name = "get_name")]
    fn name(self) -> Il2CppString;

    #[method(name = "set_name")]
    fn set_name(self, value: Il2CppString);

    #[method(name = "GetInstanceID")]
    fn get_instance_id(self) -> i32;

    #[method(name = "Destroy", args = 1)]
    fn destroy(target: Object);

    #[method(name = "DestroyImmediate", args = 1)]
    fn destroy_immediate(target: Object);

    #[method(name = "DontDestroyOnLoad")]
    fn dont_destroy_on_load(target: Object);
}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Object)]
pub struct GameObject {}

#[unity2::methods]
impl GameObject {
    #[method(name = "get_transform")]
    fn transform(self) -> Transform;

    #[method(name = "get_activeSelf")]
    fn active_self(self) -> bool;

    #[method(name = "get_activeInHierarchy")]
    fn active_in_hierarchy(self) -> bool;

    #[method(name = "SetActive")]
    fn set_active(self, value: bool);

    #[method(name = "get_layer")]
    fn layer(self) -> i32;

    #[method(name = "set_layer")]
    fn set_layer(self, value: i32);

    #[method(name = "get_tag")]
    fn tag(self) -> Il2CppString;

    #[method(name = "set_tag")]
    fn set_tag(self, value: Il2CppString);

    #[method(name = "Find")]
    fn find(name: Il2CppString) -> GameObject;
}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Object)]
pub struct Component {}

#[unity2::methods]
impl Component {
    #[method(name = "get_gameObject")]
    fn game_object(self) -> GameObject;

    #[method(name = "get_transform")]
    fn transform(self) -> Transform;
}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Component, Object)]
pub struct Transform {}

#[unity2::methods]
impl Transform {
    #[method(name = "get_position")]
    fn position(self) -> Vector3;

    #[method(name = "set_position")]
    fn set_position(self, value: Vector3);

    #[method(name = "get_localPosition")]
    fn local_position(self) -> Vector3;

    #[method(name = "set_localPosition")]
    fn set_local_position(self, value: Vector3);

    #[method(name = "get_rotation")]
    fn rotation(self) -> Quaternion;

    #[method(name = "set_rotation")]
    fn set_rotation(self, value: Quaternion);

    #[method(name = "get_localRotation")]
    fn local_rotation(self) -> Quaternion;

    #[method(name = "set_localRotation")]
    fn set_local_rotation(self, value: Quaternion);

    #[method(name = "get_localScale")]
    fn local_scale(self) -> Vector3;

    #[method(name = "set_localScale")]
    fn set_local_scale(self, value: Vector3);

    #[method(name = "get_parent")]
    fn parent(self) -> Transform;

    #[method(name = "SetParent", args = 1)]
    fn set_parent(self, parent: Transform);

    #[method(name = "get_childCount")]
    fn child_count(self) -> i32;

    #[method(name = "GetChild")]
    fn get_child(self, index: i32) -> Transform;

    #[method(name = "Find")]
    fn find(self, name: Il2CppString) -> Transform;
}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Component, Object)]
pub struct Behaviour {}

#[unity2::methods]
impl Behaviour {
    #[method(name = "get_enabled")]
    fn enabled(self) -> bool;

    #[method(name = "set_enabled")]
    fn set_enabled(self, value: bool);

    #[method(name = "get_isActiveAndEnabled")]
    fn is_active_and_enabled(self) -> bool;
}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Behaviour, Component, Object)]
pub struct MonoBehaviour {}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Object)]
pub struct ScriptableObject {}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Object)]
pub struct Material {}

#[unity2::methods]
impl Material {
    #[method(name = "get_color")]
    fn color(self) -> Color;

    #[method(name = "set_color")]
    fn set_color(self, value: Color);

    #[method(name = "get_mainTexture")]
    fn main_texture(self) -> Texture;

    #[method(name = "set_mainTexture")]
    fn set_main_texture(self, value: Texture);

    #[method(name = "HasProperty")]
    fn has_property(self, name: Il2CppString) -> bool;
}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Object)]
pub struct Texture {}

#[unity2::methods]
impl Texture {
    #[method(name = "get_width")]
    fn width(self) -> i32;

    #[method(name = "get_height")]
    fn height(self) -> i32;
}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Texture, Object)]
pub struct Texture2D {}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Object)]
pub struct Sprite {}

#[unity2::methods]
impl Sprite {
    #[method(name = "get_texture")]
    fn texture(self) -> Texture2D;
}
