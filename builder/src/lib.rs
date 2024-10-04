use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let code = generate_code(input);

    TokenStream::from(code)
}

fn generate_code(input: DeriveInput) -> proc_macro2::TokenStream {
    let item_ident = input.ident;
    let builder_ident = format_ident!("{item_ident}Builder");

    let builder = match generate_builder(&input.data, &item_ident, &builder_ident) {
        Ok(builder) => builder,
        Err(error) => return error,
    };

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

        #builder
    }
}

fn generate_builder(
    data: &Data,
    item_ident: &Ident,
    builder_ident: &Ident,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    match data {
        Data::Struct(data_struct) => {
            let is_tuple_struct = data_struct.fields.iter().any(|f| f.ident.is_none());
            if is_tuple_struct {
                return Err(quote_spanned! { item_ident.span() =>
                    compile_error!("#[derive(Builder)] does not work for a tuple struct");
                });
            }
            let mut fields = Vec::with_capacity(data_struct.fields.len());
            for field in &data_struct.fields {
                let name = field.ident.as_ref().unwrap();
                let ty = &field.ty;
                fields.push(quote! {
                    #name: Option<#ty>,
                });
            }
            Ok(quote! {
                pub struct #builder_ident {
                    #(#fields)*
                }
            })
        }
        Data::Enum(_) => Err(quote_spanned! { item_ident.span() =>
            compile_error!("#[derive(Builder)] does not work for an enum");
        }),
        Data::Union(_) => Err(quote_spanned! { item_ident.span() =>
            compile_error!("#[derive(Builder)] does not work for a union");
        }),
    }
}
