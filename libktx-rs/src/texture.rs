use crate::{
    enums::CreateStorage,
    sys,
    sys::stream::{RWSeekable, RustKtxStream},
};
use std::{marker::PhantomData, sync::Arc};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Format {
    /// For KTX1
    Gl(u32),
    /// For KTX2
    Vk(u32),
}

#[derive(Debug, PartialEq, Eq)]
pub struct TextureCreateInfo {
    pub format: Format,
    pub dfd: Vec<u32>,
    pub base_width: u32,
    pub base_height: u32,
    pub base_depth: u32,
    pub num_dimensions: u32,
    pub num_levels: u32,
    pub num_layers: u32,
    pub num_faces: u32,
    pub is_array: bool,
    pub generate_mipmaps: bool,
}

impl Default for TextureCreateInfo {
    fn default() -> Self {
        TextureCreateInfo {
            format: Format::Vk(37), // VK_R8G8B8A8_UNORM
            dfd: vec![],
            base_width: 1,
            base_height: 1,
            base_depth: 1,
            num_dimensions: 1,
            num_levels: 1,
            num_layers: 1,
            num_faces: 1,
            is_array: false,
            generate_mipmaps: false,
        }
    }
}

pub struct Texture<'a> {
    handle: *mut sys::ktxTexture,
    handle_phantom: PhantomData<&'a sys::ktxTexture>,
    #[allow(dead_code)] // This field is actually required from unsafe code
    dfd: Option<Arc<Vec<u32>>>,
}

impl<'a> Texture<'a> {
    pub fn create(
        create_info: TextureCreateInfo,
        storage_alloc: CreateStorage,
    ) -> Result<Self, sys::ktx_error_code_e> {
        let mut sys_create_info = sys::ktxTextureCreateInfo {
            glInternalformat: 0,        // Set later for KTX1
            vkFormat: 0,                // Set later for KTX2
            pDfd: std::ptr::null_mut(), // Set later for KTX2 if vkFormat == UNDEFINED
            baseWidth: create_info.base_width,
            baseHeight: create_info.base_height,
            baseDepth: create_info.base_depth,
            numDimensions: create_info.num_dimensions,
            numLevels: create_info.num_levels,
            numLayers: create_info.num_layers,
            numFaces: create_info.num_faces,
            isArray: create_info.is_array,
            generateMipmaps: create_info.generate_mipmaps,
        };

        let mut dfd: Option<Arc<Vec<u32>>> = None;
        let mut handle: *mut sys::ktxTexture = std::ptr::null_mut();
        let handle_ptr: *mut *mut sys::ktxTexture = &mut handle;

        let err = match create_info.format {
            Format::Gl(internal_format) => unsafe {
                sys_create_info.glInternalformat = internal_format;
                sys::ktxTexture1_Create(
                    &mut sys_create_info,
                    storage_alloc as u32,
                    handle_ptr as *mut *mut sys::ktxTexture1,
                )
            },
            Format::Vk(format) => unsafe {
                sys_create_info.vkFormat = format;
                dfd = Some({
                    let arc = Arc::new(create_info.dfd);
                    // SAFETY: the contents of the Vec will not change or move around memory
                    // - libKTX does not modify the given DFD pointer
                    //   (but then, why no `const` in the C API pointer?)
                    // - The Vec's data is read-only from now on (= no reallocations are possible)
                    // - The Vec itself is in a Arc (= it cannot move around memory)
                    sys_create_info.pDfd = arc.as_ptr() as *mut u32;
                    arc
                });
                sys::ktxTexture2_Create(
                    &mut sys_create_info,
                    storage_alloc as u32,
                    handle_ptr as *mut *mut sys::ktxTexture2,
                )
            },
        };

        if err == sys::ktx_error_code_e_KTX_SUCCESS && !handle.is_null() {
            Ok(Texture {
                handle,
                handle_phantom: PhantomData,
                dfd,
            })
        } else {
            Err(err)
        }
    }
}

impl<'a> Drop for Texture<'a> {
    fn drop(&mut self) {
        unsafe {
            let vtbl = (*self.handle).vtbl;
            if let Some(destroy_fn) = (*vtbl).Destroy {
                (destroy_fn)(self.handle as *mut sys::ktxTexture);
            }
        }
    }
}

pub struct StreamTexture<'a> {
    stream: RustKtxStream<'a>,
    texture: Texture<'a>,
}

impl<'a> StreamTexture<'a> {
    pub fn create(
        stream: Box<dyn RWSeekable + 'a>,
        create_flags: sys::ktxTextureCreateFlags,
    ) -> Result<Self, String> {
        let stream = RustKtxStream::new(stream)?;

        let mut handle: *mut sys::ktxTexture = std::ptr::null_mut();
        let handle_ptr: *mut *mut sys::ktxTexture = &mut handle;
        let err = unsafe {
            sys::ktxTexture_CreateFromStream(stream.ktx_stream(), create_flags, handle_ptr)
        };
        if err == sys::ktx_error_code_e_KTX_SUCCESS && !handle.is_null() {
            Ok(StreamTexture {
                stream,
                texture: Texture {
                    handle,
                    handle_phantom: PhantomData,
                    dfd: None,
                },
            })
        } else {
            // TODO proper formatting
            Err(format!("{}", err))
        }
    }

    pub fn texture(&self) -> &Texture<'a> {
        &self.texture
    }

    pub fn texture_mut(&mut self) -> &mut Texture<'a> {
        &mut self.texture
    }
}
