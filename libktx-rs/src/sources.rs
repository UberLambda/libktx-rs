use crate::{
    enums::{CreateStorage, TextureCreateFlags},
    sys::{self, stream::RustKtxStream},
    texture::{Texture, TextureSource},
    KtxError,
};
use std::{convert::TryInto, marker::PhantomData};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommonCreateInfo {
    pub create_storage: CreateStorage,
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

impl Default for CommonCreateInfo {
    fn default() -> Self {
        CommonCreateInfo {
            create_storage: CreateStorage::AllocStorage,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ktx1CreateInfo {
    pub gl_internal_format: u32,
    pub common: CommonCreateInfo,
}

impl Default for Ktx1CreateInfo {
    fn default() -> Self {
        Ktx1CreateInfo {
            gl_internal_format: 0x8058, // GL_RGBA8
            common: Default::default(),
        }
    }
}

fn try_create_texture<'a, S, C>(source: S, create_fn: C) -> Result<Texture<'a>, KtxError>
where
    S: TextureSource<'a> + 'a,
    C: FnOnce(S) -> (S, sys::ktx_error_code_e, *mut sys::ktxTexture),
{
    let (source, err, handle) = (create_fn)(source);
    if err == sys::ktx_error_code_e_KTX_SUCCESS && !handle.is_null() {
        Ok(Texture {
            source: Box::new(source),
            handle,
            handle_phantom: PhantomData,
        })
    } else {
        Err(err.try_into().unwrap_or(KtxError::InvalidOperation))
    }
}

impl<'a> TextureSource<'a> for Ktx1CreateInfo {
    fn create_texture(self) -> Result<Texture<'a>, KtxError> {
        let mut sys_create_info = sys::ktxTextureCreateInfo {
            glInternalformat: self.gl_internal_format,
            vkFormat: 0,
            pDfd: std::ptr::null_mut(),
            baseWidth: self.common.base_width,
            baseHeight: self.common.base_height,
            baseDepth: self.common.base_depth,
            numDimensions: self.common.num_dimensions,
            numLevels: self.common.num_levels,
            numLayers: self.common.num_layers,
            numFaces: self.common.num_faces,
            isArray: self.common.is_array,
            generateMipmaps: self.common.generate_mipmaps,
        };

        try_create_texture(self, |source| {
            let mut handle: *mut sys::ktxTexture = std::ptr::null_mut();
            let handle_ptr: *mut *mut sys::ktxTexture = &mut handle;

            let err = unsafe {
                sys::ktxTexture1_Create(
                    &mut sys_create_info,
                    source.common.create_storage as u32,
                    handle_ptr as *mut *mut sys::ktxTexture1,
                )
            };
            (source, err, handle)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ktx2CreateInfo {
    pub vk_format: u32,
    pub dfd: Option<Vec<u32>>,
    pub common: CommonCreateInfo,
}

impl Default for Ktx2CreateInfo {
    fn default() -> Self {
        Ktx2CreateInfo {
            vk_format: 37, // VK_R8G8B8A8_UNORM
            dfd: None,
            common: Default::default(),
        }
    }
}

impl<'a> TextureSource<'a> for Ktx2CreateInfo {
    fn create_texture(mut self) -> Result<Texture<'a>, KtxError> {
        // SAFETY: the contents of the Vec will not change or move around memory
        // - libKTX does not modify the given DFD pointer
        //   (but then, why no `const` in the C API pointer?)
        // - The Vec's data is read-only from now on (= no reallocations are possible)
        let dfd_ptr = match &mut self.dfd {
            Some(dfd_data) => dfd_data.as_mut_ptr() as *mut u32,
            None => std::ptr::null_mut(),
        };

        let mut sys_create_info = sys::ktxTextureCreateInfo {
            glInternalformat: 0,
            vkFormat: self.vk_format,
            pDfd: dfd_ptr,
            baseWidth: self.common.base_width,
            baseHeight: self.common.base_height,
            baseDepth: self.common.base_depth,
            numDimensions: self.common.num_dimensions,
            numLevels: self.common.num_levels,
            numLayers: self.common.num_layers,
            numFaces: self.common.num_faces,
            isArray: self.common.is_array,
            generateMipmaps: self.common.generate_mipmaps,
        };

        try_create_texture(self, |source| {
            let mut handle: *mut sys::ktxTexture = std::ptr::null_mut();
            let handle_ptr: *mut *mut sys::ktxTexture = &mut handle;

            let err = unsafe {
                sys::ktxTexture2_Create(
                    &mut sys_create_info,
                    source.common.create_storage as u32,
                    handle_ptr as *mut *mut sys::ktxTexture2,
                )
            };
            (source, err, handle)
        })
    }
}

#[derive(Debug)]
pub struct StreamSource<'a> {
    pub stream: RustKtxStream<'a>,
    pub texture_create_flags: TextureCreateFlags,
}

impl<'a> TextureSource<'a> for StreamSource<'a> {
    fn create_texture(self) -> Result<Texture<'a>, KtxError> {
        try_create_texture(self, |source| {
            let mut handle: *mut sys::ktxTexture = std::ptr::null_mut();
            let handle_ptr: *mut *mut sys::ktxTexture = &mut handle;

            let err = unsafe {
                sys::ktxTexture_createFromStream(
                    source.stream.ktx_stream(),
                    source.texture_create_flags.bits(),
                    handle_ptr,
                )
            };
            (source, err, handle)
        })
    }
}