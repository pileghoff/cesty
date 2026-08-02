extern crate proc_macro;
use proc_macro::TokenStream;

mod mock;
mod test_helper;

#[proc_macro]
pub fn define_mock(input: TokenStream) -> TokenStream {
    mock::define_mock(input)
}

#[proc_macro]
pub fn mock(input: TokenStream) -> TokenStream {
    mock::mock(input)
}

#[proc_macro_attribute]
pub fn cesty_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    test_helper::cesty_test(item)
}
