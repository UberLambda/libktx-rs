// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0

use crate::sys;
use bitflags::bitflags;
use std::{
    convert::TryFrom,
    error::Error,
    ffi::CStr,
    fmt::{Display, Formatter},
};

/// Error codes as returned from the underlying C library.
///
/// See [`sys::ktx_error_code_e`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum KtxError {
    FileDataError = sys::ktx_error_code_e_KTX_FILE_DATA_ERROR,
    FileIsPipe = sys::ktx_error_code_e_KTX_FILE_ISPIPE,
    FileOpenFailed = sys::ktx_error_code_e_KTX_FILE_OPEN_FAILED,
    FileOverflow = sys::ktx_error_code_e_KTX_FILE_OVERFLOW,
    FileReadError = sys::ktx_error_code_e_KTX_FILE_READ_ERROR,
    FileSeekError = sys::ktx_error_code_e_KTX_FILE_SEEK_ERROR,
    FileUnexpectedEof = sys::ktx_error_code_e_KTX_FILE_UNEXPECTED_EOF,
    FileWriteError = sys::ktx_error_code_e_KTX_FILE_WRITE_ERROR,
    GlError = sys::ktx_error_code_e_KTX_GL_ERROR,
    InvalidOperation = sys::ktx_error_code_e_KTX_INVALID_OPERATION,
    InvalidValue = sys::ktx_error_code_e_KTX_INVALID_VALUE,
    NotFound = sys::ktx_error_code_e_KTX_NOT_FOUND,
    OutOfMemory = sys::ktx_error_code_e_KTX_OUT_OF_MEMORY,
    TranscodeFailed = sys::ktx_error_code_e_KTX_TRANSCODE_FAILED,
    UnknownFileFormat = sys::ktx_error_code_e_KTX_UNKNOWN_FILE_FORMAT,
    UnsupportedTextureType = sys::ktx_error_code_e_KTX_UNSUPPORTED_TEXTURE_TYPE,
    UnsupportedFeature = sys::ktx_error_code_e_KTX_UNSUPPORTED_FEATURE,
    LibraryNotLinked = sys::ktx_error_code_e_KTX_LIBRARY_NOT_LINKED,
}

impl TryFrom<u32> for KtxError {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        // TODO: A bit ugly (but still manageable), convert to a macro?
        Ok(match value {
            sys::ktx_error_code_e_KTX_FILE_DATA_ERROR => Self::FileDataError,
            sys::ktx_error_code_e_KTX_FILE_ISPIPE => Self::FileIsPipe,
            sys::ktx_error_code_e_KTX_FILE_OPEN_FAILED => Self::FileOpenFailed,
            sys::ktx_error_code_e_KTX_FILE_OVERFLOW => Self::FileOverflow,
            sys::ktx_error_code_e_KTX_FILE_READ_ERROR => Self::FileReadError,
            sys::ktx_error_code_e_KTX_FILE_SEEK_ERROR => Self::FileSeekError,
            sys::ktx_error_code_e_KTX_FILE_UNEXPECTED_EOF => Self::FileUnexpectedEof,
            sys::ktx_error_code_e_KTX_FILE_WRITE_ERROR => Self::FileWriteError,
            sys::ktx_error_code_e_KTX_GL_ERROR => Self::GlError,
            sys::ktx_error_code_e_KTX_INVALID_OPERATION => Self::InvalidOperation,
            sys::ktx_error_code_e_KTX_INVALID_VALUE => Self::InvalidValue,
            sys::ktx_error_code_e_KTX_NOT_FOUND => Self::NotFound,
            sys::ktx_error_code_e_KTX_OUT_OF_MEMORY => Self::OutOfMemory,
            sys::ktx_error_code_e_KTX_TRANSCODE_FAILED => Self::TranscodeFailed,
            sys::ktx_error_code_e_KTX_UNKNOWN_FILE_FORMAT => Self::UnknownFileFormat,
            sys::ktx_error_code_e_KTX_UNSUPPORTED_TEXTURE_TYPE => Self::UnsupportedTextureType,
            sys::ktx_error_code_e_KTX_UNSUPPORTED_FEATURE => Self::UnsupportedFeature,
            sys::ktx_error_code_e_KTX_LIBRARY_NOT_LINKED => Self::LibraryNotLinked,
            _ => return Err("Not a KTX_ error variant"),
        })
    }
}

