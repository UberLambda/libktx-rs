// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0
#![cfg(feature = "write")]

//! [`crate::texture::TextureSink`] implementations for writing [`Texture`]s out to.

use crate::{
    enums::ktx_result,
    stream::{RWSeekable, RustKtxStream},
    texture::{Texture, TextureSink},
    KtxError,
};
use std::sync::{Arc, Mutex};

/// A [`TextureSink`] that writes to a [`RustKtxStream`].
#[derive(Debug)]
pub struct StreamSink<'a, T: RWSeekable + ?Sized + 'a> {
    stream: Arc<Mutex<RustKtxStream<'a, T>>>,
}

impl<'a, T: RWSeekable + ?Sized + 'a> StreamSink<'a, T> {
    /// Creates a new stream sink that will write to the given `inner` stream.
    pub fn new(inner: Arc<Mutex<RustKtxStream<'a, T>>>) -> Self {
        StreamSink { stream: inner }
    }

    /// Destroys this stream sink, giving back the underlying `inner` stream.
    pub fn into_inner(self) -> Arc<Mutex<RustKtxStream<'a, T>>> {
        self.stream
    }
}

impl<'a, T: RWSeekable + ?Sized + 'a> TextureSink for StreamSink<'a, T> {
    fn write_texture(&mut self, texture: &Texture) -> Result<(), KtxError> {
        // SAFETY: Safe if `texture.handle` is sound.
        let vtbl = unsafe { (*texture.handle).vtbl };
        let write_pfn = match unsafe { (*vtbl).WriteToStream } {
            Some(pfn) => pfn,
            None => return Err(KtxError::InvalidValue),
        };
        let err = unsafe {
            write_pfn(
                texture.handle,
                self.stream
                    .lock()
                    .expect("Poisoned self.stream lock")
                    .ktx_stream(),
            )
        };
        ktx_result(err, ())
    }
}
