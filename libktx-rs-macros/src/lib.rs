// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0

use glob::glob;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::{collections::HashSet, iter::FromIterator, ops::Sub};
use syn::{
    self,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, LitStr, Path, Token,
};

struct GlobPattern {
    inverted: bool,
    pattern: LitStr,
}

impl Parse for GlobPattern {
    fn parse(input: ParseStream) -> Result<Self> {
        let inverted = input.parse::<Token![!]>().is_ok();
        let pattern = input.parse()?;
        Ok(GlobPattern { inverted, pattern })
    }
}

type GlobPatternList = Punctuated<GlobPattern, Token![,]>;

struct FileTestsInput {
    test_fn: Path,
    globs: GlobPatternList,
}

impl Parse for FileTestsInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let test_fn: Path = input.parse()?;
        input.parse::<Token![=>]>()?;
        let globs: GlobPatternList = input.parse_terminated(GlobPattern::parse)?;
        Ok(FileTestsInput { test_fn, globs })
    }
}

fn glob_all<'a>(patterns: impl Iterator<Item = &'a GlobPattern>) -> HashSet<std::path::PathBuf> {
    patterns
        .filter_map(|pattern| glob(pattern.pattern.value().as_str()).ok())
        .flat_map(|paths| paths.filter_map(|path| path.ok()))
        .collect()
}

/// ```rust,ignore
/// file_tests!(test_fn => "glob", !"glob", ...);
/// ````
/// For each file matching the given glob pattern[s] (at compile time!), generates a `#[test]` that invokes
/// ```rust,ignore
/// fn test_fn(file: std::fs::File);
/// ````
/// Globs preceded by `!` are inverted (matches are removed).
#[proc_macro]
pub fn file_tests(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as FileTestsInput);

    let glob_accepted = glob_all(input.globs.iter().filter(|pattern| !pattern.inverted));
    let glob_rejected = glob_all(input.globs.iter().filter(|pattern| pattern.inverted));
    let test_files = glob_accepted.sub(&glob_rejected);

    let test_fn_name = input.test_fn.segments.last().unwrap().ident.to_string();

    let fns_tokens = test_files.iter().enumerate().map(|(i, path)| {
        let mut fn_name = path
            .file_stem()
            .map(|name| {
                format!(
                    "test{}_{}_{}",
                    i,
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
