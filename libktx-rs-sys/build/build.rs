// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0

use cmake;
use glob::glob;

const SOURCE_DIR: &str = "build/KTX-Software";
const CMAKELISTS: &str = "build/KTX-Software/CMakeLists.txt";

#[cfg(feature = "run-bindgen")]
mod run_bindgen {
    const INCLUDE_DIRS: &[&str] = &[
        "build/",
        "build/KTX-Software/include",
        "build/KTX-Software/lib",
        "build/KTX-Software/lib/basisu/transcoder",
        "build/KTX-Software/lib/basisu/zstd",
        "build/KTX-Software/other_include",
        "build/KTX-Software/utils",
    ];

    const MAIN_HEADER: &str = "build/wrapper.h";

    pub(crate) fn generate_bindings() {
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

        let mut out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
        out_path.push("bindings.rs");
        bindings
            .write_to_file(out_path)
            .expect("writing the generated bindings to file");
    }
}

mod etc_unpack {
    use std::{
        fs::OpenOptions,
        io::{Read, Seek, SeekFrom, Write},
    };

    #[allow(unused)]
    const NONFREE_ETC_WARN: &str = "feature(nonfree-etc-unpack) is enabled!
This feature enables compilation of KTX-Software/lib/etcdec.cxx, which is proprietary software!
This taints the license of the code, which is NOT fully Apache-2.0-licensed anymore!
For a fully Apache-2.0-licensed codebase, disable the feature in question.";

    #[allow(unused)]
    fn spooky_warning(msg: &str) {
        // @s are the most spooky character, as demonstrated by openSSH's warnings.
        println!("cargo:warning={:@<120}", "");
        for line in msg.split_terminator("\n") {
            println!("cargo:warning=@  {: ^114}  @", line);
        }
        println!("cargo:warning={:@<120}", "");
    }

    const ETC_CMAKELISTS_PATCH: &str = r#"
# BEGIN PATCH
include("../no_etc_unpack.cmake")
# END PATCH
"#;

    fn patch_cmakelists() -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(super::CMAKELISTS)?;
        let patch_pos = SeekFrom::End(-(ETC_CMAKELISTS_PATCH.len() as i64));
        file.seek(patch_pos)?;

        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf)?;
        if buf != ETC_CMAKELISTS_PATCH.as_bytes() {
            file.write(ETC_CMAKELISTS_PATCH.as_bytes())?;
            file.flush()?;
        }

        Ok(())
    }

    pub(crate) fn toggle(build: &mut cmake::Config) -> &mut cmake::Config {
        patch_cmakelists().expect("error patching CMakeLists.txt");

        build.define(
            "KTX_BUILD_ETC_UNPACK",
            if cfg!(feature = "nonfree-etc-unpack") {
                spooky_warning(NONFREE_ETC_WARN);
                "ON"
            } else {
                "OFF"
            },
        );

        build
    }
}

#[cfg_attr(feature = "docs-only", allow(unreachable_code))]
fn main() {
    #[cfg(feature = "docs-only")]
    {
        println!("-- docs-only build; quitting");
        return;
    }

    let (static_library, static_library_flag, lib_kind) = if cfg!(feature = "static") {
        (true, "ON", "static")
    } else {
        (false, "OFF", "dylib")
    };
    println!("-- Build KTX-Software");

    let mut lib_dir = etc_unpack::toggle(
        cmake::Config::new(SOURCE_DIR)
            .pic(true)
            .define("KTX_FEATURE_STATIC_LIBRARY", static_library_flag),
    )
    .build();
    println!("Built {} to {:?}", lib_kind, lib_dir);
    lib_dir.push("lib");

    println!("-- Link the native libKTX to the crate");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib={}=ktx", lib_kind);

    if static_library {
        // When building statically, the ASTC decoder is a separate static library
        // (otherwise, it's built inside libktx.so)
        let astc_lib_path = glob(format!("{}/*astcenc*.*", lib_dir.display()).as_str())
            .expect("globbing lib/")
            .next()
            .expect("[lib]astcenc*.{a,lib} to be present")
            .expect("the globbed path to be valid");
        let astc_lib_name = astc_lib_path
            .file_stem()
            .expect("this path to refer to a filename")
            .to_string_lossy();
        let astc_lib_name = match astc_lib_name.strip_prefix("lib") {
            Some(stripped) => stripped,
            None => &astc_lib_name,
        };

        println!("cargo:rustc-link-lib=static={}", astc_lib_name);
    }

    // Linux: GNU C++ standard library
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=stdc++");
    // AppleDarwin, BSDs, Android...: LLVM's C++ standard library
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    println!("cargo:rustc-link-lib=dylib=c++");

    #[cfg(feature = "run-bindgen")]
    run_bindgen::generate_bindings();

    println!("-- All done");
    println!("cargo:rerun-if-changed=build/build.rs");
    println!("cargo:rerun-if-changed=build/no_etc_unpack.cmake");
    println!("cargo:rerun-if-changed=build/KTX-Software/CMakeLists.txt");
}
