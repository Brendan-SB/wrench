use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Fields, GenericParam,
    Generics, Index,
};
use wecs::{Component, Entity, Registry};

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(heapsize::HeapSize));
        }
    }
    generics
}

#[proc_macro_derive(Component)]
pub fn component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics Entity for #name #ty_generics #where_clause {
            fn registry(self: Arc<Self>) -> Option<Arc<Registry>> {
                self.registry.clone()
            }

            fn components(self: Arc<Self>) -> Arc<Vec<Arc<Component>>> {
                self.components.clone()
            }
        }

        impl #impl_generics Drop for #name #ty_generics #where_clause {
            fn drop(&mut self) {
                self.on_drop();
                self.registry.remove_by_id(self.id);
            }
        }
    };

    TokenStream::from(expanded)
}
