use crate::{ImplArgs, Mode};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Error, ItemImpl, Type, TypePath};

pub(crate) fn expand(args: ImplArgs, mut input: ItemImpl, mode: Mode) -> TokenStream {
    if mode.de && !input.generics.params.is_empty() {
        let msg = "deserialization of generic impls is not supported yet; \
                   use #[inox_serializable::serialize] to generate serialization only";
        return Error::new_spanned(input.generics, msg).to_compile_error();
    }

    let name = match args.name {
        Some(name) => quote!(#name),
        None => match type_name(&input.self_ty) {
            Some(name) => quote!(#name),
            None => {
                let msg =
                    "use #[inox_serializable::serde(name = \"...\")] to specify a unique name";
                return Error::new_spanned(&input.self_ty, msg).to_compile_error();
            }
        },
    };

    augment_impl(&mut input, &name, mode);

    let object = &input.trait_.as_ref().unwrap().1;
    let this = &input.self_ty;

    if mode.de {
        input.items.push(parse_quote! {
            fn register_as_serializable(registry: &inox_serializable::SerializableRegistryRc)
            where Self: Sized 
            {
                unsafe {
                    if inox_serializable::SERIALIZABLE_REGISTRY.is_none() {
                        inox_serializable::SERIALIZABLE_REGISTRY.replace(registry.clone());
                    }
                }
                let func = (|deserializer| std::result::Result::Ok(
                    std::boxed::Box::new(
                        inox_serializable::erased_serde::deserialize::<#this>(deserializer)?
                    ),
                )) as inox_serializable::DeserializeFn<<dyn #object as inox_serializable::InheritTrait>::Object>;
                
                registry.write().unwrap().register_type::< <dyn #object as inox_serializable::InheritTrait>::Object >(#name, func);  
            
            }
        });
        input.items.push(parse_quote! {
            fn unregister_as_serializable(registry: &inox_serializable::SerializableRegistryRc)
            where Self: Sized 
            {
                registry.write().unwrap().unregister_type::< <dyn #object as inox_serializable::InheritTrait>::Object >(#name);  
            }
        });
    }

    quote! {
        #input
    }
}

fn augment_impl(input: &mut ItemImpl, name: &TokenStream, mode: Mode) {
    if mode.ser {
        input.items.push(parse_quote! {
            #[doc(hidden)]
            fn serializable_name(&self) -> &'static str {
                #name
            }
        });
    }
}

fn type_name(mut ty: &Type) -> Option<String> {
    loop {
        match ty {
            Type::Path(TypePath { qself: None, path }) => {
                return Some(path.segments.last().unwrap().ident.to_string());
            }
            Type::Group(group) => {
                ty = &group.elem;
            }
            _ => return None,
        }
    }
}
