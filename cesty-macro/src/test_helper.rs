extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

pub fn cesty_test(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let body = &input.block;

    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &input.sig;

    let head = quote! {
        #(#attrs)*
        #vis #sig
    };

    let body = quote! {
        #body
    };

    TokenStream::from(quote! {
        #[test]
        #head
        {
            cesty::test_runner::cesty_run_test(|| #body)
        }
    })
}
