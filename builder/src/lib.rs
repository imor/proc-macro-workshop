use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let code = match generate_code(input) {
        Ok(code) => code,
        Err(error) => return error.into_compile_error().into(),
    };

    TokenStream::from(code)
}

fn generate_code(input: DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    let item_ident = input.ident;
    let builder_ident = format_ident!("{item_ident}Builder");

    let struct_impl = generate_struct_impl(&item_ident, &builder_ident);
    let builder = generate_builder_struct(&input.data, &item_ident, &builder_ident)?;

    Ok(quote! {
        #struct_impl
        #builder
    })
}

fn generate_struct_impl(item_ident: &Ident, builder_ident: &Ident) -> proc_macro2::TokenStream {
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
    }
}

fn generate_builder_struct(
    data: &Data,
    item_ident: &Ident,
    builder_ident: &Ident,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    match data {
        Data::Struct(data_struct) => {
            let is_tuple_struct = data_struct.fields.iter().any(|f| f.ident.is_none());
            if is_tuple_struct {
                return Err(syn::Error::new(
                    item_ident.span(),
                    "#[derive(Builder)] does not work for a tuple struct",
                ));
            }
            let fields = data_struct.fields.iter().map(|field| {
                let name = field.ident.as_ref().unwrap();
                let ty = &field.ty;
                quote! {
                    #name: Option<#ty>,
                }
            });
            Ok(quote! {
                pub struct #builder_ident {
                    #(#fields)*
                }
            })
        }
        Data::Enum(_) => Err(syn::Error::new(
            item_ident.span(),
            "#[derive(Builder)] does not work for an enum",
        )),
        Data::Union(_) => Err(syn::Error::new(
            item_ident.span(),
            "#[derive(Builder)] does not work for a union",
        )),
    }
}
