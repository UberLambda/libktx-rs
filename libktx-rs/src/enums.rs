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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum CreateStorage {
    NoStorage = sys::ktxTextureCreateStorageEnum_KTX_TEXTURE_CREATE_NO_STORAGE,
    AllocStorage = sys::ktxTextureCreateStorageEnum_KTX_TEXTURE_CREATE_ALLOC_STORAGE,
}

bitflags! {
    #[derive(Default)]
    pub struct TextureCreateFlags: u32 {
        const LOAD_IMAGE_DATA = sys::ktxTextureCreateFlagBits_KTX_TEXTURE_CREATE_LOAD_IMAGE_DATA_BIT;
        const RAW_KVDATA = sys::ktxTextureCreateFlagBits_KTX_TEXTURE_CREATE_RAW_KVDATA_BIT;
        const SKIP_KVDATA = sys::ktxTextureCreateFlagBits_KTX_TEXTURE_CREATE_SKIP_KVDATA_BIT;
    }
}

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
