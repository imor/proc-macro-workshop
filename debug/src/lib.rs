use quote::quote;
use syn::{parse2, Data, DeriveInput, Result};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = match parse(input.into()) {
        Ok(input) => input,
        Err(e) => {
            return e.into_compile_error().into();
        }
    };

    match generate_code(input) {
        Ok(stream) => stream.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn parse(input: proc_macro2::TokenStream) -> Result<DeriveInput> {
    let derive_input = parse2(input)?;
    Ok(derive_input)
}

fn generate_code(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let name = input.ident;
    let Data::Struct(data_struct) = input.data else {
        return Err(syn::Error::new(
            name.span(),
            "only structs supported currently",
        ));
    };

    let fields = data_struct.fields.iter().map(|f| {
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
