use proc_macro2::{Literal, TokenStream};
use quote::quote;

use serde_derive_internals::{
    ast::{Container, Data, Style},
    Ctxt,
};

pub fn type_metadata_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let ctx = Ctxt::new();

    let container =
        Container::from_ast(&ctx, &ast, serde_derive_internals::Derive::Deserialize).unwrap();

    match ctx.check() {
        Ok(_) => {}
        Err(errors) => {
            let mut rv = TokenStream::new();
            for err in errors {
                rv.extend(err.to_compile_error());
            }

            return Ok(rv);
        }
    }

    let ident = &container.ident;
    let mut inner = TokenStream::new();
    match container.data {
        Data::Enum(variants) if variants.iter().all(|v| matches!(v.style, Style::Unit)) => {
            // TODO: Map this to a go "enum"
            todo!()
        }
        Data::Enum(variants) if variants.iter().all(|v| matches!(v.style, Style::Newtype)) => {
            // TODO: Map this to a union of the inner types
            todo!()
        }
        Data::Enum(variants) => {
            // TODO: Map this to a union of the inner types
            todo!()
        }
        Data::Struct(_, fields) => {
            use quote::TokenStreamExt;
            let name_literal = Literal::string(&ident.to_string());
            inner.append_all(quote! {
                let mut rv = types::Struct {
                    name: #name_literal.into(),
                    fields: vec![]
                };
            });
            for field in fields {
                let field_name = name_of_member(&field.member);
                let serialized_name = Literal::string(&field.attrs.name().serialize_name());
                let ty = field.ty;
                inner.append_all(quote! {
                    rv.fields.push(
                        types::Field {
                            name: #field_name.into(),
                            serialized_name: #serialized_name.into(),
                            ty: #ty::metadata(registry)
                        }
                    );
                });
            }
            inner.append_all(quote! {
                FieldType::Named(registry.register_struct(rv))
            })
        }
    }

    // TODO: Need to support generics here.
    Ok(quote! {
        impl ::go_away::TypeMetadata for #ident {
            fn metadata(registry: &mut ::go_away::TypeRegistry) -> ::go_away::FieldType {
                use ::go_away::{types, FieldType};
                #inner
            }
        }
    })
}

fn name_of_member(member: &syn::Member) -> proc_macro2::Literal {
    use syn::Index;
    match member {
        syn::Member::Named(ident) => Literal::string(&ident.to_string()),
        syn::Member::Unnamed(Index { index, .. }) => Literal::string(&format!("_{}", index)),
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use quote::quote;

    use super::*;

    #[test]
    fn test_struct() {
        assert_snapshot!(test_conversion(quote! {
            struct MyData {
                field_one: String,
                field_two: String
            }
        }))
    }

    fn test_conversion(ts: proc_macro2::TokenStream) -> String {
        format_code(
            &type_metadata_derive(&syn::parse2(ts).unwrap())
                .unwrap()
                .to_string(),
        )
    }

    fn format_code(text: &str) -> String {
        xshell::cmd!("rustfmt").stdin(text).read().unwrap()
    }
}
