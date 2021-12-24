extern crate proc_macro;
extern crate syn;

mod serializable_trait;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Comma, Paren, Where},
    Data, DataEnum, DataStruct, DeriveInput, Field, Fields, Generics, Ident, Index, Member, Meta,
    NestedMeta, Token, Variant,
};

#[derive(Default)]
struct PropAttributeArgs {
    pub ignore: Option<bool>,
}

#[derive(Clone)]
enum TraitImpl {
    NotImplemented,
    Implemented,
    Custom(Ident),
}

impl Default for TraitImpl {
    fn default() -> Self {
        Self::NotImplemented
    }
}

enum DeriveType {
    Struct,
    TupleStruct,
    UnitStruct,
    Enum,
    Value,
}

enum Item<'a> {
    Field(&'a Field),
    Variant(&'a Variant),
}

static SERIALIZABLE_ATTRIBUTE_NAME: &str = "serializable";
static SERIALIZABLE_VALUE_ATTRIBUTE_NAME: &str = "serializable_value";

#[proc_macro_derive(Serializable, attributes(serializable, serializable_value, module))]
pub fn derive_serializable(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let unit_struct_punctuated = Punctuated::<Field, Token![,]>::new();
    let (items, mut derive_type): (Vec<Item>, DeriveType) = match &ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => (
            fields.named.iter().map(Item::Field).collect(),
            DeriveType::Struct,
        ),
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(fields),
            ..
        }) => (
            fields.unnamed.iter().map(Item::Field).collect(),
            DeriveType::TupleStruct,
        ),
        Data::Struct(DataStruct {
            fields: Fields::Unit,
            ..
        }) => (
            unit_struct_punctuated.iter().map(Item::Field).collect(),
            DeriveType::UnitStruct,
        ),
        Data::Enum(DataEnum { variants, .. }) => (
            variants.iter().map(Item::Variant).collect(),
            DeriveType::Enum,
        ),
        _ => (
            unit_struct_punctuated.iter().map(Item::Field).collect(),
            DeriveType::Value,
        ),
    };

    let items_and_args = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            (
                item,
                match *item {
                    Item::Field(field) => &field.attrs,
                    Item::Variant(variant) => &variant.attrs,
                }
                .iter()
                .find(|a| *a.path.get_ident().as_ref().unwrap() == SERIALIZABLE_ATTRIBUTE_NAME)
                .map(|a| {
                    syn::custom_keyword!(ignore);
                    let mut attribute_args = PropAttributeArgs { ignore: None };
                    a.parse_args_with(|input: ParseStream| {
                        if input.parse::<Option<ignore>>()?.is_some() {
                            attribute_args.ignore = Some(true);
                            return Ok(());
                        }
                        Ok(())
                    })
                    .expect("Invalid 'property' attribute format.");

                    attribute_args
                }),
                i,
            )
        })
        .collect::<Vec<(&Item, Option<PropAttributeArgs>, usize)>>();
    let active_items = items_and_args
        .iter()
        .filter(|(_item, attrs, _i)| {
            attrs.is_none()
                || match attrs.as_ref().unwrap().ignore {
                    Some(ignore) => !ignore,
                    None => true,
                }
        })
        .map(|(item, _attr, i)| (*item, *i))
        .collect::<Vec<(&Item, usize)>>();
    let ignored_items = items_and_args
        .iter()
        .filter(|(_item, attrs, _i)| {
            attrs
                .as_ref()
                .map(|attrs| attrs.ignore.unwrap_or(false))
                .unwrap_or(false)
        })
        .map(|(item, _attr, i)| (*item, *i))
        .collect::<Vec<(&Item, usize)>>();

    let type_name = &ast.ident;

    let mut attrs = SerializableAttributes::default();
    let mut parent_traits = Vec::new();
    for attribute in ast.attrs.iter().filter_map(|attr| attr.parse_meta().ok()) {
        let meta_list = if let Meta::List(meta_list) = attribute {
            meta_list
        } else {
            continue;
        };

        if let Some(ident) = meta_list.path.get_ident() {
            if ident == SERIALIZABLE_ATTRIBUTE_NAME {
                attrs = SerializableAttributes::from_nested_metas(
                    &meta_list.nested,
                    Some(&mut parent_traits),
                );
            } else if ident == SERIALIZABLE_VALUE_ATTRIBUTE_NAME {
                derive_type = DeriveType::Value;
                attrs = SerializableAttributes::from_nested_metas(&meta_list.nested, None);
            }
        }
    }

    let registration_data = &attrs.data;
    let get_type_registration_impl = impl_type_info(type_name, registration_data, &ast.generics);
    let get_parentr_trait_registration_impl =
        impl_as_serializable_trait_info(type_name, &parent_traits, &ast.generics);

    match derive_type {
        DeriveType::Struct | DeriveType::UnitStruct => {
            let active_fields = active_items
                .iter()
                .map(|(item, i)| {
                    (
                        match *item {
                            Item::Field(field) => *field,
                            Item::Variant(_) => {
                                unreachable!()
                            }
                        },
                        *i,
                    )
                })
                .collect::<Vec<_>>();
            let ignored_fields = ignored_items
                .iter()
                .map(|(item, i)| {
                    (
                        match *item {
                            Item::Field(field) => *field,
                            Item::Variant(_) => {
                                unreachable!()
                            }
                        },
                        *i,
                    )
                })
                .collect::<Vec<_>>();
            impl_struct(
                type_name,
                &ast.generics,
                get_type_registration_impl,
                get_parentr_trait_registration_impl,
                &attrs,
                &active_fields,
                &ignored_fields,
            )
        }
        DeriveType::TupleStruct => {
            let active_fields = active_items
                .iter()
                .map(|(item, i)| {
                    (
                        match *item {
                            Item::Field(field) => *field,
                            Item::Variant(_) => {
                                unreachable!()
                            }
                        },
                        *i,
                    )
                })
                .collect::<Vec<_>>();
            let ignored_fields = ignored_items
                .iter()
                .map(|(item, i)| {
                    (
                        match *item {
                            Item::Field(field) => *field,
                            Item::Variant(_) => {
                                unreachable!()
                            }
                        },
                        *i,
                    )
                })
                .collect::<Vec<_>>();
            impl_tuple_struct(
                type_name,
                &ast.generics,
                get_type_registration_impl,
                get_parentr_trait_registration_impl,
                &attrs,
                &active_fields,
                &ignored_fields,
            )
        }
        DeriveType::Enum => {
            let active_variants = active_items
                .iter()
                .map(|(item, i)| {
                    (
                        match *item {
                            Item::Field(_) => unreachable!(),
                            Item::Variant(variant) => *variant,
                        },
                        *i,
                    )
                })
                .collect::<Vec<_>>();
            let ignored_variants = ignored_items
                .iter()
                .map(|(item, i)| {
                    (
                        match *item {
                            Item::Field(_) => unreachable!(),
                            Item::Variant(variant) => *variant,
                        },
                        *i,
                    )
                })
                .collect::<Vec<_>>();
            impl_enum(
                type_name,
                &ast.generics,
                get_type_registration_impl,
                &attrs,
                &active_variants,
                &ignored_variants,
            )
        }
        DeriveType::Value => {
            impl_value(type_name, &ast.generics, get_type_registration_impl, &attrs)
        }
    }
}

