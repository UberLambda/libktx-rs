// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "test-images")]
mod test_images {
    use libktx_rs::{sources::StreamSource, RustKtxStream, Texture, TextureCreateFlags};
    use libktx_rs_macros::file_tests;
    use std::fs::File;

    fn from_stream(file: File) {
        let stream = RustKtxStream::new(Box::new(file)).expect("the Rust ktxStream");
        let source = StreamSource {
            stream,
            texture_create_flags: TextureCreateFlags::LOAD_IMAGE_DATA,
        };
        let stream_texture = Texture::new(source).expect("the loaded KTX");

        println!(
            "Data size: {}, element size: {}, row pitch: {}",
            stream_texture.data_size(),
            stream_texture.element_size(),
            stream_texture.row_pitch(0)
        );
    }

    // FIXME: These glob patterns assume that `cargo build` is invoked from the root of the workspace!
    file_tests! {from_stream =>
        "libktx-rs-sys/build/KTX-Software/tests/testimages/*.ktx?",
    }
}
