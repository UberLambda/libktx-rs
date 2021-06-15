// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "test-images")]
mod test_images {
    use libktx_rs::{
        enums::TranscodeFormat, sources::StreamSource, RustKtxStream, Texture, TextureCreateFlags,
        TranscodeFlags,
    };
    use libktx_rs_macros::file_tests;
    use std::{
        fs::File,
        path::PathBuf,
        sync::{Arc, Mutex},
    };

    fn from_stream(_path: PathBuf, file: File) {
        let stream = RustKtxStream::new(Box::new(file)).expect("the Rust ktxStream");
        let source = StreamSource::new(
            Arc::new(Mutex::new(stream)),
            TextureCreateFlags::LOAD_IMAGE_DATA,
        );
        let mut stream_texture = Texture::new(source).expect("the loaded KTX");

        if let Some(_) = stream_texture.ktx1() {
            println!("Texture is KTX1");
        } else if let Some(_) = stream_texture.ktx2() {
            println!("Texture is KTX2");
        } else {
            panic!("The loaded texture should be either KTX1 or KTX2!");
        }

        dbg!(
            stream_texture.data_size(),
            stream_texture.element_size(),
            stream_texture.row_pitch(0)
        );

        if let Some(mut ktx2) = stream_texture.ktx2() {
            if ktx2.needs_transcoding() {
                println!("This KTX2 needs transcoding");
                ktx2.transcode_basis(TranscodeFormat::Rgba32, TranscodeFlags::empty())
                    .expect("transcoding to work");
            }
        }

        stream_texture
            .iterate_levels(|mip, face, width, height, depth, pixel_data| {
                dbg!(mip, face, width, height, depth, pixel_data.len());
                Ok(())
            })
            .expect("mip/face read-only iteration to succeed");

        stream_texture
            .iterate_levels_mut(|_mip, _face, _width, _height, _depth, pixel_data| {
                pixel_data.fill(0x42u8);
                Ok(())
            })
            .expect("mip/face read-write iteration to succeed");
    }

    // FIXME: These glob patterns assume that `cargo build` is invoked from the root of the workspace!
    file_tests! {from_stream =>
        "libktx-rs-sys/build/KTX-Software/tests/testimages/*.ktx*",
        // This one has a unsupported image type, skip
        !"libktx-rs-sys/build/KTX-Software/tests/testimages/luminance-reference-metadata.ktx",
    }
}
