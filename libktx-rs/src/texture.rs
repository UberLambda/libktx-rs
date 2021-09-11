// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0

//! Core types involving KTX [`Texture`]s.

use crate::{
    enums::{
        ktx_result, Orientations, PackAstcBlockDimension, PackAstcEncoderFunction,
        PackAstcEncoderMode, PackAstcQualityLevel, SuperCompressionScheme, TranscodeFlags,
        TranscodeFormat,
    },
    sys, KtxError,
};
use std::{convert::TryInto, marker::PhantomData};

/// A source of [`Texture`]s.
pub trait TextureSource<'a> {
    /// Attempts to create a new texture by consuming `self`.  
    fn create_texture(self) -> Result<Texture<'a>, KtxError>;
}

/// A sink of [`Texture`]s, e.g. something they can be written to.
#[cfg(feature = "write")]
pub trait TextureSink {
    /// Attempts to write `texture` to `self`.
    fn write_texture(&mut self, texture: &Texture) -> Result<(), KtxError>;
}

/// Parameters for ASTC compression.
///
/// This only applies to Arm's ASTC encoder, which is in `libktx-rs-sys/build/KTX-Software/lib/astc-encoder`.  
/// See [`sys::ktxAstcParams`] for information on the various fields.
pub struct AstcParams {
    pub verbose: bool,
    pub thread_count: u32,
    pub block_dimension: PackAstcBlockDimension,
    pub function: PackAstcEncoderFunction,
    pub mode: PackAstcEncoderMode,
    pub quality_level: PackAstcQualityLevel,
    pub normal_map: bool,
    pub input_swizzle: [char; 4],
}

/// A KTX (1 or 2) texture.
///
/// This wraps both a [`sys::ktxTexture`] handle, and the [`TextureSource`] it was created from.
pub struct Texture<'a> {
    // Not actually dead - there most likely are raw pointers referencing this!!
    #[allow(dead_code)]
    pub(crate) source: Box<dyn TextureSource<'a> + 'a>,
    pub(crate) handle: *mut sys::ktxTexture,
    pub(crate) handle_phantom: PhantomData<&'a sys::ktxTexture>,
}

impl<'a> Texture<'a> {
    /// Attempts to create a new texture, consuming the given [`TextureSource`].
    pub fn new<S>(source: S) -> Result<Self, KtxError>
    where
        S: TextureSource<'a>,
    {
        source.create_texture()
    }