fn impl_struct(
    struct_name: &Ident,
    generics: &Generics,
    type_info_impl: proc_macro2::TokenStream,
    parent_trait_info_impl: proc_macro2::TokenStream,
    attrs: &SerializableAttributes,
    active_fields: &[(&Field, usize)],
    ignored_fields: &[(&Field, usize)],
) -> TokenStream {
    let field_names = active_fields
        .iter()
        .map(|(field, index)| {
            field
                .ident
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or_else(|| index.to_string())
        })
        .collect::<Vec<String>>();
    let field_types = active_fields
        .iter()
        .map(|(field, _index)| field.ty.clone())
        .collect::<Vec<_>>();
    let field_idents = active_fields
        .iter()
        .map(|(field, index)| {
            field
                .ident
                .as_ref()
                .map(|ident| Member::Named(ident.clone()))
                .unwrap_or_else(|| Member::Unnamed(Index::from(*index)))
        })
        .collect::<Vec<_>>();
    let fields_count = active_fields.len();
    let field_indices = (0..fields_count).collect::<Vec<usize>>();
    let ignored_field_idents = ignored_fields
        .iter()
        .map(|(field, index)| {
            field
                .ident
                .as_ref()
                .map(|ident| Member::Named(ident.clone()))
                .unwrap_or_else(|| Member::Unnamed(Index::from(*index)))
        })
        .collect::<Vec<_>>();

    let hash_fn = attrs.get_hash_impl();
    let serialize_fn = attrs.get_serialize_impl();
    let partial_eq_fn = match attrs.equal_trait {
        TraitImpl::NotImplemented => quote! {
            use SerializableStruct;
            Some(is_struct_equal(self, value))
        },
        TraitImpl::Implemented | TraitImpl::Custom(_) => attrs.get_partial_eq_impl(),
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut where_from_serializable_clause = if where_clause.is_some() {
        quote! {#where_clause}
    } else if fields_count > 0 {
        quote! {where}
    } else {
        quote! {}
    };
    where_from_serializable_clause.extend(quote! {
        #(#field_types: FromSerializable,)*
    });

    TokenStream::from(quote! {
        #type_info_impl

        #parent_trait_info_impl

        impl #impl_generics SerializableStruct for #struct_name #ty_generics #where_clause {
            fn field(&self, name: &str) -> Option<&dyn Serializable> {
                match name {
                    #(#field_names => Some(&self.#field_idents),)*
                    _ => None,
                }
            }

            fn field_mut(&mut self, name: &str) -> Option<&mut dyn Serializable> {
                match name {
                    #(#field_names => Some(&mut self.#field_idents),)*
                    _ => None,
                }
            }

            fn field_at(&self, index: usize) -> Option<&dyn Serializable> {
                match index {
                    #(#field_indices => Some(&self.#field_idents),)*
                    _ => None,
                }
            }

            fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Serializable> {
                match index {
                    #(#field_indices => Some(&mut self.#field_idents),)*
                    _ => None,
                }
            }

            fn name_at(&self, index: usize) -> Option<&str> {
                match index {
                    #(#field_indices => Some(#field_names),)*
                    _ => None,
                }
            }

            fn fields_count(&self) -> usize {
                #fields_count
            }

            fn iter_fields(&self) -> SerializableFieldIterator {
                SerializableFieldIterator::new(self)
            }

            fn clone_as_dynamic(&self) -> SerializableDynamicStruct {
                let mut dynamic = SerializableDynamicStruct::default();
                dynamic.set_name(self.type_name().to_string());
                #(dynamic.insert_boxed(#field_names, self.#field_idents.duplicate());)*
                dynamic
            }
        }

        impl #impl_generics Serializable for #struct_name #ty_generics #where_clause {
            #[inline]
            fn type_name(&self) -> String {
                std::any::type_name::<Self>().to_string()
            }

            #[inline]
            fn as_serializable(&self) -> &dyn Serializable {
                self
            }

            #[inline]
            fn as_serializable_mut(&mut self) -> &mut dyn Serializable {
                self
            }

            #[inline]
            fn any(&self) -> &dyn std::any::Any {
                self
            }
            #[inline]
            fn any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            #[inline]
            fn duplicate(&self) -> Box<dyn Serializable> {
                use SerializableStruct;
                Box::new(self.clone_as_dynamic())
            }

            #[inline]
            fn set(&mut self, value: &dyn Serializable, registry: &SerializableRegistry) {
                use SerializableStruct;
                if let SerializableRef::Struct(struct_value) = value.serializable_ref() {
                    for (i, value) in struct_value.iter_fields().enumerate() {
                        let name = struct_value.name_at(i).unwrap();
                        self.field_mut(name).map(|v| v.set(value, registry));
                    }
                } else {
                    panic!("Attempted to set non-struct type to struct type.");
                }
            }

            fn serializable_ref(&self) -> SerializableRef {
                SerializableRef::Struct(self)
            }

            fn serializable_mut(&mut self) -> SerializableMut {
                SerializableMut::Struct(self)
            }

            fn compute_hash(&self) -> Option<u64> {
                #hash_fn
            }

            fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
                #partial_eq_fn
            }

            fn serializable_value(&self) -> Option<SerializableValue> {
                #serialize_fn
            }
        }

        impl #impl_generics FromSerializable for #struct_name #ty_generics #where_from_serializable_clause {
            fn from_serializable(value: &dyn Serializable, registry: &SerializableRegistry) -> Option<Self> {
                use SerializableStruct;
                if let SerializableRef::Struct(ref_struct) = value.serializable_ref() {
                    Some(
                        Self{
                            #(#field_idents: {
                                <#field_types as FromSerializable>::from_serializable(ref_struct.field(#field_names)?, registry)?
                            },)*
                            #(#ignored_field_idents: Default::default(),)*
                        }
                    )
                } else {
                    None
                }
            }
        }
    })
}

