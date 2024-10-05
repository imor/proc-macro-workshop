use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal};
use quote::{format_ident, quote};
use syn::{
    parse::Parse, parse2, parse_macro_input, Attribute, Data, DataStruct, DeriveInput,
    GenericArgument, Meta, PathArguments, Token, Type,
};

#[proc_macro_derive(Builder, attributes(builder))]
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
    let builder = generate_builder_struct(&item_ident, &builder_ident, data_struct)?;

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
        let ty = &field.ty;
        if is_vec_type(ty) {
            quote! {
                #name: vec![],
            }
        } else {
            quote! {
                #name: None,
            }
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
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let fields = data_struct.fields.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        if is_option_type(ty) || is_vec_type(ty) {
            quote! {
                #name: #ty,
            }
        } else {
            quote! {
                #name: std::option::Option<#ty>,
            }
        }
    });

    let mut field_mutators = vec![];
    for field in &data_struct.fields {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let mutator = if is_option_type(ty) {
            let underlying_type = get_option_underlying_type(ty);
            quote! {
                fn #name(&mut self, #name: #underlying_type) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        } else if is_vec_type(ty) {
            if let Some(singular_name) = get_singular_name(&field.attrs)? {
                let underlying_type = get_vec_underlying_type(ty);
                quote! {
                    fn #singular_name(&mut self, #singular_name: #underlying_type) -> &mut Self {
                        self.#name.push(#singular_name);
                        self
                    }
                }
            } else {
                quote! {
                    fn #name(&mut self, #name: #ty) -> &mut Self {
                        self.#name = #name;
                        self
                    }
                }
            }
        } else {
            quote! {
                fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        };
        field_mutators.push(mutator);
    }

    let field_set_checks = data_struct.fields.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();
        if is_option_type(&field.ty) || is_vec_type(&field.ty) {
            quote!{}
        } else {
            quote! {
                if self.#name.is_none() {
                    let e = std::string::String::from(std::format!("{} must be set", stringify!(#name))).into();
                    return Err(e);
                }
            }
        }
    });

    let set_fields = data_struct.fields.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();
        if is_option_type(&field.ty) || is_vec_type(&field.ty) {
            quote! {
                #name: self.#name.clone(),
            }
        } else {
            quote! {
                #name: self.#name.as_ref().unwrap().clone(),
            }
        }
    });

    let build_method = quote! {
        pub fn build(&mut self) -> std::result::Result<#item_ident, std::boxed::Box<dyn std::error::Error>> {
            #(#field_set_checks)*
            Ok(#item_ident {
                #(#set_fields)*
            })
        }
    };

    Ok(quote! {
        pub struct #builder_ident {
            #(#fields)*
        }

        impl #builder_ident {
            #(#field_mutators)*
            #build_method
        }
    })
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

fn is_option_type(ty: &Type) -> bool {
    get_option_underlying_type(ty).is_some()
}

fn get_option_underlying_type(ty: &Type) -> Option<&Type> {
    get_underlying_type(ty, "Option")
}

fn get_vec_underlying_type(ty: &Type) -> Option<&Type> {
    get_underlying_type(ty, "Vec")
}

fn get_underlying_type<'t>(ty: &'t Type, type_name: &str) -> Option<&'t Type> {
    if let Type::Path(type_path) = ty {
        let segments = &type_path.path.segments;
        if segments.len() == 1 {
            let segment = &segments[0];
            if let PathArguments::AngleBracketed(abga) = &segment.arguments {
                if abga.args.len() == 1 && segment.ident == type_name {
                    let arg = &abga.args[0];
                    if let GenericArgument::Type(ty) = arg {
                        return Some(ty);
                    }
                }
            }
        }
    }

    None
}

struct Each {
    name: Literal,
}

impl Parse for Each {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let each: Ident = input.parse()?;
        if each != "each" {
            return Err(syn::Error::new(each.span(), r#"expected `each`"#));
        }
        input.parse::<Token![=]>()?;
        let name: Literal = input.parse()?;
        Ok(Each { name })
    }
}

fn is_vec_type(ty: &Type) -> bool {
    get_vec_underlying_type(ty).is_some()
}

fn get_singular_name(attrs: &[Attribute]) -> Result<Option<Ident>, syn::Error> {
    for attr in attrs {
        if attr.path().is_ident("builder") {
            if let Meta::List(meta_list) = &attr.meta {
                let Each { name } = parse2(meta_list.tokens.clone())?;
                //TODO: find a better method of converting a string literal into ident without the surrounding doubel quotes
                let name_str = name.to_string();
                let name_str = &name_str[1..name_str.len() - 1];
                let name_ident = format_ident!("{}", name_str);
                return Ok(Some(name_ident));
            }
        }
    }
    Ok(None)
}
