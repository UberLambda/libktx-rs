// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0

use libktx_rs::{
    sources::{Ktx1CreateInfo, Ktx2CreateInfo},
    Texture,
};

#[test]
fn create_default_ktx1() {
    let texture = Texture::new(Ktx1CreateInfo::default()).expect("a default KTX1 texture");

    // 1x1 RGBA8 texel
    assert_eq!(texture.element_size(), 4);
    assert_eq!(texture.row_pitch(0), 4);
    assert_eq!(texture.data_size(), 4);
}

#[test]
fn create_default_ktx2() {
    let texture = Texture::new(Ktx2CreateInfo::default()).expect("a default KTX2 texture");

    // 1x1 RGBA8 texel
    assert_eq!(texture.element_size(), 4);
    assert_eq!(texture.row_pitch(0), 4);
    assert_eq!(texture.data_size(), 4);
}
