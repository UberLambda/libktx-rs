// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0

//! A high-level Rust wrapper over [KhronosGroup/KTX-Software](https://github.com/KhronosGroup/KTX-Software),
//! a library for reading, transcoding and writing [Khronos Textures (KTX)](https://www.khronos.org/ktx/).

pub use libktx_rs_sys as sys;

pub mod enums;
pub use enums::*;

pub mod texture;
pub use texture::{Texture, TextureSource};

pub mod stream;
pub use stream::{RWSeekable, RustKtxStream};

#[cfg(feature = "write")]
pub mod sinks;
pub mod sources;