    /// Attempts to write the texture (in its native format, either KTX1 or KTX2) to `sink`.
    #[cfg(feature = "write")]
    pub fn write_to<T: TextureSink>(&self, sink: &mut T) -> Result<(), KtxError> {
        sink.write_texture(self)
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

    /// Returns whether this texture is an array texture or not.
    pub fn is_array(&self) -> bool {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { (*self.handle).isArray }
    }

    /// Returns whether this texture is a cubemap or not.
    pub fn is_cubemap(&self) -> bool {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { (*self.handle).isCubemap }
    }

    /// Returns whether this texture is compressed or not.
    pub fn is_compressed(&self) -> bool {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { (*self.handle).isCompressed }
    }

    /// Returns the width (in texels) of this texture's base level.
    pub fn base_width(&self) -> usize {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { (*self.handle).baseWidth as usize }
    }

    /// Returns the height (in texels) of this texture's base level.
    pub fn base_height(&self) -> usize {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { (*self.handle).baseHeight as usize }
    }

    /// Returns the depth (in texels) of this texture's base level.
    pub fn base_depth(&self) -> usize {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { (*self.handle).baseDepth as usize }
    }

    /// Returns the number of dimensions in this texture (1, 2 or 3).
    pub fn num_dimensions(&self) -> usize {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { (*self.handle).numDimensions as usize }
    }

    /// Returns the number of mipmap levels in this texture.
    ///
    /// This must be 1 if pre-upload mipmap generation was enabled by the library.
    pub fn num_levels(&self) -> usize {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { (*self.handle).numLevels as usize }
    }

    /// Returns the number of array layers in this texture.
    pub fn num_layers(&self) -> usize {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { (*self.handle).numLayers as usize }
    }

    /// Returns the number of faces in this texture. It is 1 for standard images, and 6 for cubemaps.
    pub fn num_faces(&self) -> usize {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { (*self.handle).numLayers as usize }
    }

    /// Returns the logical orientation of this texture in all possible directions (X, Y and Z).
    pub fn orientation(&self) -> Orientations {
        // SAFETY: Safe if `self.handle` is sane.
        // PANICS: Theoretically never. The `try_into()` on orientations returned by the C library should never fail.
        let c_orientation = unsafe { (*self.handle).orientation };
        Orientations {
            x: (c_orientation.x as sys::ktxOrientationX)
                .try_into()
                .unwrap(),
            y: (c_orientation.y as sys::ktxOrientationY)
                .try_into()
                .unwrap(),
            z: (c_orientation.z as sys::ktxOrientationZ)
                .try_into()
                .unwrap(),
        }
    }

    /// Attempts to return the offset (in bytes) into [`Self::data`] for the image
    /// at the given mip level, array layer, and slice.  
    /// `slice` is either a cubemap's face or a 3D texture's depth slice.
    pub fn get_image_offset(&self, level: u32, layer: u32, slice: u32) -> Result<usize, KtxError> {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe {
            let vtbl = (*self.handle).vtbl;
            if let Some(get_image_offset_fn) = (*vtbl).GetImageOffset {
                let mut offset = 0usize;
                let err = get_image_offset_fn(self.handle, level, layer, slice, &mut offset);
                ktx_result(err, offset)
            } else {
                Err(KtxError::InvalidValue)
            }
        }
    }

    /// Attempts to return the size (in bytes) of the uncompressed image data.
    pub fn get_data_size_uncompressed(&self) -> Result<usize, KtxError> {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe {
            let vtbl = (*self.handle).vtbl;
            if let Some(get_data_size_fn) = (*vtbl).GetDataSizeUncompressed {
                Ok((get_data_size_fn)(self.handle))
            } else {
                Err(KtxError::InvalidValue)
            }
        }
    }

    /// Attempts to return the size (in bytes) of a certain mip level.
    pub fn get_image_size(&self, level: u32) -> Result<usize, KtxError> {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe {
            let vtbl = (*self.handle).vtbl;
            if let Some(get_image_size_fn) = (*vtbl).GetImageSize {
                Ok((get_image_size_fn)(self.handle, level))
            } else {
                Err(KtxError::InvalidValue)
            }
        }
    }

    /// Attempts to [re]load this image's data to its internal buffer.
    /// Also see [`Self::data()`].
    ///
    /// Creating the image with [`enums::TextureCreateFlags::LOAD_IMAGE_DATA`] performs this step automatically on load.
    pub fn load_image_data(&self) -> Result<(), KtxError> {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe {
            let vtbl = (*self.handle).vtbl;
            if let Some(load_image_data_fn) = (*vtbl).LoadImageData {
                let err = (load_image_data_fn)(self.handle, std::ptr::null_mut(), 0usize);
                ktx_result(err, ())
            } else {
                Err(KtxError::InvalidValue)
            }
        }
    }

    /// Attempts to iterate all mip levels of the image, and all faces of cubemaps.
    /// This calls
    /// ```rust,ignore
    /// callback(miplevel: i32, face: i32, width: i32, height: i32, depth: i32, pixel_data: &[u8]) -> Result<(), KtxError>
    /// ```
    /// for each level/face. The image data passed to the callback is immutable.
    /// Note that image data should already have been loaded (see [`Self::load_image_data()`]).
    pub fn iterate_levels<F>(&self, mut callback: F) -> Result<(), KtxError>
    where
        F: FnMut(i32, i32, i32, i32, i32, &[u8]) -> Result<(), KtxError>,
    {
        unsafe extern "C" fn c_iterator_fn<F>(
            mip: i32,
            face: i32,
            width: i32,
            height: i32,
            depth: i32,
            pixels_size: u64,
            pixels: *mut std::ffi::c_void,
            closure_ptr: *mut std::ffi::c_void,
        ) -> sys::ktx_error_code_e
        where
            F: FnMut(i32, i32, i32, i32, i32, &[u8]) -> Result<(), KtxError>,
        {
            let closure = closure_ptr as *mut F;
            let pixels_slice =
                std::slice::from_raw_parts(pixels as *const u8, pixels_size as usize);
            match (*closure)(mip, face, width, height, depth, pixels_slice) {
                Ok(_) => sys::ktx_error_code_e_KTX_SUCCESS,
                Err(code) => code as u32,
            }
        }

        // SAFETY: Safe if `self.handle` is sane.
        unsafe {
            if (*self.handle).pData.is_null() {
                // Data was not loaded
                return Err(KtxError::InvalidValue);
            }

            let vtbl = (*self.handle).vtbl;
            if let Some(iterate_levels_fn) = (*vtbl).IterateLevels {
                let closure_ptr = &mut callback as *mut F as *mut std::ffi::c_void;
                let err = (iterate_levels_fn)(self.handle, Some(c_iterator_fn::<F>), closure_ptr);
                ktx_result(err, ())
            } else {
                Err(KtxError::InvalidValue)
            }
        }
    }

    /// Attempts to iterate all mip levels of the image, and all faces of cubemaps.
    /// This calls
    /// ```rust,ignore
    /// callback(miplevel: i32, face: i32, width: i32, height: i32, depth: i32, pixel_data: &mut [u8]) -> Result<(), KtxError>
    /// ```
    /// for each level/face. The image data passed to the callback is mutable.
    /// Note that image data should already have been loaded (see [`Self::load_image_data()`]).
    pub fn iterate_levels_mut<F>(&mut self, mut callback: F) -> Result<(), KtxError>
    where
        F: FnMut(i32, i32, i32, i32, i32, &mut [u8]) -> Result<(), KtxError>,
    {
        unsafe extern "C" fn c_iterator_fn<F>(
            mip: i32,
            face: i32,
            width: i32,
            height: i32,
            depth: i32,
            pixels_size: u64,
            pixels: *mut std::ffi::c_void,
            closure_ptr: *mut std::ffi::c_void,
        ) -> sys::ktx_error_code_e
        where
            F: FnMut(i32, i32, i32, i32, i32, &mut [u8]) -> Result<(), KtxError>,
        {
            let closure = closure_ptr as *mut F;
            let pixels_slice =
                std::slice::from_raw_parts_mut(pixels as *mut u8, pixels_size as usize);
            match (*closure)(mip, face, width, height, depth, pixels_slice) {
                Ok(_) => sys::ktx_error_code_e_KTX_SUCCESS,
                Err(code) => code as u32,
            }
        }

        // SAFETY: Safe if `self.handle` is sane.
        unsafe {
            if (*self.handle).pData.is_null() {
                // Data was not loaded
                return Err(KtxError::InvalidValue);
            }

            let vtbl = (*self.handle).vtbl;
            if let Some(iterate_levels_fn) = (*vtbl).IterateLevels {
                let closure_ptr = &mut callback as *mut F as *mut std::ffi::c_void;
                let err = (iterate_levels_fn)(self.handle, Some(c_iterator_fn::<F>), closure_ptr);
                ktx_result(err, ())
            } else {
                Err(KtxError::InvalidValue)
            }
        }
    }

    /// If this [`Texture`] really is a KTX1, returns KTX1-specific functionalities for it.
    pub fn ktx1<'b>(&'b mut self) -> Option<Ktx1<'b, 'a>> {
        // SAFETY: Safe if `self.handle` is sane.
        if unsafe { &*self.handle }.classId == sys::class_id_ktxTexture1_c {
            Some(Ktx1 { texture: self })
        } else {
            None
        }
    }

