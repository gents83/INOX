use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse::Parse, parse_macro_input, Attribute, ItemTrait, Token};

pub struct TraitInfo {
    item_trait: ItemTrait,
}

impl Parse for TraitInfo {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![pub]) || lookahead.peek(Token![trait]) {
            let mut item_trait: ItemTrait = input.parse()?;
            item_trait.attrs = attrs;
            Ok(TraitInfo { item_trait })
        } else {
            Err(lookahead.error())
        }
    }
}

pub fn serializable_trait(_args: TokenStream, input: TokenStream) -> TokenStream {
    let trait_info = parse_macro_input!(input as TraitInfo);
    let item_trait = &trait_info.item_trait;
    let trait_ident = &item_trait.ident;
    let serializable_trait_ident = Ident::new(
        &format!("Serializable{}", item_trait.ident),
        Span::call_site(),
    );

    TokenStream::from(quote! {
        #item_trait

        erased_serde::serialize_trait_object!(#trait_ident);

        #[derive(Clone)]
        pub struct #serializable_trait_ident {
            get_func: fn(&dyn Serializable) -> Box<dyn Serializable>,
        }

        impl #serializable_trait_ident {
            fn get_data(&self, value: &dyn Serializable) -> Box<dyn Serializable> {
                (self.get_func)(value)
            }
        }

        impl<T> SerializableType<T> for #serializable_trait_ident
        where T: #trait_ident + FromSerializable + Serializable {
            fn from_value(&self, value: &dyn Serializable, registry: &SerializableRegistry) -> T {
                T::from_serializable(value, registry).unwrap()
            }
            fn from_type_to_serializable() -> Self {
                Self {
                    get_func: |value| {
                        let data = value.downcast_ref::<T>().map(|value| value as &dyn Serializable).unwrap();
                        data.duplicate()
                    },
                }
            }
        }

        impl Clone for Box<dyn #trait_ident> {
            fn clone(&self) -> Box<dyn #trait_ident> {
                self.clone_trait()
            }
        }

        impl_boxed_trait!(dyn #trait_ident);
    })
}
