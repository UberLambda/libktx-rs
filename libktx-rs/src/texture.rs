// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    enums::{ktx_result, TranscodeFlags, TranscodeFormat},
    sys, KtxError,
};
use std::marker::PhantomData;
pub trait TextureSource<'a> {
    fn create_texture(self) -> Result<Texture<'a>, KtxError>;
}

#[cfg(feature = "write")]
pub trait TextureSink {
    fn write_texture(&mut self, texture: &Texture) -> Result<(), KtxError>;
}

pub struct Texture<'a> {
    // Not actually dead - there most likely are raw pointers referencing this!!
    #[allow(dead_code)]
    pub(crate) source: Box<dyn TextureSource<'a> + 'a>,
    pub(crate) handle: *mut sys::ktxTexture,
    pub(crate) handle_phantom: PhantomData<&'a sys::ktxTexture>,
}

impl<'a> Texture<'a> {
    pub fn new<S>(source: S) -> Result<Self, KtxError>
    where
        S: TextureSource<'a>,
    {
        source.create_texture()
    }

    /// Returns the pointer to the (C-allocated) underlying [`sys::ktxTexture`].
    ///
    /// **SAFETY**: Pointers are harmless. Dereferencing them is not!
    pub fn handle(&self) -> *mut sys::ktxTexture {
        self.handle
    }

    /// Returns the total size of image data, in bytes.
    pub fn data_size(&self) -> usize {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { sys::ktxTexture_GetDataSize(self.handle) as usize }
    }

    /// Returns a read-only view on the image data.
    pub fn data(&self) -> &[u8] {
        let data = unsafe { sys::ktxTexture_GetData(self.handle) };
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { std::slice::from_raw_parts(data, self.data_size()) }
    }

    /// Returns a read-write view on the image data.
    pub fn data_mut(&mut self) -> &mut [u8] {
        let data = unsafe { sys::ktxTexture_GetData(self.handle) };
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { std::slice::from_raw_parts_mut(data, self.data_size()) }
    }

    /// Returns the pitch (in bytes) of an image row at the specified image level.  
    /// This is rounded up to 1 if needed.
    pub fn row_pitch(&self, level: u32) -> usize {
        // SAFETY: Safe if `self.handle` is sane.
        //         `level` is not used for indexing internally; no bounds-checking required.
        unsafe { sys::ktxTexture_GetRowPitch(self.handle, level) as usize }
    }

    /// Returns the size (in bytes) of an element of the image.
    pub fn element_size(&self) -> usize {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { sys::ktxTexture_GetElementSize(self.handle) as usize }
    }

    /// Writes the texture to the given sink in its native format (KTX1 or KTX2).
    #[cfg(feature = "write")]
    pub fn write_to<T: TextureSink>(&self, sink: &mut T) -> Result<(), KtxError> {
        sink.write_texture(self)
    }

    /// If this [`Texture`] really is a KTX1, returns KTX1-specific functionalities for it.
    pub fn ktx1<'b>(&'b mut self) -> Option<Ktx1<'b, 'a>> {
        // SAFETY: Safe if `self.handle` is sane.
        if unsafe { *self.handle }.classId == sys::class_id_ktxTexture1_c {
            Some(Ktx1 { texture: self })
        } else {
            None
        }
    }

    /// If this [`Texture`] really is a KTX2, returns KTX2-specific functionalities for it.
    pub fn ktx2<'b>(&'b mut self) -> Option<Ktx2<'b, 'a>> {
        // SAFETY: Safe if `self.handle` is sane.
        if unsafe { *self.handle }.classId == sys::class_id_ktxTexture2_c {
            Some(Ktx2 { texture: self })
        } else {
            None
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

pub struct Ktx1<'a, 'b: 'a> {
    texture: &'a mut Texture<'b>,
}

impl<'a, 'b: 'a> Ktx1<'a, 'b> {
    /// Returns a pointer to the underlying (C-allocated) [`sys::ktxTexture1`].
    ///
    /// **SAFETY**: Pointers are harmless. Dereferencing them is not!
    pub fn handle(&self) -> *mut sys::ktxTexture1 {
        self.texture.handle as *mut sys::ktxTexture1
    }

    /// Will this KTX1 need transcoding?
    pub fn needs_transcoding(&self) -> bool {
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX1
        unsafe { sys::ktxTexture1_NeedsTranscoding(self.handle()) }
    }

    // TODO: WriteKTX2ToStream with a Rust stream (and to a memory slice?)
    //       Probably needs a TextureSink trait
}

