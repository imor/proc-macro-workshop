use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let code = generate_code(input);

    TokenStream::from(code)
}

fn generate_code(input: DeriveInput) -> proc_macro2::TokenStream {
    let item_ident = input.ident;
    let builder_ident = Ident::new(&format!("{}Builder", item_ident), Span::call_site());
    quote! {
        impl #item_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }

        pub struct #builder_ident {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }
    }
}
