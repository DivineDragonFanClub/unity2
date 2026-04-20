use std::sync::OnceLock;

use crate::class::Class;
use crate::ClassIdentity;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const CLEAR: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);

    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color32 {
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const CLEAR: Self = Self { r: 0, g: 0, b: 0, a: 0 };

    #[inline]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[inline]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

impl ClassIdentity for Color {
    const NAMESPACE: &'static str = "UnityEngine";
    const NAME: &'static str = "Color";

    fn class() -> Class {
        static CACHE: OnceLock<Class> = OnceLock::new();
        *CACHE.get_or_init(|| Class::lookup(Self::NAMESPACE, Self::NAME))
    }
}

impl ClassIdentity for Color32 {
    const NAMESPACE: &'static str = "UnityEngine";
    const NAME: &'static str = "Color32";

    fn class() -> Class {
        static CACHE: OnceLock<Class> = OnceLock::new();
        *CACHE.get_or_init(|| Class::lookup(Self::NAMESPACE, Self::NAME))
    }
}