pub struct Ktx2<'a, 'b: 'a> {
    texture: &'a mut Texture<'b>,
}

impl<'a, 'b: 'a> Ktx2<'a, 'b> {
    /// Returns a pointer to the underlying (C-allocated) [`sys::ktxTexture2`].
    ///
    /// **SAFETY**: Pointers are harmless. Dereferencing them is not!
    pub fn handle(&self) -> *mut sys::ktxTexture2 {
        self.texture.handle as *mut sys::ktxTexture2
    }

    /// Will this KTX2 need transcoding?
    pub fn needs_transcoding(&self) -> bool {
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX2
        unsafe { sys::ktxTexture2_NeedsTranscoding(self.handle()) }
    }

    /// Compresses a uncompressed KTX2 texture with Basis Universal.  
    /// `quality` is 1-255; 0 -> the default quality, 128. **Lower `quality` means better (but slower) compression**.
    pub fn compress_basis(&mut self, quality: u32) -> Result<(), KtxError> {
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX2
        let errcode = unsafe { sys::ktxTexture2_CompressBasis(self.handle(), quality as u32) };
        ktx_result(errcode, ())
    }

    /// Compresses the KTX2 texture's data with ZStandard compression.  
    /// `level` is 1-22; lower is faster (hence, worse compression).  
    /// Values over 20 may consume significant memory.
    pub fn deflate_zstd(&mut self, level: u32) -> Result<(), KtxError> {
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX2
        let errcode = unsafe { sys::ktxTexture2_DeflateZstd(self.handle(), level as u32) };
        ktx_result(errcode, ())
    }

    /// Returns the number of components of the KTX2 and the size in bytes of each components.
    pub fn component_info(&self) -> (u32, u32) {
        let mut num_components: u32 = 0;
        let mut component_size: u32 = 0;
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX2
        unsafe {
            sys::ktxTexture2_GetComponentInfo(
                self.handle(),
                &mut num_components,
                &mut component_size,
            );
        }
        (num_components, component_size)
    }

    /// Returns the number of components of the KTX2, also considering compression.  
    ///
    /// **This may differ from values returned by [`component_info`]:**
    /// - For uncompressed formats: this is the number of image components, as from [`component_info`].
    /// - For block-compressed formats: 1 or 2, according to the DFD color model.
    /// - For Basis Universal-compressed textures: obtained by parsing channel IDs before any encoding and deflation.
    ///
    /// See [`sys::ktxTexture2_GetNumComponents`].
    pub fn num_components(&self) -> u32 {
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX2
        unsafe { sys::ktxTexture2_GetNumComponents(self.handle()) }
    }

    /// Returns the Opto-Electrical Transfer Function (OETF) for this KTX2, in KHR_DF format.  
    /// See <https://www.khronos.org/registry/DataFormat/specs/1.3/dataformat.1.3.inline.html#_emphasis_role_strong_emphasis_transferfunction_emphasis_emphasis>.
    pub fn oetf(&self) -> u32 {
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX2
        unsafe { sys::ktxTexture2_GetOETF(self.handle()) }
    }

    /// Does this KTX2 have premultiplied alpha?
    pub fn premultiplied_alpha(&self) -> bool {
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX2
        unsafe { sys::ktxTexture2_GetPremultipliedAlpha(self.handle()) }
    }

    /// Transcodes this KTX2 to the given format by using ETC1S (from Basis Universal) or UASTC.
    ///
    /// - BasisLZ supercompressed textures are turned back to ETC1S, then transcoded.
    /// - UASTC-compressed images are inflated (possibly, even deflating any ZStandard supercompression), then transcoded.
    /// - **All internal data of the texture may change, including the
    /// [DFD](https://www.khronos.org/registry/DataFormat/specs/1.3/dataformat.1.3.inline.html#_anchor_id_dataformatdescriptor_xreflabel_dataformatdescriptor_khronos_data_format_descriptor)**!
    pub fn transcode_basis(
        &mut self,
        format: TranscodeFormat,
        flags: TranscodeFlags,
    ) -> Result<(), KtxError> {
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX2
        let errcode =
            unsafe { sys::ktxTexture2_TranscodeBasis(self.handle(), format as u32, flags.bits()) };
        ktx_result(errcode, ())
    }
}
