#![allow(unused_imports)]

use crate::ClassIdentity;

use crate::engine::object::{IObject, Object};

// Real parent is RenderPipelineAsset, not mirrored yet, parent to Object until needed
#[unity2::class(namespace = "UnityEngine.Rendering.Universal")]
#[parent(Object)]
pub struct UniversalRenderPipelineAsset {
    #[rename(name = "m_RenderScale")]
    pub render_scale: f32,
}

#[unity2::methods]
impl UniversalRenderPipelineAsset {
    #[method(name = "get_renderScale", args = 0)]
    pub fn get_render_scale(self) -> f32;

    #[method(name = "set_renderScale", args = 1)]
    pub fn set_render_scale(self, value: f32);
}
