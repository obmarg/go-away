use proc_macro2::{Literal, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};

pub enum TypeIdCall<'a> {
    Struct(StructTypeIdCall<'a>),
    Variant(VariantTypeIdCall<'a>),
}

impl<'a> TypeIdCall<'a> {
    pub fn for_struct(ident: &'a syn::Ident, generics: &'a syn::Generics) -> Self {
        TypeIdCall::Struct(StructTypeIdCall {
            ident,
            generics: StaticGenerics(generics),
        })
    }

    pub fn for_variant(
        enum_ident: &'a syn::Ident,
        variant_name: &'a syn::Ident,
        generics: &'a syn::Generics,
    ) -> Self {
        TypeIdCall::Variant(VariantTypeIdCall {
            enum_ident,
            variant_name,
            generics: StaticGenerics(generics),
        })
    }
}

impl ToTokens for TypeIdCall<'_> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        match self {
            Self::Struct(s) => s.to_tokens(stream),
            Self::Variant(v) => v.to_tokens(stream),
        }
    }
}

pub struct StructTypeIdCall<'a> {
    ident: &'a syn::Ident,
    generics: StaticGenerics<'a>,
}

impl ToTokens for StructTypeIdCall<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.ident;
        let generics = &self.generics;

        tokens.append_all(quote! {
            ::go_away::TypeId::for_type::<#ident #generics>()
        })
    }
}

pub struct VariantTypeIdCall<'a> {
    enum_ident: &'a syn::Ident,
    variant_name: &'a syn::Ident,
    generics: StaticGenerics<'a>,
}

impl ToTokens for VariantTypeIdCall<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let enum_ident = self.enum_ident;
        let variant_name = Literal::string(&self.variant_name.to_string());
        let generics = &self.generics;

        tokens.append_all(quote! {
            ::go_away::TypeId::for_variant::<#enum_ident #generics, _>(#variant_name)
        })
    }
}

/// Some generics, but with all the lifetimes replaced with 'static.
///
/// We need this to call `TypeId::of` because it doesn't like non static
/// lifetimes
struct StaticGenerics<'a>(&'a syn::Generics);

impl quote::ToTokens for StaticGenerics<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use syn::{GenericParam, Token};

        if self.0.params.is_empty() {
            return;
        }

        TokensOrDefault(&self.0.lt_token).to_tokens(tokens);

        // Print lifetimes before types and consts, regardless of their
        // order in self.params.
        //
        // TODO: ordering rules for const parameters vs type parameters have
        // not been settled yet. https://github.com/rust-lang/rust/issues/44580
        let mut trailing_or_empty = true;
        for param in self.0.params.pairs() {
            if let GenericParam::Lifetime(_) = *param.value() {
                // Hard code this lifetime to 'static
                tokens.append_all(quote! { 'static });
                param.punct().to_tokens(tokens);
                trailing_or_empty = param.punct().is_some();
            }
        }
        for param in self.0.params.pairs() {
            if let GenericParam::Lifetime(_) = **param.value() {
                continue;
            }
            if !trailing_or_empty {
                <Token![,]>::default().to_tokens(tokens);
                trailing_or_empty = true;
            }
            match *param.value() {
                GenericParam::Lifetime(_) => unreachable!(),
                GenericParam::Type(param) => {
                    // Leave off the type parameter defaults
                    param.ident.to_tokens(tokens);
                }
                GenericParam::Const(param) => {
                    // Leave off the const parameter defaults
                    param.ident.to_tokens(tokens);
                }
            }
            param.punct().to_tokens(tokens);
        }

        TokensOrDefault(&self.0.gt_token).to_tokens(tokens);
    }
}

pub struct TokensOrDefault<'a, T: 'a>(pub &'a Option<T>);

impl<T> quote::ToTokens for TokensOrDefault<'_, T>
where
    T: quote::ToTokens + Default,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.0 {
            Some(t) => t.to_tokens(tokens),
            None => T::default().to_tokens(tokens),
        }
    }
}
