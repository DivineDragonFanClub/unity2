#![allow(unused_imports)]

use crate::{ClassIdentity, Il2CppString};

use super::object::{IObject, Object, Sprite};

#[unity2::class(namespace = "UnityEngine.U2D")]
#[parent(Object)]
pub struct SpriteAtlas {}

#[unity2::methods]
impl SpriteAtlas {
    #[method(name = "GetSprite", args = 1)]
    pub fn get_sprite(self, name: Il2CppString) -> Sprite;

    #[method(name = "get_spriteCount", args = 0)]
    pub fn sprite_count(self) -> i32;
}
