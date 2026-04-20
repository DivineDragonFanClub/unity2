use std::sync::OnceLock;

use crate::class::Class;
use crate::ClassIdentity;

macro_rules! impl_engine_class_identity {
    ($($ty:ident => $name:literal),* $(,)?) => {
        $(
            impl ClassIdentity for $ty {
                const NAMESPACE: &'static str = "UnityEngine";
                const NAME: &'static str = $name;

                fn class() -> Class {
                    static CACHE: OnceLock<Class> = OnceLock::new();
                    *CACHE.get_or_init(|| Class::lookup(Self::NAMESPACE, Self::NAME))
                }
            }
        )*
    };
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0 };
    pub const UP: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const FORWARD: Self = Self { x: 0.0, y: 0.0, z: 1.0 };
    pub const RIGHT: Self = Self { x: 1.0, y: 0.0, z: 0.0 };

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4 {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Vector2Int {
    pub x: i32,
    pub y: i32,
}

impl Vector2Int {
    #[inline]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Vector3Int {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vector3Int {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    #[inline]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Bounds {
    pub center: Vector3,
    pub extents: Vector3,
}

impl Bounds {
    #[inline]
    pub const fn new(center: Vector3, extents: Vector3) -> Self {
        Self { center, extents }
    }
}

impl_engine_class_identity! {
    Vector2    => "Vector2",
    Vector3    => "Vector3",
    Vector4    => "Vector4",
    Vector2Int => "Vector2Int",
    Vector3Int => "Vector3Int",
    Quaternion => "Quaternion",
    Rect       => "Rect",
    Bounds     => "Bounds",
}
