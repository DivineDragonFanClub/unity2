#![allow(unused_imports)]

use crate::system::Il2CppString;
use crate::{Array, ClassIdentity, IlInstance};

use super::object::{IObject, Object};

#[unity2::class(namespace = "UnityEngine")]
#[parent(Object)]
pub struct AssetBundle {}

#[unity2::methods]
impl AssetBundle {
    #[method(name = "LoadFromFileAsync", args = 1)]
    pub fn load_from_file_async(path: Il2CppString) -> AssetBundleCreateRequest;

    #[method(name = "LoadFromMemory", args = 1)]
    pub fn load_from_memory(binary: Array<u8>) -> AssetBundle;

    #[method(name = "LoadFromMemory_Internal", args = 2)]
    pub fn load_from_memory_internal(binary: Array<u8>, crc: u32) -> AssetBundle;

    #[method(name = "Unload", args = 1)]
    pub fn unload(self, unload_all_loaded_objects: bool);
}

impl AssetBundle {
    pub fn load_from_memory_async_internal(binary: Array<u8>, crc: u32) -> AssetBundleCreateRequest {
        static METHOD_PTR: ::std::sync::OnceLock<usize> = ::std::sync::OnceLock::new();
        let ptr = *METHOD_PTR.get_or_init(|| {
            let name = ::std::ffi::CString::new(
                "UnityEngine.AssetBundle::LoadFromMemoryAsync_Internal(System.Byte[],System.UInt32)",
            )
            .unwrap();
            let resolved = unsafe { skyline_method_from_name(name.as_ptr()) };
            if resolved.is_null() {
                panic!(
                    "AssetBundle::load_from_memory_async_internal: signature lookup failed for {:?}",
                    name
                );
            }
            resolved as usize
        });

        type RawFn = extern "C" fn(
            Array<u8>,
            u32,
            crate::OptionalMethod,
        ) -> AssetBundleCreateRequest;
        let f: RawFn = unsafe { ::std::mem::transmute(ptr) };
        f(binary, crc, None)
    }
}

#[::skyline::from_offset(0x491ff0)]
fn skyline_method_from_name(name: *const ::std::os::raw::c_char) -> *const u8;

#[unity2::class(namespace = "UnityEngine")]
#[parent(Object)]
pub struct AssetBundleCreateRequest {}

#[unity2::methods]
impl AssetBundleCreateRequest {
    #[method(name = "get_assetBundle", args = 0)]
    pub fn asset_bundle(self) -> AssetBundle;
}
