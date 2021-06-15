// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "run-bindgen")]
use bindgen;
use cc;
use std::path::PathBuf;

// All of these values are taken from `KTX-Software/CMakeLists.txt`
// SPDX-License-Identifier: Apache-2.0

const SOURCE_DIR: &str = "build/KTX-Software";

const INCLUDE_DIRS: &[&str] = &[
    "build/",
    "build/KTX-Software/include",
    "build/KTX-Software/lib",
    "build/KTX-Software/lib/basisu/transcoder",
    "build/KTX-Software/lib/basisu/zstd",
    "build/KTX-Software/other_include",
    "build/KTX-Software/utils",
];

#[cfg(feature = "run-bindgen")]
const MAIN_HEADER: &str = "build/wrapper.h";

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

#[cfg(feature = "write")]
const WRITE_C_SOURCE_FILES: &[&str] = &[
    // KTX_FEATURE_WRITE
    "lib/writer1.c",
    "lib/writer2.c",
    "lib/basisu/encoder/apg_bmp.c",
];

const CXX_SOURCE_FILES: &[&str] = &[
    "lib/basis_encode.cpp",
    "lib/basis_transcode.cpp",
    "lib/basisu/transcoder/basisu_transcoder.cpp",
    "lib/etcunpack.cxx",
    // this file is not open source, so it's gated behind a feature. see readme.
    #[cfg(feature = "nonfree-etc-unpack")]
    "lib/etcdec.cxx",
];

#[cfg(feature = "write")]
const WRITE_CXX_SOURCE_FILES: &[&str] = &[
    // KTX_FEATURE_WRITE
    "lib/basisu/encoder/basisu_astc_decomp.cpp",
    "lib/basisu/encoder/basisu_backend.cpp",
    "lib/basisu/encoder/basisu_basis_file.cpp",
    "lib/basisu/encoder/basisu_bc7enc.cpp",
    "lib/basisu/encoder/basisu_comp.cpp",
    "lib/basisu/encoder/basisu_enc.cpp",
    "lib/basisu/encoder/basisu_etc.cpp",
    "lib/basisu/encoder/basisu_frontend.cpp",
    "lib/basisu/encoder/basisu_global_selector_palette_helpers.cpp",
    "lib/basisu/encoder/basisu_gpu_texture.cpp",
    "lib/basisu/encoder/basisu_kernels_sse.cpp",
    "lib/basisu/encoder/basisu_pvrtc1_4.cpp",
    "lib/basisu/encoder/basisu_resample_filters.cpp",
    "lib/basisu/encoder/basisu_resampler.cpp",
    "lib/basisu/encoder/basisu_ssim.cpp",
    "lib/basisu/encoder/basisu_uastc_enc.cpp",
    "lib/basisu/encoder/jpgd.cpp",
    "lib/basisu/encoder/lodepng.cpp",
];

fn spooky_warning(msg: &str) {
    // @s are the most spooky character, as demonstrated by openSSH's warnings.
    println!("cargo:warning={:@<120}", "");
    for line in msg.split_terminator("\n") {
        println!("cargo:warning=@  {: ^114}  @", line);
    }
    println!("cargo:warning={:@<120}", "");
}

const NONFREE_ETC_WARN: &str = "feature(nonfree-etc-unpack) is enabled!
This feature enables compilation of KTX-Software/lib/etcdec.cxx, which is proprietary software!
This taints the license of the code, which is NOT fully Apache-2.0-licensed anymore!
For a fully Apache-2.0-licensed codebase, disable the feature in question.";

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
        .define(
            "KTX_FEATURE_WRITE",
            if cfg!(feature = "write") { "1" } else { "0" },
        ) // For libktx_rs::sinks::
        .define("KTX_FEATURE_GL_UPLOAD", "0")
        .define("KTX_FEATURE_VULKAN", "0")
        .define("KTX_OMIT_VULKAN", "1");

    // lib/etcdec.cxx is not open source, so it's gated behind a feature. see readme.
    let have_software_etc = if cfg!(feature = "nonfree-etc-unpack") {
        spooky_warning(NONFREE_ETC_WARN);
        "1"
    } else {
        "0"
    };
    build.define("SUPPORT_SOFTWARE_ETC_UNPACK", have_software_etc);

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    match target_os.as_str() {
        "windows" => {
            build
                //.file("lib/internalexport.def")
                //.file("lib/internalexport_write.def")
                .define("KTX_API", "__declspec(dllexport)")
                .define("BASISU_NO_ITERATOR_DEBUG_LEVEL", "1");
        }
        "linux" => {
            build.flag("-pthread").flag("-ldl");
        }
        _ if target_arch.starts_with("wasm") => {
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
    // HACK to get proper relative paths
    std::env::set_current_dir(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .expect("Failed to chdir");

    println!("-- Build the native libKTX");
    {
        let mut c_build = configure_build(cc::Build::new());
        c_build
            .cpp(false)
            .files(ktx_sources(C_SOURCE_FILES))
            .file("build/wrapper.c");

        #[cfg(feature = "write")]
        c_build.files(ktx_sources(WRITE_C_SOURCE_FILES));

        c_build.compile("ktx_c");
    }
    {
        let mut cxx_build = configure_build(cc::Build::new());
        cxx_build
            .cpp(true)
            .files(ktx_sources(CXX_SOURCE_FILES))
            // AppleClang seemingly defaults to C++98...
            .flag_if_supported("-std=c++14");

        #[cfg(feature = "write")]
        cxx_build.files(ktx_sources(WRITE_CXX_SOURCE_FILES));
        
        cxx_build.compile("ktx");
    }

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
            .blocklist_type("ktx_size_t")
            .raw_line("pub type ktx_size_t = usize;")
            .blocklist_type("ktx_off_t")
            .raw_line("#[cfg(target_os = \"windows\")]")
            .raw_line("pub type ktx_off_t = i64;")
            .raw_line("#[cfg(not(target_os = \"windows\"))]")
            .raw_line("pub type ktx_off_t = isize;")
            //
            .clang_arg("-fparse-all-comments")
            .clang_args(INCLUDE_DIRS.iter().map(|id| format!("-I{}", id)))
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
