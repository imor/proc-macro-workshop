use proc_macro2::Span;
use quote::quote;
use syn::{
    parse2, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Colon, PathSep, Where},
    Attribute, Data, DeriveInput, Expr, GenericArgument, GenericParam, Generics, Ident, Lit, Meta,
    Path, PathArguments, PathSegment, PredicateType, Result, Type, TypePath, WhereClause,
    WherePredicate,
};

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

#[derive(Debug)]
struct Field {
    ident: proc_macro2::Ident,
    format: Option<String>,
    ty: Type,
}

struct Ast {
    fields: Vec<Field>,
    name: proc_macro2::Ident,
    generics: Generics,
    attrs: Vec<Attribute>,
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
            ty: field.ty.clone(),
        });
    }

    Ok(Ast {
        fields,
        name: derive_input.ident,
        generics: derive_input.generics,
        attrs: derive_input.attrs,
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
    let ehb = get_escape_hatch_bound(&ast.attrs);
    let enable_bound_inference = ehb.is_none();
    let generics = add_generic_trait_bounds(ast.generics, &ast.fields, enable_bound_inference);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let wc = where_clause.cloned();
    let preds = get_assoc_type_where_clause_preds(&ast.fields);

    let mut wc = if let Some(wc) = wc {
        wc
    } else {
        WhereClause {
            where_token: Where::default(),
            predicates: Punctuated::default(),
        }
    };

    if let Some(ehb) = ehb {
        let pred = WherePredicate::Type(PredicateType {
            lifetimes: None,
            bounded_ty: ehb,
            colon_token: Colon::default(),
            bounds: parse_quote!(std::fmt::Debug),
        });
        wc.predicates.push(pred);
    } else {
        wc.predicates.extend(preds);
    }

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
        impl #impl_generics std::fmt::Debug for #name #ty_generics #wc {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                fmt.debug_struct(stringify!(#name))
                    #(#fields)*
                   .finish()
            }
        }
    };
    Ok(code)
}

fn add_generic_trait_bounds(
    mut generics: Generics,
    fields: &[Field],
    enable_bound_inference: bool,
) -> Generics {
    if !enable_bound_inference {
        return generics;
    }
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut tp) = *param {
            let mut generate_bound = true;
            for field in fields {
                if omit_bound(field, &tp.ident) {
                    generate_bound = false;
                    break;
                }
            }

            if generate_bound {
                tp.bounds.push(parse_quote!(std::fmt::Debug));
            }
        }
    }
    generics
}

fn omit_bound(field: &Field, param: &Ident) -> bool {
    if let Type::Path(tp) = &field.ty {
        for segment in &tp.path.segments {
            if let PathArguments::AngleBracketed(abga) = &segment.arguments {
                for arg in &abga.args {
                    if let GenericArgument::Type(Type::Path(p)) = arg {
                        if p.path.segments.len() > 1
                            && p.path.segments.first().unwrap().ident == *param
                        {
                            return true;
                        }
                        if p.path.is_ident(param) && segment.ident == "PhantomData" {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

fn get_assoc_type_where_clause_preds(fields: &[Field]) -> Vec<WherePredicate> {
    let mut res = vec![];
    for field in fields {
        if let Type::Path(tp) = &field.ty {
            for segment in &tp.path.segments {
                if let PathArguments::AngleBracketed(abga) = &segment.arguments {
                    for arg in &abga.args {
                        if let GenericArgument::Type(Type::Path(p)) = arg {
                            if p.path.segments.len() > 1 {
                                res.push(WherePredicate::Type(PredicateType {
                                    lifetimes: None,
                                    bounded_ty: parse_quote!(T::Value),
                                    colon_token: Colon::default(),
                                    bounds: parse_quote!(std::fmt::Debug),
                                }));
                            }
                        }
                    }
                }
            }
        }
    }
    res
}

fn get_escape_hatch_bound(attrs: &[Attribute]) -> Option<Type> {
    for attr in attrs {
        if let Meta::List(ml) = &attr.meta {
            if !ml.path.segments.is_empty() && ml.path.segments.first().unwrap().ident == "debug" {
                let mut tts = ml.tokens.clone().into_iter();
                let tt = tts.next();
                if let Some(proc_macro2::TokenTree::Ident(i)) = tt {
                    if i != "bound" {
                        continue;
                    }
                } else {
                    continue;
                }

                let tt = tts.next();
                if let Some(proc_macro2::TokenTree::Punct(p)) = tt {
                    if p.as_char() != '=' {
                        continue;
                    }
                } else {
                    continue;
                }

                let tt = tts.next();
                if let Some(proc_macro2::TokenTree::Literal(l)) = tt {
                    let s = String::from(l.to_string().trim_matches('"'));
                    let tokens = s.split("::");
                    let mut tokens_vec = vec![];
                    for token in tokens {
                        tokens_vec.push(token);
                    }
                    let len = tokens_vec.len();
                    let mut segments = Punctuated::new();
                    for (i, token) in tokens_vec.into_iter().enumerate() {
                        let path_seg = PathSegment::from(Ident::new(token, Span::call_site()));
                        segments.push(path_seg);

                        if i < len - 1 {
                            segments.push_punct(PathSep::default());
                        }
                    }
                    let t = Type::Path(TypePath {
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments,
                        },
                    });
                    return Some(t);
                } else {
                    continue;
                }
            }
        }
    }
    None
}
