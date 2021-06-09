#[cfg(feature = "test-images")]
mod test_images {
    use libktx_rs::{RustKtxStream, StreamSource, Texture, TextureCreateFlags};
    use libktx_rs_macros::file_tests;
    use std::fs::File;

    fn from_stream(file: File) {
        let stream = RustKtxStream::new(Box::new(file)).expect("the Rust ktxStream");
        let source = StreamSource {
            stream,
            texture_create_flags: TextureCreateFlags::LOAD_IMAGE_DATA,
        };
        let stream_texture = Texture::new(source);
        stream_texture.expect("the loaded KTX");
    }

    // FIXME: These glob patterns assume that `cargo build` is invoked from the root of the workspace!
    file_tests! {from_stream =>
        "libktx-rs-sys/build/KTX-Software/tests/testimages/*.ktx?",
    }
}
