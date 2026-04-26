// UnityEngine.Object and descendants, distinct from System.Object
#![allow(unused_imports)]

use crate::class::Class;
use crate::system::Il2CppString;
use crate::{ClassIdentity, IlInstance};

use super::color::Color;
use super::vector::{Quaternion, Rect, Vector2, Vector3};

#[unity2::class(namespace = "UnityEngine")]
pub struct Object {}

#[unity2::methods]
impl Object {
    #[method(name = "get_name")]
    pub fn name(self) -> Il2CppString;

    #[method(name = "set_name")]
    pub fn set_name(self, value: Il2CppString);

    #[method(name = "GetInstanceID")]
    pub fn get_instance_id(self) -> i32;

    #[method(name = "Destroy", args = 1)]
    pub fn destroy(target: Object);

    #[method(name = "DestroyImmediate", args = 1)]
    pub fn destroy_immediate(target: Object);

    #[method(name = "DontDestroyOnLoad")]
    pub fn dont_destroy_on_load(target: Object);
}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Object)]
pub struct GameObject {}

#[unity2::methods]
impl GameObject {
    #[method(name = "get_transform")]
    pub fn transform(self) -> Transform;

    #[method(name = "get_activeSelf")]
    pub fn active_self(self) -> bool;

    #[method(name = "get_activeInHierarchy")]
    pub fn active_in_hierarchy(self) -> bool;

    #[method(name = "SetActive")]
    pub fn set_active(self, value: bool);

    #[method(name = "get_layer")]
    pub fn layer(self) -> i32;

    #[method(name = "set_layer")]
    pub fn set_layer(self, value: i32);

    #[method(name = "get_tag")]
    pub fn tag(self) -> Il2CppString;

    #[method(name = "set_tag")]
    pub fn set_tag(self, value: Il2CppString);

    #[method(name = "Find")]
    pub fn find(name: Il2CppString) -> GameObject;

    #[method(offset = 0x2C4DEE0)]
    pub fn get_components(self, ty: crate::SystemType) -> crate::Array<Component>;
}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Object)]
pub struct Component {}

#[unity2::methods]
impl Component {
    #[method(name = "get_gameObject")]
    pub fn game_object(self) -> GameObject;

    #[method(name = "get_transform")]
    pub fn transform(self) -> Transform;
}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Component)]
pub struct Transform {}

#[unity2::methods]
impl Transform {
    #[method(name = "get_position")]
    pub fn position(self) -> Vector3;

    #[method(name = "set_position")]
    pub fn set_position(self, value: Vector3);

    #[method(name = "get_localPosition")]
    pub fn local_position(self) -> Vector3;

    #[method(name = "set_localPosition")]
    pub fn set_local_position(self, value: Vector3);

    #[method(name = "get_rotation")]
    pub fn rotation(self) -> Quaternion;

    #[method(name = "set_rotation")]
    pub fn set_rotation(self, value: Quaternion);

    #[method(name = "get_localRotation")]
    pub fn local_rotation(self) -> Quaternion;

    #[method(name = "set_localRotation")]
    pub fn set_local_rotation(self, value: Quaternion);

    #[method(name = "get_localScale")]
    pub fn local_scale(self) -> Vector3;

    #[method(name = "set_localScale")]
    pub fn set_local_scale(self, value: Vector3);

    #[method(name = "get_localEulerAngles")]
    pub fn local_euler_angles(self) -> Vector3;

    #[method(name = "set_localEulerAngles")]
    pub fn set_local_euler_angles(self, value: Vector3);

    #[method(name = "get_parent")]
    pub fn parent(self) -> Transform;

    #[method(name = "SetParent", args = 1)]
    pub fn set_parent(self, parent: Transform);

    #[method(name = "get_childCount")]
    pub fn child_count(self) -> i32;

    #[method(name = "GetChild")]
    pub fn get_child(self, index: i32) -> Transform;

    #[method(name = "Find")]
    pub fn find(self, name: Il2CppString) -> Transform;
}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Component)]
pub struct Behaviour {}

#[unity2::methods]
impl Behaviour {
    #[method(name = "get_enabled")]
    pub fn enabled(self) -> bool;

    #[method(name = "set_enabled")]
    pub fn set_enabled(self, value: bool);

