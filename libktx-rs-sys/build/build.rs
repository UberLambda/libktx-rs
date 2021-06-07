#[cfg(feature = "run-bindgen")]
use bindgen;
use cc;
use std::path::PathBuf;

// All of these values are taken from `KTX-Software/CMakeLists.txt`
// SPDX-License-Identifier: Apache-2.0

const SOURCE_DIR: &str = "build/KTX-Software";

const INCLUDE_DIRS: &[&str] = &[
    "build/KTX-Software/include",
    "build/KTX-Software/lib",
    "build/KTX-Software/lib/basisu/transcoder",
    "build/KTX-Software/lib/basisu/zstd",
    "build/KTX-Software/other_include",
    "build/KTX-Software/utils",
];

#[cfg(feature = "run-bindgen")]
const MAIN_HEADER: &str = "build/KTX-Software/include/ktx.h";

const C_SOURCE_FILES: &[&str] = &[
    "lib/basisu/zstd/zstd.c",
    "lib/checkheader.c",
    "lib/dfdutils/createdfd.c",
    "lib/dfdutils/colourspaces.c",
    "lib/dfdutils/interpretdfd.c",
    "lib/dfdutils/printdfd.c",
    "lib/dfdutils/queries.c",
    //"lib/dfdutils/dfd2vk.inl",
    "lib/dfdutils/dfd2vk.c",
    //"lib/dfdutils/vk2dfd.inl",
    "lib/dfdutils/vk2dfd.c",
    "lib/filestream.c",
    "lib/hashlist.c",
    "lib/info.c",
    "lib/memstream.c",
    "lib/strings.c",
    "lib/swap.c",
    "lib/texture.c",
    "lib/texture2.c",
    "lib/vkformat_check.c",
    "lib/vkformat_str.c",
    // KTX_FEATURE_KTX1
    "lib/texture1.c",
    // KTX_FEATURE_VULKAN (?)
];

const CXX_SOURCE_FILES: &[&str] = &[
    "lib/basis_transcode.cpp",
    "lib/basisu/transcoder/basisu_transcoder.cpp",
    "lib/etcdec.cxx",
    "lib/etcunpack.cxx",
];

fn configure_build(mut build: cc::Build) -> cc::Build {
    build
        .includes(INCLUDE_DIRS)
        .warnings(false)
        .extra_warnings(false)
        //
        .define("LIBKTX", "1") // This one is important (compilation fails otherwise!)
        .define("BASISD_SUPPORT_FXT1", "0")
        .define("BASISD_SUPPORT_KTX2_ZSTD", "0") // ZSTD support is added by libktx itself
        .define("KTX_FEATURE_KTX1", "1")
        .define("KTX_FEATURE_KTX2", "1")
        .define("KTX_FEATURE_WRITE", "0")
        .define("KTX_FEATURE_GL_UPLOAD", "0")
        .define("KTX_FEATURE_VULKAN", "0")
        .define("KTX_OMIT_VULKAN", "1");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    match (target_os.as_str(), target_arch.as_str()) {
        ("windows", _) => {
            build
                .file("lib/internalexport.def")
                .file("lib/internalexport_write.def")
                .define("KTX_API", "__declspec(dllexport)")
                .define("BASISU_NO_ITERATOR_DEBUG_LEVEL", "1");
        }
        ("linux", _) => {
            build.flag("-pthread").flag("-ldl");
        }
        (_, "wasm32" | "wasm64") => {
            build
                .define("BASISD_SUPPORT_ATC", "0")
                .define("BASISD_SUPPORT_PVRTC2", "0")
                .define("BASISD_SUPPORT_ASTC_HIGHER_OPAQUE_QUALITY", "0");
        }
        _ => (),
    }

    build
}

fn ktx_sources<'a>(rel_paths: &'a [&'a str]) -> impl Iterator<Item = PathBuf> + 'a {
    rel_paths
        .iter()
        .map(|f| [SOURCE_DIR, f].iter().collect::<PathBuf>())
}

fn main() {
    println!("-- Build the native libKTX");

    configure_build(cc::Build::new())
        .cpp(false)
        .files(ktx_sources(C_SOURCE_FILES))
        .compile("ktx_c");

    let mut cxx_build = configure_build(cc::Build::new());
    cxx_build.cpp(true).files(ktx_sources(CXX_SOURCE_FILES));
    cxx_build.compile("ktx");

    println!("-- Link the native libKTX to the crate");

    println!("cargo:rustc-link-lib=static=ktx_c");
    println!("cargo:rustc-link-lib=static=ktx");

    #[cfg(feature = "link-libstdc++")]
    println!("cargo:rustc-link-lib=dylib=stdc++");

    #[cfg(feature = "run-bindgen")]
    {
        println!("-- Generate Rust bindings");

        let bindings = bindgen::Builder::default()
            .header(MAIN_HEADER)
            //
            .opaque_type("FILE")
            .allowlist_function(r"ktx.*")
            .allowlist_type(r"[Kk][Tt][Xx].*")
            .allowlist_var(r"[Kk][Tt][Xx].*")
            //
            .clang_arg("-fparse-all-comments")
            .generate()
            .expect("generating the bindings");

        let mut out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
        out_path.push("bindings.rs");
        bindings
            .write_to_file(out_path)
            .expect("writing the generated bindings to file");
    }

    println!("-- All done");
    println!("cargo:rerun-if-changed=build/build.rs");
}
