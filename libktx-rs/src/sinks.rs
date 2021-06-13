// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    enums::ktx_result,
    sys::stream::RustKtxStream,
    texture::{Texture, TextureSink},
    KtxError,
};

#[derive(Debug)]
pub struct StreamSink<'a> {
    pub sink: RustKtxStream<'a>,
}

impl<'a> TextureSink for StreamSink<'a> {
    fn write_texture(&mut self, texture: &Texture) -> Result<(), KtxError> {
        // SAFETY: Safe if `texture.handle` is sound.
        let vtbl = unsafe { (*texture.handle).vtbl };
        let write_pfn = match unsafe { (*vtbl).WriteToStream } {
            Some(pfn) => pfn,
            None => return Err(KtxError::InvalidValue),
        };
        let err = unsafe { write_pfn(texture.handle, self.sink.ktx_stream()) };
        ktx_result(err, ())
    }
}
