use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Fields, ItemEnum};
use heck::{ToShoutySnakeCase, ToSnakeCase};

struct Variant {
    tag_const: proc_macro2::Ident,
    constructor: TokenStream2,
    is: TokenStream2,
    get_method: Option<TokenStream2>,
    set_method: Option<TokenStream2>,
    debug_arm: TokenStream2,
}

struct Backing {
    ident: TokenStream2,
    width: u32,
}

impl Backing {
    fn new(input: &syn::Ident) -> Self {
        Self::try_new(&input).unwrap_or_else(|| panic!(
            "unsupported backing type {} — expected u8, u16, u32, u64, or u128",
            input,
        ))
    }

    fn try_new(input: &syn::Ident) -> Option<Self> {
        match input.to_string().as_str() {
            "u8" => Some(Self { width: 8, ident: quote!(u8) }),
            "u16" => Some(Self { width: 16, ident: quote!(u16) }),
            "u32" => Some(Self { width: 32, ident: quote!(u32) }),
            "u64" => Some(Self { width: 64, ident: quote!(u64) }),
            "u128" => Some(Self { width: 128, ident: quote!(u128) }),
            _ => None,
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Minimum bits to represent `count` distinct tags.
fn tags_width_needed(count: usize) -> u32 {
    if count <= 1 {
        return 0;
    }
    let mut bits = 0u32;
    let mut n = count - 1; // 0-indexed max tag value
    while n > 0 {
        bits += 1;
        n >>= 1;
    }
    bits
}

/// Smallest standard unsigned type that can hold `bits` bits.
fn smallest_uint_tokens(bits: u32) -> TokenStream2 {
    match bits {
        0..=8 => quote!(u8),
        9..=16 => quote!(u16),
        17..=32 => quote!(u32),
        33..=64 => quote!(u64),
        65..=128 => quote!(u128),
        _ => panic!("bit width {bits} exceeds u128"),
    }
}

struct FieldType {
    /// Type used in the public API (e.g. `u30` or `u16`).
    public_type: TokenStream2,
    /// Smallest standard uint that contains the value (e.g. `u32` for `u30`).
    storage_type: TokenStream2,
    /// Number of significant bits.
    declared_bits: u32,
    /// Whether this is an `arbitrary_int` pseudo-type (needs `.value()` / `::new()`).
    is_arbitrary: bool,
}

/// Inspect a field type. Standard types (`u8`..`u128`) pass through as-is.
/// Non-standard `uN` (e.g. `u30`) are treated as `arbitrary_int` types —
/// the public API uses the original type, with `.value()`/`::new()` for conversion.
fn parse_field_type(ty: &syn::Type, payload_bits: u32) -> FieldType {
    if let syn::Type::Path(tp) = ty {
        if tp.qself.is_none() && tp.path.segments.len() == 1 {
            let seg = &tp.path.segments[0];
            let name = seg.ident.to_string();
            if let Some(rest) = name.strip_prefix('u') {
                if let Ok(bits) = rest.parse::<u32>() {
                    // Standard widths → pass through as real types
                    if matches!(bits, 8 | 16 | 32 | 64 | 128) {
                        return FieldType {
                            public_type: quote!(#ty),
                            storage_type: quote!(#ty),
                            declared_bits: bits,
                            is_arbitrary: false,
                        };
                    }
                    // Non-standard → arbitrary_int pseudo-type
                    if bits > payload_bits {
                        panic!(
                            "u{bits} exceeds available payload of {payload_bits} bits"
                        );
                    }
                    let storage = smallest_uint_tokens(bits);
                    return FieldType {
                        public_type: quote!(#ty),
                        storage_type: storage,
                        declared_bits: bits,
                        is_arbitrary: true,
                    };
                }
            }
        }
    }
    // Anything else (custom type) — assume it fits, user's responsibility
    FieldType {
        public_type: quote!(#ty),
        storage_type: quote!(#ty),
        declared_bits: payload_bits,
        is_arbitrary: false,
    }
}

fn process_variant(
    idx: usize,
    variant: &syn::Variant,
    backing: &Backing,
    enum_ident: &syn::Ident,
    payload_width: u32,
) -> Variant {
    let backing_ident = &backing.ident;
    let variant_name = &variant.ident;
    let snake = &variant_name.to_string().to_snake_case();
    let screaming = &variant_name.to_string().to_shouty_snake_case();

    let tag_const = format_ident!("{}_TAG", screaming);
    let is_ident = format_ident!("is_{}", snake);
    let tag_val = idx as u64;

    let is_method = quote! {
        #[inline]
        pub const fn #is_ident(self) -> bool {
            self.tag() == Self::#tag_const
        }
    };

    match &variant.fields {
        // ── Unit variant ────────────────────────────────────────
        Fields::Unit => {
            let const_ident = format_ident!("{}", screaming);
            Variant {
                tag_const: tag_const.clone(),
                constructor: quote! {
                    pub const #const_ident: Self = Self(#tag_val as #backing_ident);
                },
                is: is_method,
                get_method: None,
                set_method: None,
                debug_arm: quote! {
                    Self::#tag_const => f.write_str(concat!(stringify!(#enum_ident), "::", stringify!(#variant_name)))
                },
            }
        }

        // ── Tuple variant with 1 field ──────────────────────────
        Fields::Unnamed(fields) => {
            assert!(
                fields.unnamed.len() == 1,
                "tagged: variants must have 0 or 1 fields, \
                 `{}` has {}",
                variant_name,
                fields.unnamed.len()
            );

            let field = parse_field_type(&fields.unnamed[0].ty, payload_width);
            let public_type = &field.public_type;
            let storage_type = &field.storage_type;

            let ctor_ident = format_ident!("{}", snake);
            let get_method_ident = format_ident!("get_{}", snake);
            let set_method_ident = format_ident!("set_{}", snake);

            // Mask that keeps exactly `declared_bits` bits.
            let payload_mask = if field.declared_bits >= 64 {
                u64::MAX
            } else {
                (1u64 << field.declared_bits) - 1
            };

            // For arbitrary_int types, unwrap via `.value()` and wrap via `::new()`.
            let val_to_raw = if field.is_arbitrary {
                quote! { val.value() as #backing_ident }
            } else {
                quote! { val as #backing_ident }
            };

            let raw_to_val = if field.is_arbitrary {
                quote! { #public_type::new(raw as #storage_type) }
            } else {
                quote! { raw as #public_type }
            };

            let set_to_raw = if field.is_arbitrary {
                quote! { value.value() as #backing_ident }
            } else {
                quote! { value as #backing_ident }
            };

            let constructor = quote! {
                #[inline]
                pub const fn #ctor_ident(val: #public_type) -> Self {
                    let masked = (#val_to_raw) & (#payload_mask as #backing_ident);
                    Self(masked << Self::TAG_WIDTH | (#tag_val as #backing_ident))
                }
            };

            let get_method = quote! {
                #[inline]
                pub const fn #get_method_ident(self) -> #public_type {
                    let raw = (self.0 >> Self::TAG_WIDTH) & (#payload_mask as #backing_ident);
                    #raw_to_val
                }
            };

            let set_method = quote! {
                #[inline]
                pub const fn #set_method_ident(&mut self, value: #public_type) {
                    let masked = (#set_to_raw) & (#payload_mask as #backing_ident);
                    self.0 = (self.0 & !((#payload_mask as #backing_ident) << Self::TAG_WIDTH)) | (masked << Self::TAG_WIDTH);
                }
            };

            let debug_arm = quote! {
                Self::#tag_const => f.debug_tuple(concat!(stringify!(#enum_ident), "::", stringify!(#variant_name)))
                    .field(&self.#get_method_ident())
                    .finish()
            };

            Variant {
                tag_const: tag_const.clone(),
                constructor,
                is: is_method,
                get_method: Some(get_method),
                set_method: Some(set_method),
                debug_arm,
            }
        }

        Fields::Named(_) => panic!("tagged: named fields not supported"),
    }
}

// ── Entry point ──────────────────────────────────────────────────────

#[proc_macro_attribute]
pub fn tagged(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);
    let backing = Backing::new(&parse_macro_input!(attr as syn::Ident));

    let backing_ident = &backing.ident;
    let backing_width = backing.width;

    assert!(
        !input.variants.is_empty(),
        "tagged: enum must have at least one variant"
    );

    let tags_width = tags_width_needed(input.variants.len());
    let payload_width = backing_width
        .checked_sub(tags_width)
        .unwrap_or_else(|| {
            panic!(
                "too many variants ({}) for {backing_ident}: \
                 need {tags_width} tag bits but only have {backing_width}",
                input.variants.len(),
            )
        });

    let tag_mask: u64 = if tags_width == 0 {
        0
    } else {
        (1u64 << tags_width) - 1
    };

    let vis = &input.vis;
    let ident = &input.ident;

    // Forward non-macro attributes (e.g. doc comments)
    let attrs: Vec<_> = input
        .attrs
        .iter()
        .filter(|a| !a.path().is_ident("tagged"))
        .collect();

    // ── #[default] ──
    let default_variants: Vec<_> = input
        .variants
        .iter()
        .filter(|v| v.attrs.iter().any(|a| a.path().is_ident("default")))
        .collect();

    assert!(
        default_variants.len() <= 1,
        "tagged: only one variant can be marked #[default]"
    );

    let default_impl = default_variants.first().map(|v| {
        assert!(
            matches!(v.fields, Fields::Unit),
            "tagged: #[default] can only be applied to unit variants"
        );
        let const_ident = format_ident!("{}", v.ident.to_string().to_shouty_snake_case());
        quote! {
            impl core::default::Default for #ident {
                #[inline]
                fn default() -> Self {
                    Self::#const_ident
                }
            }
        }
    });

    let infos: Vec<_> = input
        .variants
        .iter()
        .enumerate()
        .map(|(i, v)| process_variant(i, v, &backing, &input.ident, payload_width))
        .collect();

    let tag_consts: Vec<_> = infos
        .iter()
        .enumerate()
        .map(|(i, info)| {
            let tc = &info.tag_const;
            let val = i;
            quote! { const #tc: #backing_ident = #val as #backing_ident; }
        })
        .collect();

    let constructors: Vec<_> = infos.iter().map(|i| &i.constructor).collect();
    let is_methods: Vec<_> = infos.iter().map(|i| &i.is).collect();
    let get_methods: Vec<_> = infos.iter().filter_map(|i| i.get_method.as_ref()).collect();
    let set_methods: Vec<_> = infos.iter().filter_map(|i| i.set_method.as_ref()).collect();
    let debug_arms: Vec<_> = infos.iter().map(|i| &i.debug_arm).collect();

    let expanded = quote! {
        #(#attrs)*
        #[derive(Copy, Clone, PartialEq, Eq, Hash)]
        #[repr(transparent)]
        #vis struct #ident(#backing_ident);

        #[allow(non_upper_case_globals)]
        impl #ident {
            /// Number of bits used for the tag.
            pub const TAG_WIDTH: u32 = #tags_width;

            /// Bitmask for extracting the tag.
            pub const TAG_MASK: #backing_ident = #tag_mask as #backing_ident;

            /// Number of bits available for payload data.
            pub const PAYLOAD_WIDTH: u32 = #payload_width;

            // ── Tag constants ──
            #(#tag_consts)*

            // ── Constructors ──
            #(#constructors)*

            // ── Discriminant checks ──
            #(#is_methods)*

            #(#get_methods)*

            #(#set_methods)*

            // ── Raw access ──
            /// Extract the raw tag value.
            #[inline]
            const fn tag(self) -> #backing_ident {
                self.0 & Self::TAG_MASK
            }

            /// The raw backing integer.
            #[inline]
            pub const fn bits(self) -> #backing_ident {
                self.0
            }

            /// Construct from a raw backing integer.
            ///
            /// # Safety
            /// Caller must ensure the tag and payload are valid.
            #[inline]
            pub const unsafe fn from_bits(raw: #backing_ident) -> Self {
                Self(raw)
            }
        }

        impl core::fmt::Debug for #ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self.tag() {
                    #(#debug_arms,)*
                    _ => write!(f, "{}(unknown raw={})", stringify!(#ident), self.0),
                }
            }
        }

        #default_impl
    };

    expanded.into()
}
