use quote::quote;
use syn::{parse2, spanned::Spanned, Attribute, Data, DeriveInput, Expr, Lit, Meta, Result};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = match parse(input.into()) {
        Ok(input) => input,
        Err(e) => {
            return e.into_compile_error().into();
        }
    };

    match generate_code(ast) {
        Ok(stream) => stream.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

struct Field {
    ident: proc_macro2::Ident,
    format: Option<String>,
}

struct Ast {
    fields: Vec<Field>,
    name: proc_macro2::Ident,
}

fn parse(input: proc_macro2::TokenStream) -> Result<Ast> {
    let derive_input: DeriveInput = parse2(input)?;

    let Data::Struct(data_struct) = derive_input.data else {
        return Err(syn::Error::new(
            derive_input.ident.span(),
            "#[derive(CustomDebug) supports structs only",
        ));
    };

    let is_tuple_struct = data_struct.fields.iter().any(|f| f.ident.is_none());
    if is_tuple_struct {
        return Err(syn::Error::new(
            derive_input.ident.span(),
            "#[derive(CustomDebug)] does not work for a tuple struct",
        ));
    }

    let mut fields = Vec::with_capacity(data_struct.fields.len());
    for field in &data_struct.fields {
        fields.push(Field {
            ident: field.ident.clone().unwrap(),
            format: get_format_str(&field.attrs)?,
        });
    }

    Ok(Ast {
        fields,
        name: derive_input.ident,
    })
}

fn get_format_str(attrs: &[Attribute]) -> Result<Option<String>> {
    let mut res = None;
    for attr in attrs {
        if let Meta::NameValue(mnv) = &attr.meta {
            if mnv.path.is_ident("debug") {
                if res.is_some() {
                    return Err(syn::Error::new(
                        attr.span(),
                        "debug attribute can only be applied once",
                    ));
                }

                let Expr::Lit(el) = &mnv.value else {
                    return Err(syn::Error::new(
                        attr.span(),
                        "rhs of debug should be a string literal",
                    ));
                };

                let Lit::Str(ls) = &el.lit else {
                    return Err(syn::Error::new(
                        attr.span(),
                        "rhs of debug should be a string literal",
                    ));
                };

                res = Some(ls.value())
            }
        }
    }
    Ok(res)
}

fn generate_code(ast: Ast) -> Result<proc_macro2::TokenStream> {
    let name = ast.name;

    let fields = ast.fields.iter().map(|f| {
        let field_name = &f.ident;
        let format = if let Some(format) = &f.format {
            quote! { &format_args!(#format, &self.#field_name) }
        } else {
            quote! { &self.#field_name }
        };
        quote! { .field(stringify!(#field_name), #format) }
    });

    let code = quote! {
        impl std::fmt::Debug for #name {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                fmt.debug_struct(stringify!(#name))
                    #(#fields)*
                   .finish()
            }
        }
    };
    Ok(code)
}
