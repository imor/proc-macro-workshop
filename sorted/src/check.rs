use quote::{quote, ToTokens};
use syn::{
    parse2, spanned::Spanned, visit_mut::VisitMut, Arm, Attribute, Error, ExprMatch, Pat, PatIdent,
    Path, Result,
};

pub fn check_impl(input: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream> {
    let item_fn = parse(input)?;
    Ok(item_fn)
}

fn parse(input: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream> {
    let mut item = parse2(input)?;
    let mut attr_stripper = SortedAttrStripper { error: None };
    attr_stripper.visit_item_fn_mut(&mut item);
    let mut stream = item.to_token_stream();
    if let Some(e) = attr_stripper.error {
        stream.extend(e.into_compile_error());
    };
    Ok(stream)
}

struct SortedAttrStripper {
    error: Option<syn::Error>,
}

impl SortedAttrStripper {
    fn report_error(&mut self, error: Error) {
        if self.error.is_none() {
            self.error = Some(error)
        }
    }
}

impl VisitMut for SortedAttrStripper {
    fn visit_expr_match_mut(&mut self, node: &mut ExprMatch) {
        if node.attrs.iter().any(is_sorted_attr) {
            node.attrs.retain(is_not_sorted_attr);
        }

        let mut prev_paths = vec![];
        for (i, arm) in node.arms.iter().enumerate() {
            let (is_wild, path) = get_arm_path(arm);
            if is_wild {
                if i == node.arms.len() - 1 {
                    continue;
                } else {
                    self.report_error(Error::new(arm.pat.span(), "_ should sort at the end"));
                }
            }
            if let Some(path) = path {
                let curr_path = arm_path_to_string(&path);
                for prev_path in &prev_paths {
                    if curr_path < *prev_path {
                        self.report_error(Error::new_spanned(
                            path.clone(),
                            format!("{} should sort before {}", curr_path, prev_path),
                        ));
                        break;
                    }
                }
                prev_paths.push(curr_path);
            } else {
                self.report_error(Error::new(arm.pat.span(), "unsupported by #[sorted]"));
            }
        }
    }
}

fn is_not_sorted_attr(attr: &Attribute) -> bool {
    !is_sorted_attr(attr)
}

fn is_sorted_attr(attr: &Attribute) -> bool {
    attr.meta.path().is_ident("sorted")
}

fn get_arm_path(arm: &Arm) -> (bool, Option<Path>) {
    match arm.pat {
        Pat::TupleStruct(ref p) => (false, Some(p.path.clone())),
        Pat::Path(ref p) => (false, Some(p.path.clone())),
        Pat::Ident(PatIdent { ident: ref p, .. }) => (false, Some(p.clone().into())),
        Pat::Wild(_) => (true, None),
        _ => (false, None),
    }
}

fn arm_path_to_string(path: &Path) -> String {
    path.segments
        .iter()
        .map(|s| format!("{}", quote! {#s}))
        .collect::<Vec<_>>()
        .join("::")
}
