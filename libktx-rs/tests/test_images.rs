#[cfg(feature = "test-images")]
mod test_images {
    use libktx_rs::{self as ktx};
    use libktx_rs_macros::file_tests;
    use std::fs::File;

    fn from_stream(file: File) {
        let stream_texture = StreamTexture::create(
            Box::new(file),
            sys::ktxTextureCreateFlagBits_KTX_TEXTURE_CREATE_LOAD_IMAGE_DATA_BIT,
        );
        stream_texture.expect("the loaded KTX");
    }

    // FIXME: These glob pattersn assume that `cargo build` is invoked from the root of the workspace!
    file_tests! {from_stream =>
        "libktx-rs-sys/build/KTX-Software/tests/testimages/*.ktx?",
    }
}