fn tuple_field_name(i: usize) -> String {
    format!("t{}", i)
}

fn tuple_field_ident(i: usize) -> Ident {
    Ident::new(tuple_field_name(i).as_str(), Span::call_site())
}

fn impl_tuple_struct(
    struct_name: &Ident,
    generics: &Generics,
    type_info_impl: proc_macro2::TokenStream,
    parent_trait_info_impl: proc_macro2::TokenStream,
    attrs: &SerializableAttributes,
    active_fields: &[(&Field, usize)],
    ignored_fields: &[(&Field, usize)],
) -> TokenStream {
    let field_idents = active_fields
        .iter()
        .map(|(_field, index)| Member::Unnamed(Index::from(*index)))
        .collect::<Vec<_>>();
    let field_types = active_fields
        .iter()
        .map(|(field, _index)| field.ty.clone())
        .collect::<Vec<_>>();
    let fields_count = active_fields.len();
    let field_indices = (0..fields_count).collect::<Vec<usize>>();
    let ignored_field_idents = ignored_fields
        .iter()
        .map(|(_field, index)| Member::Unnamed(Index::from(*index)))
        .collect::<Vec<_>>();

    let hash_fn = attrs.get_hash_impl();
    let serialize_fn = attrs.get_serialize_impl();
    let partial_eq_fn = match attrs.equal_trait {
        TraitImpl::NotImplemented => quote! {
            use SerializableTupleStruct;
            Some(is_tuple_struct_equal(self, value))
        },
        TraitImpl::Implemented | TraitImpl::Custom(_) => attrs.get_partial_eq_impl(),
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut where_from_serializable_clause = if where_clause.is_some() {
        quote! {#where_clause}
    } else if fields_count > 0 {
        quote! {where}
    } else {
        quote! {}
    };
    where_from_serializable_clause.extend(quote! {
        #(#field_types: FromSerializable,)*
    });

    TokenStream::from(quote! {
        #type_info_impl

        #parent_trait_info_impl

        impl #impl_generics SerializableTupleStruct for #struct_name #ty_generics {
            fn field(&self, index: usize) -> Option<&dyn Serializable> {
                match index {
                    #(#field_indices => Some(&self.#field_idents),)*
                    _ => None,
                }
            }

            fn field_mut(&mut self, index: usize) -> Option<&mut dyn Serializable> {
                match index {
                    #(#field_indices => Some(&mut self.#field_idents),)*
                    _ => None,
                }
            }

            fn fields_count(&self) -> usize {
                #fields_count
            }

            fn iter_fields(&self) -> SerializableTupleStructFieldIterator {
                SerializableTupleStructFieldIterator::new(self)
            }

            fn clone_as_dynamic(&self) -> SerializableDynamicTupleStruct {
                let mut dynamic = SerializableDynamicTupleStruct::default();
                dynamic.set_name(self.type_name().to_string());
                #(dynamic.insert_boxed(self.#field_idents.duplicate());)*
                dynamic
            }
        }

        impl #impl_generics Serializable for #struct_name #ty_generics {
            #[inline]
            fn type_name(&self) -> String {
                std::any::type_name::<Self>().to_string()
            }

            #[inline]
            fn as_serializable(&self) -> &dyn Serializable {
                self
            }

            #[inline]
            fn as_serializable_mut(&mut self) -> &mut dyn Serializable {
                self
            }

            #[inline]
            fn any(&self) -> &dyn std::any::Any {
                self
            }
            #[inline]
            fn any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            #[inline]
            fn duplicate(&self) -> Box<dyn Serializable> {
                use SerializableTupleStruct;
                Box::new(self.clone_as_dynamic())
            }

            #[inline]
            fn set(&mut self, value: &dyn Serializable, registry: &SerializableRegistry) {
                use SerializableTupleStruct;
                if let SerializableRef::TupleStruct(struct_value) = value.serializable_ref() {
                    for (i, value) in struct_value.iter_fields().enumerate() {
                        self.field_mut(i).map(|v| v.set(value, registry));
                    }
                } else {
                    panic!("Attempted to set non-TupleStruct type to TupleStruct type.");
                }
            }

            fn serializable_ref(&self) -> SerializableRef {
                SerializableRef::TupleStruct(self)
            }

            fn serializable_mut(&mut self) -> SerializableMut {
                SerializableMut::TupleStruct(self)
            }

            fn compute_hash(&self) -> Option<u64> {
                #hash_fn
            }

            fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
                #partial_eq_fn
            }

            fn serializable_value(&self) -> Option<SerializableValue> {
                #serialize_fn
            }
        }

        impl #impl_generics FromSerializable for #struct_name #ty_generics #where_from_serializable_clause
        {
            fn from_serializable(value: &dyn Serializable, registry: &SerializableRegistry) -> Option<Self> {
                use SerializableTupleStruct;
                if let SerializableRef::TupleStruct(ref_tuple_struct) = value.serializable_ref() {
                    Some(
                        Self{
                            #(#field_idents:
                                <#field_types as FromSerializable>::from_serializable(ref_tuple_struct.field(#field_indices)?, registry)?
                            ,)*
                            #(#ignored_field_idents: Default::default(),)*
                        }
                    )
                } else {
                    None
                }
            }
        }
    })
}

