use crate::{Mode, TraitArgs};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_quote, Error, Ident, ItemTrait, LitStr, TraitBoundModifier, TypeParamBound};

pub(crate) fn expand(args: TraitArgs, mut input: ItemTrait, mode: Mode) -> TokenStream {
    if mode.de && !input.generics.params.is_empty() {
        let msg = "deserialization of generic traits is not supported yet; \
                   use #[inox_serializable::serialize] to generate serialization only";
        return Error::new_spanned(input.generics, msg).to_compile_error();
    }

    augment_trait(&mut input, mode);

    let (serialize_impl, deserialize_impl) = match args {
        TraitArgs::External => externally_tagged(&input),
        TraitArgs::Internal { tag } => internally_tagged(tag, &input),
        TraitArgs::Adjacent { tag, content } => adjacently_tagged(tag, content, &input),
    };

    let object = &input.ident;

    let mut expanded = TokenStream::new();

    expanded.extend(quote! {
        type SerializableInheritTrait = <dyn #object as inox_serializable::InheritTrait>::Object;
        type SerializableFn = inox_serializable::DeserializeFn<SerializableInheritTrait>;

    });

    if mode.ser {
        let mut impl_generics = input.generics.clone();
        impl_generics.params.push(parse_quote!('inox_serialize));
        let (impl_generics, _, _) = impl_generics.split_for_impl();
        let (_, ty_generics, where_clause) = input.generics.split_for_impl();

        expanded.extend(quote! {
            impl #impl_generics inox_serializable::serde::Serialize
            for dyn #object #ty_generics + 'inox_serialize #where_clause {
                fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
                where
                    S: inox_serializable::serde::Serializer,
                {
                    #serialize_impl
                }
            }
        });

        for marker_traits in &[quote!(Send), quote!(Sync), quote!(Send + Sync)] {
            expanded.extend(quote! {
                impl #impl_generics inox_serializable::serde::Serialize
                for dyn #object #ty_generics + #marker_traits + 'inox_serialize #where_clause {
                    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
                    where
                        S: inox_serializable::serde::Serializer,
                    {
                        inox_serializable::serde::Serialize::serialize(self as &dyn #object #ty_generics, serializer)
                    }
                }
            });
        }
    }

    if mode.de {
        let is_send = has_supertrait(&input, "Send");
        let is_sync = has_supertrait(&input, "Sync");
        let (inherit, others) = match (is_send, is_sync) {
            (false, false) => (quote!(), vec![]),
            (true, false) => (quote!(Send), vec![quote!()]),
            (false, true) => (quote!(Sync), vec![quote!()]),
            (true, true) => (
                quote!(Send + Sync),
                vec![quote!(), quote!(Send), quote!(Sync)],
            ),
        };

        expanded.extend(quote! {
            impl inox_serializable::InheritTrait for dyn #object {
                type Object = dyn #object + #inherit;
            }

            impl<'de> inox_serializable::serde::Deserialize<'de> for std::boxed::Box<dyn #object + #inherit> {
                fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
                where
                    D: inox_serializable::serde::Deserializer<'de>,
                {
                    #deserialize_impl
                }
            }
        });

        for marker_traits in others {
            expanded.extend(quote! {
                impl<'de> inox_serializable::serde::Deserialize<'de> for std::boxed::Box<dyn #object + #marker_traits> {
                    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
                    where
                        D: inox_serializable::serde::Deserializer<'de>,
                    {
                        std::result::Result::Ok(
                            <std::boxed::Box<dyn #object + #inherit>
                                as inox_serializable::serde::Deserialize<'de>>::deserialize(deserializer)?
                        )
                    }
                }
            });
        }
    }

    wrap_in_dummy_const(input, expanded)
}

