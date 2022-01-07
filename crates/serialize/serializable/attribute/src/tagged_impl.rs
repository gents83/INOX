use crate::{ImplArgs, Mode};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Error, ItemImpl, Type, TypePath};

pub(crate) fn expand(args: ImplArgs, mut input: ItemImpl, mode: Mode) -> TokenStream {
    if mode.de && !input.generics.params.is_empty() {
        let msg = "deserialization of generic impls is not supported yet; \
                   use #[sabi_serializable::serialize] to generate serialization only";
        return Error::new_spanned(input.generics, msg).to_compile_error();
    }

    let name = match args.name {
        Some(name) => quote!(#name),
        None => match type_name(&input.self_ty) {
            Some(name) => quote!(#name),
            None => {
                let msg =
                    "use #[sabi_serializable::serde(name = \"...\")] to specify a unique name";
                return Error::new_spanned(&input.self_ty, msg).to_compile_error();
            }
        },
    };

    augment_impl(&mut input, &name, mode);

    let object = &input.trait_.as_ref().unwrap().1;
    let this = &input.self_ty;

    if mode.de {
        input.items.push(parse_quote! {
            fn register_as_serializable()
            where Self: Sized 
            {
                let func = (|deserializer| std::result::Result::Ok(
                    std::boxed::Box::new(
                        sabi_serializable::erased_serde::deserialize::<#this>(deserializer)?
                    ),
                )) as sabi_serializable::DeserializeFn<<dyn #object as sabi_serializable::InheritTrait>::Object>;
                
                let serializable_registry = sabi_serializable::get_serializable_registry!();
                serializable_registry.register_type::< <dyn #object as sabi_serializable::InheritTrait>::Object >(#name, func);  
            }
        });
        input.items.push(parse_quote! {
            fn unregister_as_serializable()
            where Self: Sized 
            {
                let serializable_registry = sabi_serializable::get_serializable_registry!();
                serializable_registry.unregister_type::< <dyn #object as sabi_serializable::InheritTrait>::Object >(#name);  
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

    if mode.de {
        input.items.push(parse_quote! {
            #[doc(hidden)]
            fn serializable_deserialize(&self) {}
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
