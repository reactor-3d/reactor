use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{Data, DeriveInput};

pub fn enum_as_inner(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let variants = match &ast.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => return Err(syn::Error::new(Span::call_site(), "This macro only supports enums.")),
    };

    let enum_name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let variants: Vec<_> = variants
        .iter()
        .filter_map(|variant| match &variant.fields {
            syn::Fields::Unnamed(values) => {
                let variant_name = &variant.ident;
                let types: Vec<_> = values.unnamed.iter().map(|field| field.ty.to_token_stream()).collect();
                let field_names: Vec<_> = values
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let name = "x".repeat(i + 1);
                        let name = format_ident!("{}", name);
                        quote! {#name}
                    })
                    .collect();

                let ref_fn_name = format_ident!("{}_ref", variant_name.to_string().to_snake_case());
                let as_ref_fn_name = format_ident!("as_{}_ref", variant_name.to_string().to_snake_case());
                let mut_fn_name = format_ident!("{}_mut", variant_name.to_string().to_snake_case());
                let as_mut_fn_name = format_ident!("as_{}_mut", variant_name.to_string().to_snake_case());

                Some(quote! {
                    #[must_use]
                    #[inline]
                    pub fn #ref_fn_name(&self) -> ::core::option::Option<(#(&#types),*)> {
                        match self {
                            #enum_name::#variant_name (#(#field_names),*) => Some((#(#field_names),*)),
                            _ => None
                        }
                    }

                    #[must_use]
                    #[inline]
                    pub fn #as_ref_fn_name(&self) -> (#(&#types),*) {
                        match self {
                            #enum_name::#variant_name (#(#field_names),*) => (#(#field_names),*),
                            _ => panic!("Unexpected enum variant for {}", stringify!(#variant_name))
                        }
                    }

                    #[must_use]
                    #[inline]
                    pub fn #mut_fn_name(&mut self) -> ::core::option::Option<(#(&mut #types),*)> {
                        match self {
                            #enum_name::#variant_name (#(#field_names),*) => Some((#(#field_names),*)),
                            _ => None
                        }
                    }

                    #[must_use]
                    #[inline]
                    pub fn #as_mut_fn_name(&mut self) -> (#(&mut #types),*) {
                        match self {
                            #enum_name::#variant_name (#(#field_names),*) => (#(#field_names),*),
                            _ => panic!("Unexpected enum variant for {}", stringify!(#variant_name))
                        }
                    }
                })
            },
            _ => {
                return None;
            },
        })
        .collect();

    Ok(quote! {
        impl #impl_generics #enum_name #ty_generics #where_clause {
            #(#variants)*
        }
    })
}
