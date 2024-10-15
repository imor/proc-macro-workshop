use quote::quote;
use syn::{parse2, Data, DataStruct, DeriveInput, Result};

#[proc_macro_derive(CustomDebug)]
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

struct Ast {
    data_struct: DataStruct,
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
    Ok(Ast {
        data_struct,
        name: derive_input.ident,
    })
}

fn generate_code(ast: Ast) -> Result<proc_macro2::TokenStream> {
    let name = ast.name;

    let fields = ast.data_struct.fields.iter().map(|f| {
        let field_name = f.ident.clone().unwrap();
        quote! { .field(stringify!(#field_name), &self.#field_name) }
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
