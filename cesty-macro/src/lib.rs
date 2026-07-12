extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    FnArg, Ident, Signature, Type, parse_macro_input, parse_str, punctuated::Punctuated,
    token::Comma,
};

fn get_names(in_sig: &Punctuated<FnArg, Comma>) -> Punctuated<Ident, Comma> {
    let mut res = Punctuated::new();

    in_sig
        .iter()
        .map(|f| match f {
            FnArg::Typed(t) => match *t.pat.clone() {
                syn::Pat::Ident(id) => id.ident,
                _ => unimplemented!(),
            },
            FnArg::Receiver(_) => unimplemented!(),
        })
        .for_each(|f| res.push(f));

    res
}

fn get_types(in_sig: &Punctuated<FnArg, Comma>) -> Punctuated<Type, Comma> {
    let mut res = Punctuated::new();

    in_sig
        .iter()
        .map(|f| match f {
            FnArg::Typed(t) => *t.ty.clone(),
            FnArg::Receiver(_) => unimplemented!(),
        })
        .for_each(|f| res.push(f));

    res
}

#[proc_macro]
pub fn define_mock(input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as Signature);
    let static_mock_name = format_ident!(
        "STATIC_MOCK_{}",
        func.ident.to_string().to_ascii_uppercase()
    );
    let extern_name = func.ident;
    let in_types = get_types(&func.inputs);
    let in_names = get_names(&func.inputs);
    let in_sig = func.inputs;
    let out_sig: Type = match func.output {
        syn::ReturnType::Default => parse_str("()").unwrap(),
        syn::ReturnType::Type(_, t) => *t,
    };

    quote!(
        cesty::lazy_static! {
            static ref #static_mock_name: std::sync::Mutex<cesty::FunctionMockInner<(#in_types), #out_sig>> =
                std::sync::Mutex::new(cesty::FunctionMockInner::new(None));
        }

        #[unsafe(no_mangle)]
        unsafe extern "C" fn #extern_name(#in_sig) -> #out_sig {
            let mut cesty_mutex = #static_mock_name.lock().unwrap();
            cesty_mutex.handle( (#in_names) )
        }
    )
    .into()
}

#[proc_macro]
pub fn mock(input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as Ident);
    let static_mock_name = format_ident!("STATIC_MOCK_{}", func.to_string().to_ascii_uppercase());

    quote!(
        {
            cesty::FunctionMock::new(&#static_mock_name)
        }
    )
    .into()
}
