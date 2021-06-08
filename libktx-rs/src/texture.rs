use crate::sys;
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
        storage_alloc: sys::ktxTextureCreateStorageEnum,
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

        #[allow(unused_mut)] // This pointer is modified by C code!
        let mut handle: *mut sys::ktxTexture = std::ptr::null_mut();
        let mut dfd: Option<Arc<Vec<u32>>> = None;
        let err = match create_info.format {
            Format::Gl(internal_format) => unsafe {
                sys_create_info.glInternalformat = internal_format;
                sys::ktxTexture1_Create(
                    &mut sys_create_info,
                    storage_alloc,
                    &mut (handle as *mut sys::ktxTexture1),
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
                    storage_alloc,
                    &mut (handle as *mut sys::ktxTexture2),
                )
            },
        };

        if err == sys::ktx_error_code_e_KTX_SUCCESS {
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
