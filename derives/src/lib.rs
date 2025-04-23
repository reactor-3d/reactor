use syn::DeriveInput;

mod enum_as;

#[proc_macro_derive(EnumAs)]
pub fn enum_as(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let toks = enum_as::enum_as_inner(&ast).unwrap_or_else(|err| err.to_compile_error());
    toks.into()
}
