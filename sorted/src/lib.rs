use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    parse2, punctuated::Punctuated, spanned::Spanned, token::Comma, Error, Item, Result, Variant,
};

#[proc_macro_attribute]
pub fn sorted(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match sorted_impl(input.into()) {
        Ok(res) => res,
        Err(e) => e.into_compile_error(),
    }
    .into()
}

fn sorted_impl(input: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream> {
    let item = parse(input)?;
    let item = analyze(item)?;
    Ok(item.to_token_stream())
}

fn analyze(item: Item) -> Result<Item> {
    let Item::Enum(item_enum) = item.clone() else {
        unreachable!()
    };

    check_sorting(&item_enum.variants)?;

    Ok(item)
}

fn check_sorting(variants: &Punctuated<Variant, Comma>) -> Result<()> {
    for (i, variant_curr) in variants.iter().enumerate() {
        let name_curr = variant_curr.ident.to_string();
        for j in 0..i {
            let variant_prev = variants.get(j).expect("failed to get previous variant");
            let name_prev = variant_prev.ident.to_string();
            if name_curr < name_prev {
                return Err(Error::new(
                    variant_curr.span(),
                    format!("{name_curr} should sort before {name_prev}"),
                ));
            }
        }
    }
    Ok(())
}

fn parse(input: proc_macro2::TokenStream) -> Result<Item> {
    let item = parse2(input)?;

    let Item::Enum(_) = item else {
        return Err(Error::new(
            Span::call_site(),
            "expected enum or match expression",
        ));
    };

    Ok(item)
}
