use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Component)]
pub fn component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics Component for #name #ty_generics #where_clause {
            fn entity(&self) -> Arc<Mutex<Option<Arc<Entity>>>> {
                self.entity.clone()
            }

            fn set_entity(&self, entity: Option<Arc<Entity>>) {
                *self.entity.lock().unwrap() = entity;
            }

            fn id(&self) -> Arc<String> {
                self.id.clone()
            }

            fn tid(&self) -> Arc<String> {
                self.tid.clone()
            }

            fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync + 'static> {
                self.clone()
            }
        }
    };

    TokenStream::from(expanded)
}