impl Display for KtxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // SAFETY: Safe - this just accessess a C array of strings under the hood
        let c_str = unsafe { CStr::from_ptr(sys::ktxErrorString(*self as u32)) };
        match c_str.to_str() {
            Ok(msg) => write!(f, "{}", msg),
            _ => Err(std::fmt::Error),
        }
    }
}

impl Error for KtxError {}

pub(crate) fn ktx_result<T>(errcode: sys::ktx_error_code_e, ok: T) -> Result<T, KtxError> {
    if errcode == sys::ktx_error_code_e_KTX_SUCCESS {
        Ok(ok)
    } else {
        Err(KtxError::try_from(errcode as u32).unwrap_or(KtxError::InvalidValue))
    }
}

/// The supercompression scheme for a [`Texture`].
///
/// See [`sys::ktxSupercmpScheme`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SuperCompressionScheme {
    None,
    BasisLZ,
    ZStd,
    Vendor(u32),
}

impl Default for SuperCompressionScheme {
    fn default() -> Self {
        SuperCompressionScheme::None
    }
}

impl From<SuperCompressionScheme> for u32 {
    fn from(scheme: SuperCompressionScheme) -> Self {
        match scheme {
            SuperCompressionScheme::None => sys::ktxSupercmpScheme_KTX_SS_NONE,
            SuperCompressionScheme::BasisLZ => sys::ktxSupercmpScheme_KTX_SUPERCOMPRESSION_BASIS,
            SuperCompressionScheme::ZStd => sys::ktxSupercmpScheme_KTX_SUPERCOMPRESSION_ZSTD,
            SuperCompressionScheme::Vendor(value) => value,
        }
    }
}

impl Display for SuperCompressionScheme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // SAFETY: Safe - this is a C switch/case under the hood, with invalid value checking
        let c_str = unsafe { CStr::from_ptr(sys::ktxSupercompressionSchemeString((*self).into())) };
        match c_str.to_str() {
            Ok(msg) => write!(f, "{}", msg),
            _ => Err(std::fmt::Error),
        }
    }
}

/// [`Texture`] storage creation flags.
///
/// See [`sys::ktxTextureCreateStorageEnum`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum CreateStorage {
    NoStorage = sys::ktxTextureCreateStorageEnum_KTX_TEXTURE_CREATE_NO_STORAGE,
    AllocStorage = sys::ktxTextureCreateStorageEnum_KTX_TEXTURE_CREATE_ALLOC_STORAGE,
}

bitflags! {
    /// [`Texture`] creation flags.
    ///
    /// See [`sys::ktxTextureCreateFlags`].
    #[derive(Default)]
    pub struct TextureCreateFlags: u32 {
        const LOAD_IMAGE_DATA = sys::ktxTextureCreateFlagBits_KTX_TEXTURE_CREATE_LOAD_IMAGE_DATA_BIT;
        const RAW_KVDATA = sys::ktxTextureCreateFlagBits_KTX_TEXTURE_CREATE_RAW_KVDATA_BIT;
        const SKIP_KVDATA = sys::ktxTextureCreateFlagBits_KTX_TEXTURE_CREATE_SKIP_KVDATA_BIT;
    }
}

