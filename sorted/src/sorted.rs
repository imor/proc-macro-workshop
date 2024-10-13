use proc_macro2::Span;
use syn::{
    parse2, punctuated::Punctuated, spanned::Spanned, token::Comma, Error, Item, Result, Variant,
};

pub fn sorted_impl(input: proc_macro2::TokenStream) -> Result<()> {
    let item = parse(input)?;
    analyze(item)?;
    Ok(())
}

fn analyze(item: Item) -> Result<()> {
    let Item::Enum(item_enum) = item.clone() else {
        unreachable!()
    };

    check_sorting(&item_enum.variants)?;

    Ok(())
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
