**Archived**: put in the backlog for now...

* * * * *

# libktx-rs

[![crates.io](https://img.shields.io/crates/v/libktx-rs.svg)](https://crates.io/crates/libktx-rs)
[![docs.rs](https://docs.rs/libktx-rs/badge.svg)](https://docs.rs/libktx-rs)
[![license](https://img.shields.io/github/license/UberLambda/libktx-rs)](LICENSE)
[![CI status](https://github.com/UberLambda/libktx-rs/actions/workflows/push.yml/badge.svg)](https://github.com/UberLambda/libktx-rs/actions/workflows/push.yml)

A library for reading, writing and transcoding [Khronos Textures](https://www.khronos.org/ktx/) (KTX1 and KTX2) in Rust.

**This repository contains both [high-level Rust bindings](libktx-rs/) and
[low-level FFI](libktx-rs-sys/) to [KhronosGroup/KTX-Software](https://github.com/KhronosGroup/KTX-Software)**.

## Structure
- [libktx-rs](libktx-rs/) contains the high-level Rust wrapper.
- [libktx-rs-sys](libktx-rs-sys/) contains the low-level C FFI, and it builds KTX-Software from source.
- [libktx-rs-macros](libktx-rs-macros/) contains helpers for testing.

## Docs
See <https://docs.rs/libktx-rs> for the latest documentation of the high-level API,
and <https://docs.rs/libktx-rs-sys> for the low-level FFI.

## Building and features
Clone this root repository and all git submodule (`git clone --recursive https://github.com/UberLambda/libktx-rs`), then run `cargo build`.

### Image writing
To enable KTX image writing support (which is already enabled in the default feature set), enable the `libktx-rs/write` feature.

### Image-based tests
To enable image loading tests, **clone the libktx-rs-sys/KTX-Software submodule with git LFS support**, then enable the `libktx-rs-sys/test-images` feature.

### rust-bindgen at build time
To have [rust-bindgen](https://github.com/rust-lang/rust-bindgen) generate bindings in the build script (instead of using [the pre-generated ones](libktx-rs-sys/src/ffi.rs)),
enable the `libktx-rs-sys/run-bindgen` feature.

### Docs-only
To skip building or linking KTX-Software altogether, enable the `libktx-rs-sys/docs-only` feature.

## License
This Rust wrapper, and the KTX-Software library itself, are both licensed under the [Apache-2.0 license](LICENSE).

### Linux and GCC
Note that the library links to libstdc++, which is licensed under [LGPL with the "Runtime Library Exception"](https://gcc.gnu.org/onlinedocs/libstdc++/manual/license.html).

### License exception
**If the ETC decoder is enabled, the build will contain a proprietary source code file by Ericsson - [KTX-Software/lib/etcdec.cxx](https://github.com/KhronosGroup/KTX-Software/blob/master/lib/etcdec.cxx)!**  
Building this file is optional, and it is disabled by default.
Build libktx-rs-sys with the `nonfree-etc-unpack` to enable this feature if you agree with the terms of the license.

See [the original LICENSE](https://github.com/KhronosGroup/KTX-Software/blob/63d9e76b90d00703e7c097ad936f1725ecc0e505/LICENSE.md) for more information.