fn impl_value(
    type_name: &Ident,
    generics: &Generics,
    type_info_impl: proc_macro2::TokenStream,
    attrs: &SerializableAttributes,
) -> TokenStream {
    let hash_fn = attrs.get_hash_impl();
    let partial_eq_fn = attrs.get_partial_eq_impl();
    let serialize_fn = attrs.get_serialize_impl();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    TokenStream::from(quote! {
        #type_info_impl

        impl #impl_generics Serializable for #type_name #ty_generics #where_clause  {
            #[inline]
            fn type_name(&self) -> String {
                std::any::type_name::<Self>().to_string()
            }

            #[inline]
            fn as_serializable(&self) -> &dyn Serializable {
                self
            }

            #[inline]
            fn as_serializable_mut(&mut self) -> &mut dyn Serializable {
                self
            }

            #[inline]
            fn any(&self) -> &dyn std::any::Any {
                self
            }

            #[inline]
            fn any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            #[inline]
            fn duplicate(&self) -> Box<dyn Serializable> {
                Box::new(self.clone())
            }

            #[inline]
            fn set(&mut self, value: &dyn Serializable, _registry: &SerializableRegistry) {
                let value = value.any();
                if let Some(value) = value.downcast_ref::<Self>() {
                    *self = value.clone();
                } else {
                    panic!("Value is not {}.", std::any::type_name::<Self>());
                }
            }

            fn serializable_ref(&self) -> SerializableRef {
                SerializableRef::Value(self)
            }

            fn serializable_mut(&mut self) -> SerializableMut {
                SerializableMut::Value(self)
            }

            fn compute_hash(&self) -> Option<u64> {
                #hash_fn
            }

            fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
                #partial_eq_fn
            }

            fn serializable_value(&self) -> Option<SerializableValue> {
                #serialize_fn
            }
        }

        impl #impl_generics FromSerializable for #type_name #ty_generics #where_clause  {
            fn from_serializable(value: &dyn Serializable, _registry: &SerializableRegistry) -> Option<Self> {
                Some(value.any().downcast_ref::<#type_name #ty_generics>()?.clone())
            }
        }
    })
}

