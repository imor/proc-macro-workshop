use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput};

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

    let data_struct = get_data_struct(&input.data, &item_ident)?;
    let struct_impl = generate_struct_impl(&item_ident, &builder_ident, data_struct);
    let builder = generate_builder_struct(&item_ident, &builder_ident, data_struct);

    Ok(quote! {
        #struct_impl
        #builder
    })
}

fn generate_struct_impl(
    item_ident: &Ident,
    builder_ident: &Ident,
    data_struct: &DataStruct,
) -> proc_macro2::TokenStream {
    let field_inits = data_struct.fields.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();
        quote! {
            #name: None,
        }
    });

    quote! {
        impl #item_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#field_inits)*
                }
            }
        }
    }
}

fn generate_builder_struct(
    item_ident: &Ident,
    builder_ident: &Ident,
    data_struct: &DataStruct,
) -> proc_macro2::TokenStream {
    let fields = data_struct.fields.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        quote! {
            #name: Option<#ty>,
        }
    });

    let field_mutators = data_struct.fields.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        quote! {
            fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    let field_set_checks = data_struct.fields.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();
        quote! {
            if self.#name.is_none() {
                let e = std::string::String::from(std::format!("{} must be set", stringify!(#name))).into();
                return Err(e);
            }
        }
    });

    let set_fields = data_struct.fields.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();
        quote! {
            #name: self.#name.as_ref().unwrap().clone(),
        }
    });

    let build_method = quote! {
        pub fn build(&mut self) -> Result<#item_ident, std::boxed::Box<dyn std::error::Error>> {
            #(#field_set_checks)*
            Ok(#item_ident {
                #(#set_fields)*
            })
        }
    };

    quote! {
        pub struct #builder_ident {
            #(#fields)*
        }

        impl #builder_ident {
            #(#field_mutators)*
            #build_method
        }
    }
}

fn get_data_struct<'a>(data: &'a Data, item_ident: &Ident) -> Result<&'a DataStruct, syn::Error> {
    match data {
        Data::Struct(data_struct) => {
            let is_tuple_struct = data_struct.fields.iter().any(|f| f.ident.is_none());
            if is_tuple_struct {
                return Err(syn::Error::new(
                    item_ident.span(),
                    "#[derive(Builder)] does not work for a tuple struct",
                ));
            }
            Ok(data_struct)
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
