#![allow(unused_imports)]

use crate::{ClassIdentity, Il2CppString};

use super::color::Color;
use super::object::{IObject, Object, Sprite};

#[unity2::class(namespace = "UnityEngine.UI")]
#[parent(Object)]
pub struct Image {}

#[unity2::methods]
impl Image {
    #[method(name = "get_sprite", args = 0)]
    pub fn sprite(self) -> Sprite;

    #[method(name = "set_sprite", args = 1)]
    pub fn set_sprite(self, value: Sprite);

    #[method(name = "get_overrideSprite", args = 0)]
    pub fn override_sprite(self) -> Sprite;

    #[method(name = "set_overrideSprite", args = 1)]
    pub fn set_override_sprite(self, value: Sprite);

    #[method(name = "set_color", args = 1)]
    pub fn set_color(self, value: Color);

    #[method(name = "get_color", args = 0)]
    pub fn color(self) -> Color;
}

#[unity2::class(namespace = "TMPro")]
#[parent(Object)]
pub struct TextMeshProUGUI {}

#[unity2::methods]
impl TextMeshProUGUI {
    #[method(name = "set_text", args = 1)]
    pub fn set_text(self, value: Il2CppString);

    #[method(name = "get_text", args = 0)]
    pub fn text(self) -> Il2CppString;

    #[method(name = "set_color", args = 1)]
    pub fn set_color(self, value: Color);
}
