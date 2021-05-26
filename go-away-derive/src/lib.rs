#![allow(clippy::let_and_return)]

extern crate proc_macro;

use proc_macro::TokenStream;

use go_away_derive_internals::type_metadata_derive;

/// Derives TypeMetadata for a given struct.
///
/// This allows go-away to generate go types for a given type.
#[proc_macro_derive(TypeMetadata, attributes(serde))]
pub fn type_metadata_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let rv = match type_metadata_derive::type_metadata_derive(&ast) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    };

    //eprintln!("{}", rv);

    rv
}
