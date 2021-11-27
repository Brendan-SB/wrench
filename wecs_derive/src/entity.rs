use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Fields, GenericParam,
    Generics, Index,
};

#[proc_macro_derive(Entity)]
pub fn entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = quote! {
        impl #impl_generics Entity for #name #ty_generics #where_clause {
            fn registry(&self) -> usize {
                self.registry
            }
        }
    };

    TokenStream::from(expanded)
}
