use libktx_rs::{Ktx1CreateInfo, Ktx2CreateInfo, Texture};

#[test]
fn create_default_ktx1() {
    Texture::new(Ktx1CreateInfo::default()).expect("a default KTX1 texture");
}

#[test]
fn create_default_ktx2() {
    Texture::new(Ktx2CreateInfo::default()).expect("a default KTX2 texture");
}
