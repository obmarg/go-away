use proc_macro2::{Literal, TokenStream};
use quote::quote;
use serde_derive_internals::{
    ast::{Container, Data, Field, Style},
    attr::TagType,
    Ctxt,
};

mod type_id;

use type_id::TypeIdCall;

pub fn type_metadata_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use quote::TokenStreamExt;

    let ctx = Ctxt::new();

    let container =
        Container::from_ast(&ctx, ast, serde_derive_internals::Derive::Deserialize).unwrap();

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
    let type_id = TypeIdCall::for_struct(&container.ident, container.generics);

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
                FieldType::Named(registry.register_enum(#type_id, rv))
            });
        }
        Data::Enum(variants) if variants.iter().all(|v| matches!(v.style, Style::Newtype)) => {
            let repr = tag_to_representation(container.attrs.tag());
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
                let metadata_call = metadata_call(variant.fields.first().unwrap().ty);
                inner.append_all(quote! {
                    rv.variants.push(
                        types::UnionVariant {
                            name: Some(#variant_name.to_string()),
                            ty: #metadata_call,
                            serialized_name: #serialized_name.to_string()
                        }
                    );
                })
            }
            inner.append_all(quote! {
                FieldType::Named(registry.register_union(#type_id, rv))
            })
        }
        Data::Enum(variants) => {
            let repr = tag_to_representation(container.attrs.tag());
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
                let type_id =
                    TypeIdCall::for_variant(&container.ident, &variant.ident, container.generics);
                let inner_type_block =
                    struct_block(&variant.ident.to_string(), &variant.fields, type_id);
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
                FieldType::Named(registry.register_union(#type_id, rv))
            })
        }
        Data::Struct(Style::Newtype, fields) => {
            let metadata_call = metadata_call(fields.first().unwrap().ty);
            inner.append_all(quote! {
                let nt = types::NewType {
                    name: #name_literal.to_string(),
                    inner: #metadata_call,
                };
                FieldType::Named(registry.register_newtype(#type_id, nt))
            });
        }
        Data::Struct(_, fields) => {
            let struct_block_contents = struct_block(&ident.to_string(), &fields, type_id);
            inner.append_all(quote! {
                let type_ref = {
                    #struct_block_contents
                };
                FieldType::Named(type_ref)
            });
        }
    }

    let (impl_generics, ty_generics, where_clause) = container.generics.split_for_impl();
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::go_away::TypeMetadata for #ident #ty_generics #where_clause {
            fn metadata(registry: &mut ::go_away::TypeRegistry) -> ::go_away::types::FieldType {
                use ::go_away::types::{self, FieldType};
                #inner
            }
        }
    })
}

fn struct_block(name: &str, fields: &[Field], type_id: TypeIdCall<'_>) -> TokenStream {
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
        let ty_def = metadata_call(field.ty);
        rv.append_all(quote! {
            st.fields.push(
                types::Field {
                    name: #field_name.into(),
                    serialized_name: #serialized_name.into(),
                    ty: #ty_def
                }
            );
        });
    }
    rv.append_all(quote! {
        registry.register_struct(#type_id, st)
    });

    rv
}

fn tag_to_representation(tag: &TagType) -> proc_macro2::TokenStream {
    match tag {
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
        TagType::External => {
            quote! { types::UnionRepresentation::ExternallyTagged }
        }
        TagType::Internal { tag } => {
            let tag = Literal::string(tag);
            quote! {
                types::UnionRepresentation::InternallyTagged {
                    tag: #tag.into()
                }
            }
        }
        TagType::None => {
            quote! { types::UnionRepresentaiton::Untagged }
        }
    }
}

fn metadata_call(ty: &syn::Type) -> proc_macro2::TokenStream {
    match ty {
        syn::Type::Reference(r) => metadata_call(r.elem.as_ref()),
        syn::Type::Paren(r) => metadata_call(r.elem.as_ref()),
        other => quote! { <#other as ::go_away::TypeMetadata>::metadata(registry) },
    }
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

    #[test]
    fn test_newtype_struct() {
        assert_snapshot!(test_conversion(quote! {
            struct MyData(String);
        }))
    }

    #[test]
    fn test_struct_with_single_field() {
        assert_snapshot!(test_conversion(quote! {
            struct MyData {
                data: String
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
