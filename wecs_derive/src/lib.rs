use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, DeriveInput, GenericParam,
    Generics
};

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
        impl #impl_generics Component for #name #ty_generics #where_clause {
            fn entity(&self) -> Arc<Entity> {
                self.entity.clone()
            }

            fn id(&self) -> Arc<String> {
                self.id.clone()
            }

            fn type_id(&self) -> Arc<String> {
                self.type_id.clone()
            }
        }

        impl #impl_generics Drop for #name #ty_generics #where_clause {
            fn drop(&mut self) {
                self.on_drop();
                self.entity.remove_by_id(self.id);
            }
        }
    };

    TokenStream::from(expanded)
}
