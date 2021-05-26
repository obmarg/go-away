use proc_macro2::{Literal, TokenStream};
use quote::quote;

use serde_derive_internals::{
    ast::{Container, Data, Field, Style},
    attr::TagType,
    Ctxt,
};

pub fn type_metadata_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use quote::TokenStreamExt;

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
    let name_literal = Literal::string(&ident.to_string());
    let mut inner = TokenStream::new();
    match container.data {
        Data::Enum(variants) if variants.iter().all(|v| matches!(v.style, Style::Unit)) => {
            inner.append_all(quote! {
                let mut rv = types::Enum {
                    name: #name_literal.into(),
                    variants: vec![]
                };
            });
            for variant in variants {
                let variant_name = Literal::string(&variant.ident.to_string());
                let serialized_name = Literal::string(&variant.attrs.name().serialize_name());
                inner.append_all(quote! {
                    rv.variants.push(types::EnumVariant {
                        name: #variant_name.into(),
                        serialized_name: #serialized_name.into(),
                    });
                })
            }

            inner.append_all(quote! {
                FieldType::Named(registry.register_enum(rv))
            });
        }
        Data::Enum(variants) if variants.iter().all(|v| matches!(v.style, Style::Newtype)) => {
            let repr = match container.attrs.tag() {
                TagType::Adjacent { tag, content } => {
                    let tag = Literal::string(tag);
                    let content = Literal::string(content);
                    quote! {
                        types::UnionRepresentation::AdjacentlyTagged {
                            tag: #tag.into(),
                            content: #content.into()
                        }
                    }
                }
                _ => todo!("do the other tag types"),
            };
            inner.append_all(quote! {
                let mut rv = types::Union {
                    name: #name_literal.into(),
                    representation: #repr,
                    variants: vec![]
                };
            });
            for variant in variants {
                let variant_name = Literal::string(&variant.ident.to_string());
                let serialized_name = Literal::string(&variant.attrs.name().serialize_name());
                let variant_type = variant.fields.first().unwrap().ty;
                inner.append_all(quote! {
                    rv.variants.push(
                        types::UnionVariant {
                            name: Some(#variant_name.to_string()),
                            ty: #variant_type::metadata(registry),
                            serialized_name: #serialized_name.to_string()
                        }
                    );
                })
            }
            inner.append_all(quote! {
                FieldType::Named(registry.register_union(rv))
            })
        }
        Data::Enum(variants) => {
            let repr = match container.attrs.tag() {
                TagType::Adjacent { tag, content } => {
                    let tag = Literal::string(tag);
                    let content = Literal::string(content);
                    quote! {
                        types::UnionRepresentation::AdjacentlyTagged {
                            tag: #tag.into(),
                            content: #content.into()
                        }
                    }
                }
                _ => todo!("do the other tag types"),
            };
            inner.append_all(quote! {
                let mut rv = types::Union {
                    name: #name_literal.into(),
                    representation: #repr,
                    variants: vec![]
                };
            });
            for variant in variants {
                let variant_name = Literal::string(&variant.ident.to_string());
                let serialized_name = Literal::string(&variant.attrs.name().serialize_name());
                let inner_type_block = struct_block(&variant.ident.to_string(), &variant.fields);
                inner.append_all(quote! {
                    rv.variants.push(
                        types::UnionVariant {
                            name: Some(#variant_name.to_string()),
                            ty: FieldType::Named({#inner_type_block}),
                            serialized_name: #serialized_name.to_string()
                        }
                    );
                })
            }
            inner.append_all(quote! {
                FieldType::Named(registry.register_union(rv))
            })
        }
        Data::Struct(_, fields) => {
            let struct_block_contents = struct_block(&ident.to_string(), &fields);
            inner.append_all(quote! {
                let type_ref = {
                    #struct_block_contents
                };
                FieldType::Named(type_ref)
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

fn struct_block(name: &str, fields: &[Field]) -> TokenStream {
    use quote::TokenStreamExt;

    let mut rv = TokenStream::new();

    let name_literal = Literal::string(name);
    rv.append_all(quote! {
        let mut st = types::Struct {
            name: #name_literal.into(),
            fields: vec![]
        };
    });
    for field in fields {
        let field_name = name_of_member(&field.member);
        let serialized_name = Literal::string(&field.attrs.name().serialize_name());
        let ty = field.ty;
        rv.append_all(quote! {
            st.fields.push(
                types::Field {
                    name: #field_name.into(),
                    serialized_name: #serialized_name.into(),
                    ty: #ty::metadata(registry)
                }
            );
        });
    }
    rv.append_all(quote! {
        registry.register_struct(st)
    });

    rv
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
