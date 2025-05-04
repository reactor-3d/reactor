use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput};

pub fn noded_inner(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let fields = match &ast.data {
        Data::Struct(data_struct) => &data_struct.fields,
        _ => return Err(syn::Error::new(Span::call_site(), "This macro only supports structs.")),
    };

    let struct_name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let mut subscription_methods = None;
    let mut resets = Vec::new();
    let mut input_index: usize = 0;

    for field in fields.iter() {
        if let Some(field_ident) = field.ident.as_ref() {
            if let syn::Type::Path(syn::TypePath { qself: None, path }) = &field.ty {
                if let Some(type_ident) = path.segments.last().map(|segment| &segment.ident) {
                    if *type_ident == format_ident!("NodePin") {
                        resets.push(quote! { #input_index => self.#field_ident.reset() });
                        input_index += 1;
                    }

                    if *type_ident == format_ident!("Subscription") {
                        subscription_methods = Some(quote! {
                            #[inline]
                            fn subscription_ref(&self) -> Option<&Subscription> {
                                Some(&self.#field_ident)
                            }

                            #[inline]
                            fn subscription_mut(&mut self) -> Option<&mut Subscription> {
                                Some(&mut self.#field_ident)
                            }
                        });
                    }
                }
            }
        }
    }

    let reset_input_method = if resets.is_empty() {
        None
    } else {
        Some(quote! {
            #[inline]
            fn reset_input(&mut self, pin: &egui_snarl::InPin) -> bool {
                match pin.id.input {
                    #(#resets,)*
                    _ => return false,
                }
                true
            }
        })
    };

    Ok(quote! {
        impl #impl_generics Noded for #struct_name #ty_generics #where_clause {
            #[inline]
            fn name(&self) -> &str {
                Self::NAME
            }

            #[inline]
            fn inputs(&self) -> &[u64] {
                &Self::INPUTS
            }

            #[inline]
            fn outputs(&self) -> &[u64] {
                &Self::OUTPUTS
            }

            #reset_input_method
            #subscription_methods
        }
    })
}