    /// If this [`Texture`] really is a KTX2, returns KTX2-specific functionalities for it.
    pub fn ktx2<'b>(&'b mut self) -> Option<Ktx2<'b, 'a>> {
        // SAFETY: Safe if `self.handle` is sane.
        if unsafe { &*self.handle }.classId == sys::class_id_ktxTexture2_c {
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

/// KTX1-specific [`Texture`] functionality.
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

    /// Returns the OpenGL format of the texture's data (e.g. `GL_RGBA`).
    ///
    /// Also see [`Self::gl_internal_format`], [`Self::gl_base_internal_format`].
    pub fn gl_format(&self) -> u32 {
        let handle = self.handle();
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX1
        unsafe { (*handle).glFormat }
    }

    /// Returns the OpenGL format of the texture's data (e.g. `GL_RGBA`).
    ///
    /// Also see [`Self::gl_format`], [`Self::gl_base_internal_format`].
    pub fn gl_internal_format(&self) -> u32 {
        let handle = self.handle();
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX1
        unsafe { (*handle).glFormat }
    }

    /// Returns the OpenGL base internal format of the texture's data (e.g. `GL_RGBA`).
    ///
    /// Also see [`Self::gl_format`], [`Self::gl_internal_format`].
    pub fn gl_base_internal_format(&self) -> u32 {
        let handle = self.handle();
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX1
        unsafe { (*handle).glBaseInternalformat }
    }

    /// Returns the OpenGL datatype of the texture's data (e.g. `GL_UNSIGNED_BYTE`).
    pub fn gl_type(&self) -> u32 {
        let handle = self.handle();
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX1
        unsafe { (*handle).glType }
    }

    /// Will this KTX1 need transcoding?
    pub fn needs_transcoding(&self) -> bool {
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX1
        unsafe { sys::ktxTexture1_NeedsTranscoding(self.handle()) }
    }

    // TODO: WriteKTX2ToStream with a Rust stream (and to a memory slice?)
    //       Probably needs a TextureSink trait
}

/// KTX2-specific [`Texture`] functionality.
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

    /// Returns the Vulkan format of the texture's data (e.g. `VK_R8G8B8A8_UNORM`).
    pub fn vk_format(&self) -> u32 {
        let handle = self.handle();
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX1
        unsafe { (*handle).vkFormat }
    }

    /// Returns the supercompression scheme in use for this texture's data.
    pub fn supercompression_scheme(&self) -> SuperCompressionScheme {
        let handle = self.handle();
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX1
        unsafe { (*handle).supercompressionScheme.into() }
    }

    /// Is this a video texture?
    pub fn is_video(&self) -> bool {
        let handle = self.handle();
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX1
        unsafe { (*handle).isVideo }
    }

    /// Returns the duration of the video texture (if [`Self::is_video`]).
    pub fn duration(&self) -> u32 {
        let handle = self.handle();
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX1
        unsafe { (*handle).duration }
    }

    /// Returns the timescale of the video texture (if [`Self::is_video`]).
    pub fn timescale(&self) -> u32 {
        let handle = self.handle();
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX1
        unsafe { (*handle).timescale }
    }

    /// Returns the loop count of the video texture (if [`Self::is_video`]).
    pub fn loop_count(&self) -> u32 {
        let handle = self.handle();
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX1
        unsafe { (*handle).loopcount }
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

    /// Compresses the KTX2's image data with ASTC.  
    /// This is a simplified version of [`Ktx2::compress_astc_ex`].
    pub fn compress_astc(&mut self, quality: u32) -> Result<(), KtxError> {
        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX2
        let errcode = unsafe { sys::ktxTexture2_CompressAstc(self.handle(), quality) };
        ktx_result(errcode, ())
    }

    /// Compresses the KTX2's image data with ASTC.   
    /// This is an extended version of [`Ktx2::compress_astc`].
    pub fn compress_astc_ex(&mut self, params: AstcParams) -> Result<(), KtxError> {
        let mut c_input_swizzle: [std::os::raw::c_char; 4] = [0, 0, 0, 0];
        for (ch, c_ch) in params.input_swizzle.iter().zip(c_input_swizzle.iter_mut()) {
            *c_ch = *ch as _;
        }
        let mut c_params = sys::ktxAstcParams {
            structSize: std::mem::size_of::<sys::ktxAstcParams>() as u32,
            verbose: params.verbose,
            threadCount: params.thread_count,
            blockDimension: params.block_dimension as u32,
            function: params.function as u32,
            mode: params.mode as u32,
            qualityLevel: params.quality_level as u32,
            normalMap: params.normal_map,
            inputSwizzle: c_input_swizzle,
        };

        // SAFETY: Safe if `self.texture.handle` is sane + actually a KTX2
        let errcode = unsafe { sys::ktxTexture2_CompressAstcEx(self.handle(), &mut c_params) };
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
    /// **This may differ from values returned by [`Self::component_info`]:**
    /// - For uncompressed formats: this is the number of image components, as from [`Self::component_info`].
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
