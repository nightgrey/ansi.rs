use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Error, Meta};

/// Derive the `Maybe` trait for an enum.
///
/// # None resolution (in order)
/// 1. A variant marked `#[none]` — must be a unit variant.
/// 2. A variant named `None` — must be a unit variant.
/// 3. Neither → compile error.
///
/// # Default
/// Always emits `impl Default`. The default variant is:
/// 1. The variant marked `#[maybe(default)]`, if any.
/// 2. Otherwise the none variant.
///
/// Do **not** also `#[derive(Default)]` — this will conflict.
///
/// # Generated impls
/// - `impl Maybe for T`
/// - `impl Default for T`
/// - `impl From<T> for Option<T>`
/// - `impl From<Option<T>> for T`
///
/// # Examples
/// ```rust
/// // Zero-config: variant named `None` is auto-detected
/// #[derive(Maybe)]
/// enum Color {
///     None,
///     Black,
///     Red,
/// }
///
/// // Explicit none + separate default
/// #[derive(Maybe)]
/// enum Weight {
///     #[none]
///     Unset,
///     #[maybe(default)]
///     Normal,
///     Bold,
/// }
/// ```
#[proc_macro_derive(Maybe, attributes(none, maybe))]
pub fn derive_maybe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_maybe_inner(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn has_attr(attrs: &[syn::Attribute], ident: &str) -> bool {
    attrs.iter().any(|a| a.path().is_ident(ident))
}

/// Check for `#[maybe(default)]`
fn has_maybe_default(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        if !a.path().is_ident("maybe") {
            return false;
        }
        match a.meta {
            Meta::List(ref list) => {
                list.parse_args::<syn::Ident>()
                    .map(|id| id == "default")
                    .unwrap_or(false)
            }
            _ => false,
        }
    })
}

fn derive_maybe_inner(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let (impl_generic, generic, where_clause) = input.generics.split_for_impl();

    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => return Err(Error::new_spanned(name, "Maybe can only be derived for enums")),
    };

    // --- Resolve none ---

    let attr_none: Vec<_> = variants
        .iter()
        .filter(|v| has_attr(&v.attrs, "none"))
        .collect();

    let none_variant = match attr_none.len() {
        0 => {
            variants
                .iter()
                .find(|v| v.ident == "None")
                .ok_or_else(|| Error::new_spanned(
                    name,
                    "No #[none] attribute and no variant named `None`. \
                     Mark a unit variant with #[none] or name one `None`.",
                ))?
        }
        1 => attr_none[0],
        _ => {
            return Err(Error::new_spanned(
                &attr_none[1].ident,
                "Multiple #[none] variants — mark exactly one",
            ))
        }
    };

    if !matches!(none_variant.fields, Fields::Unit) {
        return Err(Error::new_spanned(
            &none_variant.ident,
            "The none variant must be a unit variant (no fields)",
        ));
    }

    let none_ident = &none_variant.ident;

    // --- Resolve default ---

    let attr_default: Vec<_> = variants
        .iter()
        .filter(|v| has_maybe_default(&v.attrs))
        .collect();

    let default_ident = match attr_default.len() {
        0 => none_ident,
        1 => {
            let v = attr_default[0];
            if !matches!(v.fields, Fields::Unit) {
                return Err(Error::new_spanned(
                    &v.ident,
                    "#[maybe(default)] variant must be a unit variant (no fields)",
                ));
            }
            &v.ident
        }
        _ => {
            return Err(Error::new_spanned(
                &attr_default[1].ident,
                "Multiple #[maybe(default)] variants — mark at most one",
            ))
        }
    };

    Ok(quote! {
        impl #impl_generic Maybe for #name #generic #where_clause {
            #[allow(non_upper_case_globals)]
            const None: Self = #name::#none_ident;

            #[inline]
            fn is_none(&self) -> bool {
                matches!(self, #name::#none_ident)
            }
        }

        
        impl #impl_generic From<Option<#name #generic >> for #name #generic #where_clause {
            #[inline]
            fn from(value: Option<#name #generic>) -> Self {
                match value {
                    Some(value) => if value.is_some() { value } else { <#name #generic as Maybe>::None },
                    None => <#name #generic as Maybe>::None,
                }
            }
        }

        impl #impl_generic PartialEq<Option<#name #generic>> for #name #generic #where_clause {
            #[inline]
            fn eq(&self, other: &Option<#name #generic>) -> bool {
                match other {
                    Some(rhs) => rhs == self,
                    None => &Self::None == self,
                }
            }
        }

        impl #impl_generic PartialEq<#name #generic> for Option<#name #generic> #where_clause {
            #[inline]
            fn eq(&self, other: &#name #generic) -> bool {
                match self {
                    Some(lhs) => lhs == other,
                    None => &#name #generic::None == other,
                }
            }
        }

    })
}