    #[method(name = "get_isActiveAndEnabled")]
    pub fn is_active_and_enabled(self) -> bool;
}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Behaviour)]
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
    pub fn color(self) -> Color;

    #[method(name = "set_color")]
    pub fn set_color(self, value: Color);

    #[method(name = "get_mainTexture")]
    pub fn main_texture(self) -> Texture;

    #[method(name = "set_mainTexture")]
    pub fn set_main_texture(self, value: Texture);

    #[method(name = "HasProperty")]
    pub fn has_property(self, name: Il2CppString) -> bool;
}

#[unity2::enumeration(namespace = "UnityEngine", name = "FilterMode")]
#[repr(i32)]
pub enum FilterMode {
    Point = 0,
    Bilinear = 1,
    Trilinear = 2,
}

// set_filterMode and set_anisoLevel live on Texture rather than Texture2D so subclasses inherit
#[unity2::class(namespace = "UnityEngine")]
#[parent(Object)]
pub struct Texture {}

#[unity2::methods]
impl Texture {
    #[method(name = "get_width", args = 0)]
    pub fn width(self) -> i32;

    #[method(name = "get_height", args = 0)]
    pub fn height(self) -> i32;

    #[method(name = "set_filterMode", args = 1)]
    pub fn set_filter_mode(self, value: FilterMode);

    #[method(name = "get_filterMode", args = 0)]
    pub fn filter_mode(self) -> FilterMode;

    #[method(name = "set_anisoLevel", args = 1)]
    pub fn set_aniso_level(self, value: i32);
}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Texture)]
pub struct Texture2D {}

#[unity2::methods]
impl Texture2D {
    #[method(name = ".ctor", args = 2)]
    pub fn ctor(self, width: i32, height: i32);

    // Three 4-arg overloads exist (DefaultFormat, GraphicsFormat, TextureFormat), pin by RVA
    #[method(offset = 0x378BB90)]
    pub fn ctor_with_format(self, width: i32, height: i32, texture_format: i32, mip_chain: bool);

    #[method(name = "Apply", args = 1)]
    pub fn apply(self, update_mipmaps: bool);

    #[method(name = "get_format", args = 0)]
    pub fn format(self) -> i32;

    #[method(name = "GetRawTextureData", args = 0)]
    pub fn get_raw_texture_data(self) -> crate::Array<u8>;

    #[method(name = "SetPixelDataImplArray", args = 5)]
    pub fn set_pixel_data_impl_array(
        self,
        data: crate::Array<u8>,
        mip_level: i32,
        element_size: i32,
        data_array_size: i32,
        source_data_start_index: i32,
    ) -> bool;
}

impl Texture2D {
    pub fn new(width: i32, height: i32) -> Self {
        let this = <Self as crate::FromIlInstance>::instantiate().expect("Texture2D::instantiate");
        this.ctor(width, height);
        this
    }

    pub fn new_with_format(width: i32, height: i32, texture_format: i32, mip_chain: bool) -> Self {
        let this = <Self as crate::FromIlInstance>::instantiate().expect("Texture2D::instantiate");
        this.ctor_with_format(width, height, texture_format, mip_chain);
        this
    }
}

#[unity2::enumeration(namespace = "UnityEngine", name = "SpriteMeshType")]
#[repr(i32)]
pub enum SpriteMeshType {
    FullRect = 0,
    Tight = 1,
}

#[unity2::class(namespace = "UnityEngine")]
#[parent(Object)]
pub struct Sprite {}

#[unity2::methods]
impl Sprite {
    #[method(name = "get_texture", args = 0)]
    pub fn texture(self) -> Texture2D;

    #[method(name = "Create", args = 6)]
    pub fn create(
        texture: Texture2D,
        rect: Rect,
        pivot: Vector2,
        pixels_per_unit: f32,
        extrude: u32,
        mesh_type: SpriteMeshType,
    ) -> Sprite;
}

#[unity2::class(namespace = "UnityEngine")]
pub struct ImageConversion {}

#[unity2::methods]
impl ImageConversion {
    #[method(name = "LoadImage", args = 2)]
    pub fn load_image(texture: Texture2D, data: crate::Array<u8>) -> bool;

    #[method(name = "EncodeToPNG", args = 1)]
    pub fn encode_to_png(texture: Texture2D) -> crate::Array<u8>;
}
