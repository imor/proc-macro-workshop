use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{parse2, Error, Item, Result};

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

fn parse(input: proc_macro2::TokenStream) -> Result<Ast> {
    let item = parse2(input)?;
    let Item::Enum(_) = item else {
        return Err(Error::new(
            Span::call_site(),
            "expected enum or match expression",
        ));
    };
    Ok(item)
}