bitflags! {
    pub struct PackUastcFlags: u32 {
        const LEVEL_FASTEST  = sys::ktx_pack_uastc_flag_bits_e_KTX_PACK_UASTC_LEVEL_FASTEST;
        const LEVEL_FASTER   = sys::ktx_pack_uastc_flag_bits_e_KTX_PACK_UASTC_LEVEL_FASTER;
        const LEVEL_DEFAULT  = sys::ktx_pack_uastc_flag_bits_e_KTX_PACK_UASTC_LEVEL_DEFAULT;
        const LEVEL_SLOWER   = sys::ktx_pack_uastc_flag_bits_e_KTX_PACK_UASTC_LEVEL_SLOWER;
        const LEVEL_VERYSLOW = sys::ktx_pack_uastc_flag_bits_e_KTX_PACK_UASTC_LEVEL_VERYSLOW;
        const LEVEL_MASK     = sys::ktx_pack_uastc_flag_bits_e_KTX_PACK_UASTC_LEVEL_MASK;
        const FAVOR_UASTC_ERROR = sys::ktx_pack_uastc_flag_bits_e_KTX_PACK_UASTC_FAVOR_UASTC_ERROR;
        const FAVOR_BC7_ERROR = sys::ktx_pack_uastc_flag_bits_e_KTX_PACK_UASTC_FAVOR_BC7_ERROR;
        const ETC1_FASTER_HINTS = sys::ktx_pack_uastc_flag_bits_e_KTX_PACK_UASTC_ETC1_FASTER_HINTS;
        const ETC1_FASTEST_HINTS = sys::ktx_pack_uastc_flag_bits_e_KTX_PACK_UASTC_ETC1_FASTEST_HINTS;
        const _ETC1_DISABLE_FLIP_AND_INDIVIDUAL = sys::ktx_pack_uastc_flag_bits_e_KTX_PACK_UASTC__ETC1_DISABLE_FLIP_AND_INDIVIDUAL;
}
}

/// The destination format for transcoding a [`texture::Ktx2`] via Basis Universal.
///
/// See [`sys::ktx_transcode_fmt_e`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum TranscodeFormat {
    // ETC
    Etc1Rgb = sys::ktx_transcode_fmt_e_KTX_TTF_ETC1_RGB,
    Etc2Rgba = sys::ktx_transcode_fmt_e_KTX_TTF_ETC2_RGBA,
    // BC
    Bc1Rgb = sys::ktx_transcode_fmt_e_KTX_TTF_BC1_RGB,
    Bc3Rgba = sys::ktx_transcode_fmt_e_KTX_TTF_BC3_RGBA,
    Bc3R = sys::ktx_transcode_fmt_e_KTX_TTF_BC4_R,
    Bc5Rg = sys::ktx_transcode_fmt_e_KTX_TTF_BC5_RG,
    Bc7Rgba = sys::ktx_transcode_fmt_e_KTX_TTF_BC7_RGBA,
    // PVRTC 1
    Pvrtc14Rgb = sys::ktx_transcode_fmt_e_KTX_TTF_PVRTC1_4_RGB,
    Pvrtc14Rgba = sys::ktx_transcode_fmt_e_KTX_TTF_PVRTC1_4_RGBA,
    // ASTC
    Astc4x4Rgba = sys::ktx_transcode_fmt_e_KTX_TTF_ASTC_4x4_RGBA,
    // PVRTC 2
    Pvrtc24Rgb = sys::ktx_transcode_fmt_e_KTX_TTF_PVRTC2_4_RGB,
    Pvrtc24Rgba = sys::ktx_transcode_fmt_e_KTX_TTF_PVRTC2_4_RGBA,
    // EAC
    Etc2EacR11 = sys::ktx_transcode_fmt_e_KTX_TTF_ETC2_EAC_R11,
    Etc2EacRg11 = sys::ktx_transcode_fmt_e_KTX_TTF_ETC2_EAC_RG11,
    // Uncompressed (raw)
    Rgba32 = sys::ktx_transcode_fmt_e_KTX_TTF_RGBA32,
    Rgb565 = sys::ktx_transcode_fmt_e_KTX_TTF_RGB565,
    Bgr565 = sys::ktx_transcode_fmt_e_KTX_TTF_BGR565,
    Rgba4444 = sys::ktx_transcode_fmt_e_KTX_TTF_RGBA4444,
    // Automatic selection
    Etc = sys::ktx_transcode_fmt_e_KTX_TTF_ETC,
    Bc1or3 = sys::ktx_transcode_fmt_e_KTX_TTF_BC1_OR_3,
    // Misc.
    NoSelection = sys::ktx_transcode_fmt_e_KTX_TTF_NOSELECTION,
}

