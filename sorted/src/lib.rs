use check::check_impl;
use sorted::sorted_impl;

mod check;
mod sorted;

#[proc_macro_attribute]
pub fn sorted(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut out = input.clone();
    match sorted_impl(input.into()) {
        Ok(()) => out,
        Err(e) => {
            out.extend(proc_macro::TokenStream::from(e.into_compile_error()));
            out
        }
    }
}

#[proc_macro_attribute]
pub fn check(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut out = input.clone();
    match check_impl(input.into()) {
        Ok(()) => out,
        Err(e) => {
            out.extend(proc_macro::TokenStream::from(e.into_compile_error()));
            out
        }
    }
}