fn impl_enum(
    enum_name: &Ident,
    generics: &Generics,
    get_type_registration_impl: proc_macro2::TokenStream,
    attrs: &SerializableAttributes,
    active_variants: &[(&Variant, usize)],
    _ignored_variants: &[(&Variant, usize)],
) -> TokenStream {
    let mut variant_indices = Vec::new();
    let mut struct_wrappers = Vec::new();
    let mut tuple_wrappers = Vec::new();
    let mut variant_names = Vec::new();
    let mut variant_idents = Vec::new();
    let mut serializable_variants = Vec::new();
    let mut serializable_variants_mut = Vec::new();
    let mut variant_with_fields_idents = Vec::new();
    let mut variant_without_fields_idents = Vec::new();
    for (variant, variant_index) in active_variants.iter() {
        let variant_ident = {
            let ident = &variant.ident;
            quote!(#enum_name::#ident)
        };
        let variant_name = variant_ident
            .to_string()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();
        let variant_without_fields_ident = {
            match &variant.fields {
                Fields::Named(_struct_fields) => {
                    quote!(#variant_ident {..})
                }
                Fields::Unnamed(tuple) => {
                    let tuple_fields = &tuple.unnamed;
                    if tuple_fields.len() == 1 {
                        quote!(#variant_ident (_))
                    } else {
                        quote!(#variant_ident (..))
                    }
                }
                Fields::Unit => {
                    quote!(#variant_ident)
                }
            }
        };
        let variant_with_fields_ident = {
            match &variant.fields {
                Fields::Named(struct_fields) => {
                    let field_idents = struct_fields
                        .named
                        .iter()
                        .map(|field| field.ident.as_ref().unwrap())
                        .collect::<Vec<_>>();
                    quote!(#variant_ident {#(#field_idents,)*})
                }
                Fields::Unnamed(tuple_fields) => {
                    let field_idents = (0..tuple_fields.unnamed.len())
                        .map(tuple_field_ident)
                        .collect::<Vec<_>>();
                    if tuple_fields.unnamed.len() == 1 {
                        quote!(#variant_ident (#(#field_idents,)*))
                    } else {
                        quote!(#variant_ident (#(#field_idents,)*))
                    }
                }
                Fields::Unit => {
                    quote!(#variant_ident)
                }
            }
        };
        let wrapper_ident = if let Fields::Named(_) | Fields::Unnamed(_) = &variant.fields {
            Ident::new(
                format!("{}{}Wrapper", enum_name, variant.ident).as_str(),
                Span::call_site(),
            )
        } else {
            Ident::new("unused", Span::call_site())
        };
        let wrapper_name = match &variant.fields {
            Fields::Named(struct_fields) => quote!(#struct_fields).to_string(),
            Fields::Unnamed(tuple_fields) => quote!(#tuple_fields).to_string(),
            Fields::Unit => "unused".to_string(),
        };
        let serializable_variant = {
            match &variant.fields {
                Fields::Named(_struct_fields) => {
                    quote!({
                        let wrapper_ref = unsafe { std::mem::transmute::< &Self, &#wrapper_ident >(self) };
                        SerializableEnumVariant::Struct(wrapper_ref as &dyn SerializableStruct)
                    })
                }
                Fields::Unnamed(_tuple_fields) => {
                    quote!({
                        let wrapper_ref = unsafe { std::mem::transmute::< &Self, &#wrapper_ident >(self) };
                        SerializableEnumVariant::Tuple(wrapper_ref as &dyn SerializableTuple)
                    })
                }
                Fields::Unit => {
                    quote!(SerializableEnumVariant::Unit)
                }
            }
        };
        let serializable_variant_mut = {
            match &variant.fields {
                Fields::Named(_struct_fields) => {
                    quote!({
                        let wrapper_ref = unsafe { std::mem::transmute::< &mut Self, &mut #wrapper_ident >(self) };
                        SerializableEnumVariantMut::Struct(wrapper_ref as &mut dyn SerializableStruct)
                    })
                }
                Fields::Unnamed(_tuple) => {
                    quote!({
                        let wrapper_ref = unsafe { std::mem::transmute::< &mut Self, &mut #wrapper_ident >(self) };
                        SerializableEnumVariantMut::Tuple(wrapper_ref as &mut dyn SerializableTuple)
                    })
                }
                Fields::Unit => {
                    quote!(SerializableEnumVariantMut::Unit)
                }
            }
        };
        match &variant.fields {
            Fields::Named(struct_fields) => {
                struct_wrappers.push((
                    wrapper_ident,
                    wrapper_name,
                    variant_index,
                    variant_with_fields_ident.clone(),
                    struct_fields.clone(),
                ));
            }
            Fields::Unnamed(tuple_fields) => {
                tuple_wrappers.push((
                    wrapper_ident,
                    wrapper_name,
                    variant_index,
                    variant_with_fields_ident.clone(),
                    tuple_fields.clone(),
                ));
            }
            Fields::Unit => {}
        }
        variant_indices.push(variant_index);
        variant_names.push(variant_name);
        variant_idents.push(variant_ident);
        serializable_variants.push(serializable_variant);
        serializable_variants_mut.push(serializable_variant_mut);
        variant_with_fields_idents.push(variant_with_fields_ident);
        variant_without_fields_idents.push(variant_without_fields_ident);
    }
    let hash_fn = attrs.get_hash_impl();
    let serialize_fn = attrs.get_serialize_impl();
    let partial_eq_fn = match attrs.equal_trait {
        TraitImpl::NotImplemented => quote! {
            use SerializableEnum;
            Some(is_enum_equal(self, value))
        },
        TraitImpl::Implemented | TraitImpl::Custom(_) => attrs.get_partial_eq_impl(),
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut token_stream = TokenStream::from(quote! {
        #get_type_registration_impl

        impl #impl_generics SerializableEnum for #enum_name #ty_generics #where_clause {
            fn variant(&self) -> SerializableEnumVariant<'_> {
                match self {
                    #(#variant_with_fields_idents => #serializable_variants,)*
                }
            }

            fn variant_mut(&mut self) -> SerializableEnumVariantMut<'_> {
                match self {
                    #(#variant_with_fields_idents => #serializable_variants_mut,)*
                }
            }

            fn variant_info(&self) -> SerializableVariantInfo<'_> {
                let index = match self {
                    #(#variant_without_fields_idents => #variant_indices,)*
                };
                SerializableVariantInfo {
                    index,
                    name: self.get_index_name(index).unwrap(),
                }
            }

            fn get_index_name(&self, index: usize) -> Option<&'_ str> {
                match index {
                    #(#variant_indices => Some(#variant_names),)*
                    _ => None,
                }
            }

            fn get_index_from_name(&self, name: &str) -> Option<usize> {
                match name {
                    #(#variant_names => Some(#variant_indices),)*
                    _ => None,
                }
            }

            fn iter_variants_info(&self) -> SerializableVariantInfoIterator<'_> {
                SerializableVariantInfoIterator::new(self)
            }

            fn clone_as_dynamic(&self) -> SerializableDynamicEnum {
                let mut dynamic = SerializableDynamicEnum::default();
                dynamic.set_name(self.type_name().to_string());
                #(match #serializable_variants {
                    SerializableEnumVariant::Unit => {
                        dynamic.insert_boxed(#variant_names, Box::new(#variant_indices));
                    },
                    SerializableEnumVariant::NewType(value) => {
                        dynamic.insert_boxed(#variant_names, value.duplicate());
                    },
                    SerializableEnumVariant::Tuple(tuple_value) => {
                        dynamic.insert_boxed(#variant_names, tuple_value.duplicate());
                    },
                    SerializableEnumVariant::Struct(struct_value) => {
                        dynamic.insert_boxed(#variant_names, struct_value.duplicate());
                    },
                })*
                let index = match self {
                    #(#variant_without_fields_idents => #variant_indices,)*
                };
                dynamic.set_variant_index(index);
                dynamic
            }
        }

        impl #impl_generics Serializable for #enum_name #ty_generics #where_clause {
            #[inline]
            fn type_name(&self) -> String {
                std::any::type_name::<Self>().to_string()
            }

            fn as_serializable(&self) -> &dyn Serializable {
                self
            }

            fn as_serializable_mut(&mut self) -> &mut dyn Serializable {
                self
            }

            #[inline]
            fn any(&self) -> &dyn std::any::Any {
                self
            }
            #[inline]
            fn any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
            #[inline]
            fn duplicate(&self) -> Box<dyn Serializable> {
                use SerializableEnum;
                Box::new(self.clone_as_dynamic())
            }

            #[inline]
            fn set(&mut self, value: &dyn Serializable, _registry: &SerializableRegistry) {
                use SerializableEnum;
                let v = value.any();
                if let Some(v) = v.downcast_ref::<Self>() {
                    *self = v.clone(); //FIXME: should apply the variant instead
                    todo!();
                } else {
                    panic!("Attempted to apply non-enum type {} to enum type {}", value.type_name(), self.type_name());
                }
            }

            fn serializable_ref(&self) -> SerializableRef {
                SerializableRef::Enum(self)
            }

            fn serializable_mut(&mut self) -> SerializableMut {
                SerializableMut::Enum(self)
            }

            fn serializable_value(&self) -> Option<SerializableValue> {
                #serialize_fn
            }

            fn compute_hash(&self) -> Option<u64> {
                #hash_fn
            }

            fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
                #partial_eq_fn
            }
        }

        impl #impl_generics FromSerializable for #enum_name #ty_generics #where_clause {
            fn from_serializable(
                value: &dyn Serializable,
                registry: &SerializableRegistry,
            ) -> Option<Self> {
                if let SerializableRef::Enum(ref_enum) = value.serializable_ref() {
                    //Todo: load the enum from the registry
                    todo!();
                } else {
                    None
                }
            }
        }
    });
    for (wrapper_ident, wrapper_name, _variant_index, variant_with_fields_ident, fields) in
        struct_wrappers
    {
        let mut field_names = Vec::new();
        let mut field_idents = Vec::new();
        let mut field_indices = Vec::new();
        for (i, field) in fields.named.iter().enumerate() {
            field_names.push(field.ident.as_ref().unwrap().to_string());
            field_idents.push(field.ident.clone());
            field_indices.push(i);
        }
        let fields_len = field_indices.len();
        let match_fields = quote!(
            #variant_with_fields_ident => (#(#field_idents,)*),
            _ => unreachable!(),
        );
        let match_fields_mut = quote!(let (#(#field_idents,)*) = match &mut self.0 {
            #match_fields
        };);
        let match_fields = quote!(let (#(#field_idents,)*) = match &self.0 {
            #match_fields
        };);
        token_stream.extend(TokenStream::from(quote! {
            #[repr(transparent)]
            pub struct #wrapper_ident(#enum_name);
            impl Serializable for #wrapper_ident {
                fn type_name(&self) -> String {
                    #wrapper_name.to_string()
                }

                fn as_serializable(&self) -> &dyn Serializable {
                    self
                }

                fn as_serializable_mut(&mut self) -> &mut dyn Serializable {
                    self
                }

                fn any(&self) -> &dyn std::any::Any {
                    self.0.any()
                }

                fn any_mut(&mut self) -> &mut dyn std::any::Any {
                    self.0.any_mut()
                }

                fn set(&mut self, value: &dyn Serializable) {
                    self.0.set(value);
                }

                fn serializable_ref(&self) -> SerializableRef {
                    SerializableRef::Struct(self)
                }

                fn serializable_mut(&mut self) -> SerializableMut {
                    SerializableMut::Struct(self)
                }

                fn duplicate(&self) -> Box<dyn Serializable> {
                    self.0.duplicate()
                }

                fn compute_hash(&self) -> Option<u64> {
                    self.0.compute_hash()
                }

                fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
                    self.0.is_equal(value)
                }

                fn serializable_value(&self) -> Option<SerializableValue> {
                    self.0.serializable_value()
                }
            }
            impl SerializableStruct for #wrapper_ident {
                fn field(&self, name: &str) -> Option<&dyn Serializable> {
                    #match_fields
                    match name {
                        #(#field_names => Some(#field_idents),)*
                        _ => None,
                    }
                }

                fn field_mut(&mut self, name: &str) -> Option<&mut dyn Serializable> {
                    #match_fields_mut
                    match name {
                        #(#field_names => Some(#field_idents),)*
                        _ => None,
                    }
                }

                fn field_at(&self, index: usize) -> Option<&dyn Serializable> {
                    #match_fields
                    match index {
                        #(#field_indices => Some(#field_idents),)*
                        _ => None,
                    }
                }

                fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Serializable> {
                    #match_fields_mut
                    match index {
                        #(#field_indices => Some(#field_idents),)*
                        _ => None,
                    }
                }
                fn name_at(&self, index: usize) -> Option<&str> {
                    match index {
                        #(#field_indices => Some(#field_names),)*
                        _ => None,
                    }
                }

                fn fields_count(&self) -> usize {
                    #fields_len
                }

                fn iter_fields(&self) -> SerializableFieldIterator {
                    SerializableFieldIterator::new(self)
                }

                fn clone_as_dynamic(&self) -> SerializableDynamicStruct {
                    #match_fields
                    let mut dynamic = SerializableDynamicStruct::default();
                    dynamic.set_name(self.type_name().to_string());
                    #(dynamic.insert_boxed(#field_names, #field_idents.duplicate());)*
                    dynamic
                }
            }
        }));
    }
    for (wrapper_ident, wrapper_name, _variant_index, variant_with_fields_ident, fields) in
        tuple_wrappers
    {
        let mut field_names = Vec::new();
        let mut field_idents = Vec::new();
        let mut field_indices = Vec::new();
        for (index, _field) in fields.unnamed.iter().enumerate() {
            field_names.push(tuple_field_name(index));
            field_idents.push(tuple_field_ident(index));
            field_indices.push(index);
        }
        let fields_len = field_indices.len();
        let match_fields = quote!(
            #variant_with_fields_ident => (#(#field_idents,)*),
            _ => unreachable!(),
        );
        let match_fields_mut = quote!(let (#(#field_idents,)*) = match &mut self.0 {
            #match_fields
        };);
        let match_fields = quote!(let (#(#field_idents,)*) = match &self.0 {
            #match_fields
        };);
        token_stream.extend(TokenStream::from(quote! {
            #[repr(transparent)]
            pub struct #wrapper_ident(#enum_name);
            impl Serializable for #wrapper_ident {
                fn type_name(&self) -> String {
                    #wrapper_name.to_string()
                }

                fn as_serializable(&self) -> &dyn Serializable {
                    self
                }

                fn as_serializable_mut(&mut self) -> &mut dyn Serializable {
                    self
                }

                fn any(&self) -> &dyn std::any::Any {
                    self.0.any()
                }

                fn any_mut(&mut self) -> &mut dyn std::any::Any {
                    self.0.any_mut()
                }

                fn set(&mut self, value: &dyn Serializable, registry: &SerializableRegistry) {
                    self.0.set(value, registry);
                }

                fn serializable_ref(&self) -> SerializableRef {
                    SerializableRef::Tuple(self)
                }

                fn serializable_mut(&mut self) -> SerializableMut {
                    SerializableMut::Tuple(self)
                }

                fn duplicate(&self) -> Box<dyn Serializable> {
                    self.0.duplicate()
                }

                fn compute_hash(&self) -> Option<u64> {
                    self.0.compute_hash()
                }

                fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
                    self.0.is_equal(value)
                }

                fn serializable_value(&self) -> Option<SerializableValue> {
                    self.0.serializable_value()
                }
            }
            impl SerializableTuple for #wrapper_ident {
                fn field(&self, index: usize) -> Option<&dyn Serializable> {
                    #match_fields
                    match index {
                        #(#field_indices => Some(#field_idents),)*
                        _ => None,
                    }
                }

                fn field_mut(&mut self, index: usize) -> Option<&mut dyn Serializable> {
                    #match_fields_mut
                    match index {
                        #(#field_indices => Some(#field_idents),)*
                        _ => None,
                    }
                }

                fn fields_count(&self) -> usize {
                    #fields_len
                }

                fn iter_fields(&self) -> SerializableTupleFieldIterator {
                    SerializableTupleFieldIterator::new(self)
                }

                fn clone_as_dynamic(&self) -> SerializableDynamicTuple {
                    #match_fields
                    let mut dynamic = SerializableDynamicTuple::default();
                    #(dynamic.insert_boxed(#field_idents.duplicate());)*
                    dynamic
                }
            }
        }));
    }
    token_stream
}

struct SerializableDef {
    type_name: Ident,
    generics: Generics,
    attrs: Option<SerializableAttributes>,
}

impl Parse for SerializableDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let type_ident = input.parse::<Ident>()?;
        let generics = input.parse::<Generics>()?;
        let mut lookahead = input.lookahead1();
        let mut where_clause = None;
        if lookahead.peek(Where) {
            where_clause = Some(input.parse()?);
            lookahead = input.lookahead1();
        }

        let mut attrs = None;
        if lookahead.peek(Paren) {
            let content;
            parenthesized!(content in input);
            attrs = Some(content.parse::<SerializableAttributes>()?);
        }

        Ok(SerializableDef {
            type_name: type_ident,
            generics: Generics {
                where_clause,
                ..generics
            },
            attrs,
        })
    }
}

#[proc_macro]
pub fn impl_serializable_value(input: TokenStream) -> TokenStream {
    let value_def = parse_macro_input!(input as SerializableDef);

    let ty = &value_def.type_name;
    let attrs = value_def.attrs.unwrap_or_default();
    let registration_data = &attrs.data;
    let type_info_impl = impl_type_info(ty, registration_data, &value_def.generics);
    impl_value(ty, &value_def.generics, type_info_impl, &attrs)
}

#[derive(Default)]
struct SerializableAttributes {
    hash: TraitImpl,
    equal_trait: TraitImpl,
    serialize: TraitImpl,
    data: Vec<Ident>,
}

impl SerializableAttributes {
    fn from_nested_metas(
        nested_metas: &Punctuated<NestedMeta, Comma>,
        mut parent_traits: Option<&mut Vec<Ident>>,
    ) -> Self {
        let mut attrs = SerializableAttributes::default();
        for nested_meta in nested_metas.iter() {
            match nested_meta {
                NestedMeta::Lit(_) => {}
                NestedMeta::Meta(meta) => match meta {
                    Meta::Path(path) => {
                        if let Some(segment) = path.segments.iter().next() {
                            let ident = segment.ident.to_string();
                            match ident.as_str() {
                                "PartialEq" => attrs.equal_trait = TraitImpl::Implemented,
                                "Hash" => attrs.hash = TraitImpl::Implemented,
                                "Serialize" => attrs.serialize = TraitImpl::Implemented,
                                "Deserialize" => attrs.data.push(Ident::new(
                                    &format!("Serializable{}", segment.ident),
                                    Span::call_site(),
                                )),
                                _ => {
                                    attrs.data.push(Ident::new(
                                        &format!("Serializable{}", segment.ident),
                                        Span::call_site(),
                                    ));
                                    if let Some(parent_traits) = &mut parent_traits {
                                        parent_traits.push(segment.ident.clone());
                                    }
                                }
                            }
                        }
                    }
                    Meta::List(list) => {
                        let ident = if let Some(segment) = list.path.segments.iter().next() {
                            segment.ident.to_string()
                        } else {
                            continue;
                        };

                        if let Some(list_nested) = list.nested.iter().next() {
                            match list_nested {
                                NestedMeta::Meta(list_nested_meta) => match list_nested_meta {
                                    Meta::Path(path) => {
                                        if let Some(segment) = path.segments.iter().next() {
                                            match ident.as_str() {
                                                "PartialEq" => {
                                                    attrs.equal_trait =
                                                        TraitImpl::Custom(segment.ident.clone())
                                                }
                                                "Hash" => {
                                                    attrs.hash =
                                                        TraitImpl::Custom(segment.ident.clone())
                                                }
                                                "Serialize" => {
                                                    attrs.serialize =
                                                        TraitImpl::Custom(segment.ident.clone())
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    Meta::List(_) => {}
                                    Meta::NameValue(_) => {}
                                },
                                NestedMeta::Lit(_) => {}
                            }
                        }
                    }
                    Meta::NameValue(_) => {}
                },
            }
        }

        attrs
    }

    fn get_hash_impl(&self) -> proc_macro2::TokenStream {
        match &self.hash {
            TraitImpl::Implemented => quote! {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                Hash::hash(&std::any::Any::type_id(self), &mut hasher);
                Hash::hash(self, &mut hasher);
                Some(hasher.finish())
            },
            TraitImpl::Custom(impl_fn) => quote! {
                Some(#impl_fn(self))
            },
            TraitImpl::NotImplemented => quote! {
                None
            },
        }
    }

    fn get_partial_eq_impl(&self) -> proc_macro2::TokenStream {
        match &self.equal_trait {
            TraitImpl::Implemented => quote! {
                let value = value.any();
                if let Some(value) = value.downcast_ref::<Self>() {
                    Some(std::cmp::PartialEq::eq(self, value))
                } else {
                    Some(false)
                }
            },
            TraitImpl::Custom(impl_fn) => quote! {
                Some(#impl_fn(self, value))
            },
            TraitImpl::NotImplemented => quote! {
                None
            },
        }
    }

    fn get_serialize_impl(&self) -> proc_macro2::TokenStream {
        match &self.serialize {
            TraitImpl::Implemented => quote! {
                Some(SerializableValue::Ref(self))
            },
            TraitImpl::Custom(impl_fn) => quote! {
                Some(#impl_fn(self))
            },
            TraitImpl::NotImplemented => quote! {
                None
            },
        }
    }
}

impl Parse for SerializableAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let result = Punctuated::<NestedMeta, Comma>::parse_terminated(input)?;
        Ok(SerializableAttributes::from_nested_metas(&result, None))
    }
}

fn impl_type_info(
    type_name: &Ident,
    registration_data: &[Ident],
    generics: &Generics,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        #[allow(unused_mut)]
        impl #impl_generics TypeInfo for #type_name #ty_generics #where_clause {
            fn type_info() -> SerializableTypeInfo {
                let mut type_registration = SerializableTypeInfo::of::<#type_name #ty_generics>();
                #(type_registration.insert_with_type_id::<#registration_data>(std::any::TypeId::of::<#type_name #ty_generics>(), SerializableType::<#type_name #ty_generics>::from_type_to_serializable());)*
                #(type_registration.insert::<#registration_data>(SerializableType::<#type_name #ty_generics>::from_type_to_serializable());)*
                type_registration
            }
        }
    }
}

fn impl_as_serializable_trait_info(
    type_name: &Ident,
    parent_traits: &[Ident],
    generics: &Generics,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        #(impl #impl_generics AsSerializable<dyn #parent_traits> for #type_name #ty_generics #where_clause {
            fn into_type(self: Box<#type_name #ty_generics>) -> Box<dyn #parent_traits>
            #where_clause {
                self
            }
        })*
    }
}

#[proc_macro_attribute]
pub fn serializable_trait(args: TokenStream, input: TokenStream) -> TokenStream {
    serializable_trait::serializable_trait(args, input)
}
