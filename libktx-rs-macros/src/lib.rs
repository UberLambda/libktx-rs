// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0

use glob::glob;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::{collections::HashSet, iter::FromIterator};
use syn::{
    self,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, LitStr, Path, Token,
};

type LitStrList = Punctuated<LitStr, Token![,]>;

struct FileTestsInput {
    test_fn: Path,
    globs: LitStrList,
}

impl Parse for FileTestsInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let test_fn: Path = input.parse()?;
        input.parse::<Token![=>]>()?;
        let globs: LitStrList = input.parse_terminated(<LitStr as Parse>::parse)?;
        Ok(FileTestsInput { test_fn, globs })
    }
}

/// ```rust,ignore
/// file_tests!(test_fn => "glob", "glob", ...);
/// ````
/// For each file matching the given glob pattern[s] (at compile time!), generates a `#[test]` that invokes
/// ```rust,ignore
/// fn test_fn(file: std::fs::File);
/// ````
#[proc_macro]
pub fn file_tests(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as FileTestsInput);

    let test_files: HashSet<std::path::PathBuf> = input
        .globs
        .iter()
        .filter_map(|pattern| glob(pattern.value().as_str()).ok())
        .flat_map(|paths| paths.filter_map(|path| path.ok()))
        .collect();
    let test_fn_name = input.test_fn.segments.last().unwrap().ident.to_string();

    let fns_tokens = test_files.iter().map(|path| {
        let mut fn_name = path
            .file_stem()
            .map(|name| {
                format!(
                    "test_{}_{}",
                    test_fn_name,
                    name.to_str().expect("Invalid filename")
                )
            })
            .expect("Invalid globbed path");
        // Sanitize the identifier
        fn_name = fn_name
            .chars()
            .map(|ch| match ch {
                'A'..='Z' | 'a'..='z' | '0'..='9' => ch,
                _ => '_',
            })
            .collect();

        let test_fn = &input.test_fn;
        let abs_path = path.canonicalize().expect("Could not make absolute path");
        let path_str = abs_path.to_str().expect("Invalid path");
        let fn_ident = Ident::new(fn_name.as_str(), Span::call_site());

        quote! {
            #[test]
            fn #fn_ident() {
                let path = std::path::PathBuf::from(#path_str);
                println!(">>> Test file: {} <<<", #path_str);
                match std::fs::File::open(path) {
                    Ok(file) => #test_fn(file),
                    Err(err) => panic!("Error loading test file: {}: {}", #path_str, err),
                }
            }
        }
    });

    proc_macro2::TokenStream::from_iter(fns_tokens).into()
}
