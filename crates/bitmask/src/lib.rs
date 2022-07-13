use proc_macro::TokenStream;
use syn::{parse_macro_input, Ident, ItemEnum};

#[proc_macro_attribute]
pub fn bitmask(attr: TokenStream, item: TokenStream) -> TokenStream {
    let typ = quote::format_ident!("{}", typ(attr));

    let item = parse_macro_input!(item as ItemEnum);
    let vis = item.vis.clone();
    let (ident, idents, exprs) = enm(item);

    let enm = quote::quote! {
        #[derive(Default, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[serde(crate = "inox_serialize")]
        #vis struct #ident {
            bits: #typ,
        }

        #[allow(non_upper_case_globals)]
        impl #ident {
            #(#vis const #idents: #ident = #exprs;)*

            /// returns the underlying bits
            #[inline]
            #vis const fn bits(&self) -> #typ {
                self.bits
            }

            /// contains all values
            #[inline]
            #vis const fn all() -> Self {
                Self { bits: !0 }
            }

            /// if self contains all values
            #[inline]
            #vis const fn is_all(&self) -> bool {
                self.bits == !0
            }

            /// contains no value
            #[inline]
            #vis const fn none() -> Self {
                Self { bits: 0 }
            }

            /// if self contains no value
            #[inline]
            #vis const fn is_none(&self) -> bool {
                self.bits == 0
            }

            /// self intersects one of the other
            /// `(self & other) != 0 || other == 0`
            #[inline]
            #vis const fn intersects(&self, other: Self) -> bool {
                (self.bits & other.bits) != 0 || other.bits == 0
            }

            /// self contains all of the other
            /// `(self & other) == other`
            #[inline]
            #vis const fn contains(&self, other: Self) -> bool {
                (self.bits & other.bits) == other.bits
            }

            /// constant bitwise not
            #[inline]
            #vis const fn not(self) -> Self {
                Self { bits: !self.bits }
            }

            /// constant bitwise and
            #[inline]
            #vis const fn and(self, other: Self) -> Self {
                Self { bits: self.bits & other.bits }
            }

            /// constant bitwise or
            #[inline]
            #vis const fn or(self, other: Self) -> Self {
                Self { bits: self.bits | other.bits }
            }

            /// constant bitwise xor
            #[inline]
            #vis const fn xor(self, other: Self) -> Self {
                Self { bits: self.bits ^ other.bits }
            }
        }

        impl core::ops::Not for #ident {
            type Output = Self;
            #[inline]
            fn not(self) -> Self::Output {
                Self { bits: self.bits.not() }
            }
        }

        impl core::ops::BitAnd for #ident {
            type Output = Self;
            #[inline]
            fn bitand(self, rhs: Self) -> Self::Output {
                Self { bits: self.bits.bitand(rhs.bits) }
            }
        }

        impl core::ops::BitAndAssign for #ident {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self){
                self.bits.bitand_assign(rhs.bits)
            }
        }

        impl core::ops::BitOr for #ident {
            type Output = Self;
            #[inline]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self { bits: self.bits.bitor(rhs.bits) }
            }
        }

        impl core::ops::BitOrAssign for #ident {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self){
                self.bits.bitor_assign(rhs.bits)
            }
        }

        impl core::ops::BitXor for #ident {
            type Output = Self;
            #[inline]
            fn bitxor(self, rhs: Self) -> Self::Output {
                Self { bits: self.bits.bitxor(rhs.bits) }
            }
        }

        impl core::ops::BitXorAssign for #ident {
            #[inline]
            fn bitxor_assign(&mut self, rhs: Self){
                self.bits.bitxor_assign(rhs.bits)
            }
        }

        impl From<#typ> for #ident {
            #[inline]
            fn from(val: #typ) -> Self {
                Self { bits: val }
            }
        }

        impl From<#ident> for #typ {
            #[inline]
            fn from(val: #ident) -> #typ {
                val.bits
            }
        }

        impl PartialEq<#typ> for #ident {
            #[inline]
            fn eq(&self, other: &#typ) -> bool {
                self.bits == *other
            }
        }

        impl core::fmt::Binary for #ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.bits.fmt(f)
            }
        }

        impl core::fmt::LowerHex for #ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.bits.fmt(f)
            }
        }

        impl core::fmt::UpperHex for #ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.bits.fmt(f)
            }
        }

        impl core::fmt::Octal for #ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.bits.fmt(f)
            }
        }
    };

    TokenStream::from(enm)
}

fn typ(attr: TokenStream) -> String {
    if attr.is_empty() {
        "u32".to_owned()
    } else {
        let typ = attr.into_iter().next().expect("unreachable").to_string();
        match typ.as_str() {
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => (),
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => (),
            _ => panic!("type can only be an (un)signed integer."),
        }
        typ
    }
}

fn enm(item: ItemEnum) -> (Ident, Vec<Ident>, Vec<impl quote::ToTokens>) {
    let ident = item.ident;

    let mut has_vals: Option<bool> = None;

    let mut idents = Vec::with_capacity(item.variants.len());
    let mut exprs = Vec::with_capacity(item.variants.len());
    item.variants.iter().enumerate().for_each(|(i, v)| {
        idents.push(v.ident.clone());

        let hv = has_vals.unwrap_or_else(|| {
            let hv = v.discriminant.is_some();
            has_vals.replace(hv);
            hv
        });

        assert!(
            hv == v.discriminant.is_some(),
            "the bitmask can either have assigned or default values, not both."
        );

        if hv {
            let (_, ref expr) = v.discriminant.as_ref().expect("unreachable");
            exprs.push(quote::quote!(Self { bits: #expr }));
        } else {
            exprs.push(quote::quote!(Self { bits: 1 << #i }));
        }
    });

    (ident, idents, exprs)
}
