use proc_macro::TokenStream;
use syn::{parse2, DeriveInput, Result};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    match parse(input.into()) {
        Ok(_) => TokenStream::new(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn parse(input: proc_macro2::TokenStream) -> Result<DeriveInput> {
    let derive_input = parse2(input)?;
    Ok(derive_input)
}