fn augment_trait(input: &mut ItemTrait, mode: Mode) {
    input.items.push(parse_quote! {
        fn register_as_serializable(registry: &inox_serializable::SerializableRegistryRc) where Self: Sized;
    });
    input.items.push(parse_quote! {
        fn unregister_as_serializable(registry: &inox_serializable::SerializableRegistryRc) where Self: Sized;
    });

    if mode.ser {
        input
            .supertraits
            .push(parse_quote!(inox_serializable::Serialize));

        input.items.push(parse_quote! {
            #[doc(hidden)]
            fn serializable_name(&self) -> &'static str;
        });
    }

    if mode.de {
        input
            .supertraits
            .push(parse_quote!(inox_serializable::Deserialize));
    }
}

fn externally_tagged(input: &ItemTrait) -> (TokenStream, TokenStream) {
    let object = &input.ident;
    let object_name = object.to_string();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    let serialize_impl = quote! {
        let name = <Self as #object #ty_generics>::serializable_name(self);
        inox_serializable::externally::serialize(serializer, name, self)
    };

    let deserialize_impl = quote! {
        unsafe {
            if let Some(serializable_registry) = inox_serializable::SERIALIZABLE_REGISTRY.as_ref() {
                serializable_registry.read().unwrap().deserialize::<SerializableInheritTrait, D>(deserializer,
                        inox_serializable::DeserializeType::External {
                            trait_object: #object_name
                        })
            } else {
                panic!("inox_serializable::SERIALIZABLE_REGISTRY for externally_tagged is not set");
            }
        }
    };

    (serialize_impl, deserialize_impl)
}

fn internally_tagged(tag: LitStr, input: &ItemTrait) -> (TokenStream, TokenStream) {
    let object = &input.ident;
    let object_name = object.to_string();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    let serialize_impl = quote! {
        let name = <Self as #object #ty_generics>::serializable_name(self);
        inox_serializable::internally::serialize(serializer, #tag, name, self)
    };
    let deserialize_impl = quote! {
        unsafe {
            if let Some(serializable_registry) = inox_serializable::SERIALIZABLE_REGISTRY.as_ref() {
                serializable_registry.read().unwrap().deserialize::<SerializableInheritTrait, D>(deserializer,
            inox_serializable::DeserializeType::Internal {
                trait_object: #object_name,
                tag: #tag,
            })
        } else {
            panic!("inox_serializable::SERIALIZABLE_REGISTRY for internally_tagged is not set");
        }
    }
    };

    (serialize_impl, deserialize_impl)
}

fn adjacently_tagged(
    tag: LitStr,
    content: LitStr,
    input: &ItemTrait,
) -> (TokenStream, TokenStream) {
    let object = &input.ident;
    let object_name = object.to_string();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    let serialize_impl = quote! {
        let name = <Self as #object #ty_generics>::serializable_name(self);
        inox_serializable::adjacently::serialize(serializer, #object_name, #tag, name, #content, self)
    };

    let deserialize_impl = quote! {
        unsafe {
            if let Some(serializable_registry) = inox_serializable::SERIALIZABLE_REGISTRY.as_ref() {
                serializable_registry.read().unwrap().deserialize::<SerializableInheritTrait, D>(deserializer,
            inox_serializable::DeserializeType::Adjacent {
                trait_object: #object_name,
                fields: &[#tag, #content],
            })
        } else {
            panic!("inox_serializable::SERIALIZABLE_REGISTRY for adjacently_tagged is not set");
        }
    }
    };

    (serialize_impl, deserialize_impl)
}

fn has_supertrait(input: &ItemTrait, find: &str) -> bool {
    for supertrait in &input.supertraits {
        if let TypeParamBound::Trait(trait_bound) = supertrait {
            if let TraitBoundModifier::None = trait_bound.modifier {
                if trait_bound.path.is_ident(find) {
                    return true;
                }
            }
        }
    }
    false
}

fn wrap_in_dummy_const(input: ItemTrait, expanded: TokenStream) -> TokenStream {
    let dummy_const_name = format!("_{}_registry", input.ident);
    let dummy_const = Ident::new(&dummy_const_name, Span::call_site());

    quote! {
        #input

        #[allow(non_upper_case_globals)]
        const #dummy_const: () = {
            #expanded
        };
    }
}
