use libktx_rs::{Format, Texture, TextureCreateInfo};

#[test]
fn create_default_ktx1() {
    Texture::create(
        TextureCreateInfo {
            format: Format::Gl(0x8058), // GL_RGBA8
            ..TextureCreateInfo::default()
        },
        libktx_rs::CreateStorage::AllocStorage,
    )
    .expect("a default KTX2 texture");
}

#[test]
fn create_default_ktx2() {
    Texture::create(
        TextureCreateInfo::default(),
        libktx_rs::CreateStorage::AllocStorage,
    )
    .expect("a default KTX2 texture");
}
