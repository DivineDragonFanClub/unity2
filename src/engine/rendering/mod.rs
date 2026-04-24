#![allow(unused_imports)]

pub mod universal;

use crate::ClassIdentity;

use super::color::Color;

pub use universal::UniversalRenderPipelineAsset;

#[unity2::class(namespace = "UnityEngine.Rendering")]
pub struct ColorUtils {}

#[unity2::methods]
impl ColorUtils {
    #[method(name = "ToRGBA", args = 1)]
    pub fn to_rgba(hex: u32) -> Color;

    #[method(name = "ToHex", args = 1)]
    pub fn to_hex(c: Color) -> u32;
}
