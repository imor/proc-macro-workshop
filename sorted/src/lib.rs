use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse2;

type Ast = syn::Item;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;

    let ast = match parse(input.into()) {
        Ok(ast) => ast,
        Err(e) => {
            return e.into_compile_error().into();
        }
    };

    ast.to_token_stream().into()
}

fn parse(input: proc_macro2::TokenStream) -> syn::Result<Ast> {
    parse2(input)
}