impl TryFrom<u32> for TranscodeFormat {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        // TODO: A bit ugly (but still manageable), convert to a macro?
        Ok(match value {
            sys::ktx_transcode_fmt_e_KTX_TTF_ETC1_RGB => Self::Etc1Rgb,
            sys::ktx_transcode_fmt_e_KTX_TTF_ETC2_RGBA => Self::Etc2Rgba,
            // BC
            sys::ktx_transcode_fmt_e_KTX_TTF_BC1_RGB => Self::Bc1Rgb,
            sys::ktx_transcode_fmt_e_KTX_TTF_BC3_RGBA => Self::Bc3Rgba,
            sys::ktx_transcode_fmt_e_KTX_TTF_BC4_R => Self::Bc3R,
            sys::ktx_transcode_fmt_e_KTX_TTF_BC5_RG => Self::Bc5Rg,
            sys::ktx_transcode_fmt_e_KTX_TTF_BC7_RGBA => Self::Bc7Rgba,
            // PVRTC 1
            sys::ktx_transcode_fmt_e_KTX_TTF_PVRTC1_4_RGB => Self::Pvrtc14Rgb,
            sys::ktx_transcode_fmt_e_KTX_TTF_PVRTC1_4_RGBA => Self::Pvrtc14Rgba,
            // ASTC
            sys::ktx_transcode_fmt_e_KTX_TTF_ASTC_4x4_RGBA => Self::Astc4x4Rgba,
            // PVRTC 2
            sys::ktx_transcode_fmt_e_KTX_TTF_PVRTC2_4_RGB => Self::Pvrtc24Rgb,
            sys::ktx_transcode_fmt_e_KTX_TTF_PVRTC2_4_RGBA => Self::Pvrtc24Rgba,
            // EAC
            sys::ktx_transcode_fmt_e_KTX_TTF_ETC2_EAC_R11 => Self::Etc2EacR11,
            sys::ktx_transcode_fmt_e_KTX_TTF_ETC2_EAC_RG11 => Self::Etc2EacRg11,
            // Uncompressed (raw)
            sys::ktx_transcode_fmt_e_KTX_TTF_RGBA32 => Self::Rgba32,
            sys::ktx_transcode_fmt_e_KTX_TTF_RGB565 => Self::Rgb565,
            sys::ktx_transcode_fmt_e_KTX_TTF_BGR565 => Self::Bgr565,
            sys::ktx_transcode_fmt_e_KTX_TTF_RGBA4444 => Self::Rgba4444,
            // Automatic selection
            sys::ktx_transcode_fmt_e_KTX_TTF_ETC => Self::Etc,
            sys::ktx_transcode_fmt_e_KTX_TTF_BC1_OR_3 => Self::Bc1or3,
            // Misc.
            sys::ktx_transcode_fmt_e_KTX_TTF_NOSELECTION => Self::NoSelection,
            _ => return Err("Not a KTX_ error variant"),
        })
    }
}

bitflags! {
    /// Flags applied when transcoding a [`texture::Ktx2`] via Basis Universal.
    ///
    /// See [`sys::ktx_transcode_flags`].
    #[derive(Default)]
    pub struct TranscodeFlags: u32 {
        const PVRTC_DECODE_TO_NEXT_POW2 = sys::ktx_transcode_flag_bits_e_KTX_TF_PVRTC_DECODE_TO_NEXT_POW2;
        const TRANSCODE_ALPHA_DATA_TO_OPAQUE_FORMATS = sys::ktx_transcode_flag_bits_e_KTX_TF_TRANSCODE_ALPHA_DATA_TO_OPAQUE_FORMATS;
        const HIGH_QUALITY = sys::ktx_transcode_flag_bits_e_KTX_TF_HIGH_QUALITY;
    }
}
