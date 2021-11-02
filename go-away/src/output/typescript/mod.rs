use std::{fmt, fmt::Write};

use indenter::indented;

pub use crate::types::*;

pub enum TypeScriptType<'a> {
    Struct(&'a Struct),
    NewType(&'a NewType),
    Alias(&'a Alias),
    Enum(&'a Enum),
    Union(&'a Union),
}

impl<'a> fmt::Display for TypeScriptType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            TypeScriptType::Struct(details) => {
                writeln!(f, "type {} = {{", details.name)?;
                for field in &details.fields {
                    writeln!(indented(f), "{}", TypeScriptField(field))?;
                }
                writeln!(f, "}}")?;
            }
            TypeScriptType::NewType(details) => {
                writeln!(
                    f,
                    "type {} = {};",
                    details.name,
                    details.inner.typescript_type()
                )?;
            }
            TypeScriptType::Alias(details) => {
                writeln!(
                    f,
                    "type {} = {};",
                    details.name,
                    details.inner.typescript_type()
                )?;
            }
            TypeScriptType::Enum(details) => {
                writeln!(f, "enum {} {{", details.name)?;
                for variant in &details.variants {
                    writeln!(
                        indented(f),
                        "{} = \"{}\",",
                        variant.name,
                        variant.serialized_name
                    )?;
                }
                writeln!(f, "}}")?;
            }
            TypeScriptType::Union(details) => match &details.representation {
                UnionRepresentation::ExternallyTagged => {
                    let mut union_types: Vec<String> = vec![];
                    for variant in &details.variants {
                        union_types.push(format!(
                            "{{ \"{}\": {} }}",
                            variant.serialized_name,
                            variant.typescript_name()
                        ));
                    }
                    writeln!(f, "type {} = {};", details.name, union_types.join(" | "))?;
                }
                UnionRepresentation::InternallyTagged { tag } => {
                    let mut union_types: Vec<String> = vec![];
                    for variant in &details.variants {
                        union_types.push(format!(
                            "({{ \"{}\": \"{}\" }} & {})",
                            tag,
                            variant.serialized_name,
                            variant.typescript_name()
                        ));
                    }
                    writeln!(f, "type {} = {};", details.name, union_types.join(" | "))?;
                }
                UnionRepresentation::Untagged => {
                    let mut union_types: Vec<String> = vec![];
                    for variant in &details.variants {
                        union_types.push(variant.typescript_name());
                    }
                    writeln!(f, "type {} = {};", details.name, union_types.join(" | "))?;
                }
                UnionRepresentation::AdjacentlyTagged { tag, content } => {
                    let mut union_types: Vec<String> = vec![];
                    for variant in &details.variants {
                        union_types.push(format!(
                            "{{ \"{}\": \"{}\", \"{}\": {} }}",
                            tag,
                            variant.serialized_name,
                            content,
                            variant.typescript_name()
                        ));
                    }
                    writeln!(f, "type {} = {};", details.name, union_types.join(" | "))?;
                }
            },
        }

        Ok(())
    }
}

pub struct TypeScriptField<'a>(&'a Field);

impl<'a> fmt::Display for TypeScriptField<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let details = self.0;
        write!(
            f,
            r#"{}: {};"#,
            details.serialized_name,
            details.ty.typescript_type(),
        )
    }
}

impl FieldType {
    fn typescript_type(&self) -> String {
        match self {
            FieldType::Named(type_ref) => type_ref.name().to_string(),
            FieldType::Optional(inner) => format!("{} | null", inner.typescript_type()),
            FieldType::List(inner) => format!("{}[]", inner.typescript_type()),
            FieldType::Map { key, value } => {
                format!(
                    "Record<{}, {}>",
                    key.typescript_type(),
                    value.typescript_type()
                )
            }
            FieldType::Primitive(Primitive::String) => "string".to_string(),
            FieldType::Primitive(Primitive::Float) => "number".to_string(),
            FieldType::Primitive(Primitive::Int) => "number".to_string(),
            FieldType::Primitive(Primitive::Bool) => "boolean".to_string(),
            FieldType::Primitive(Primitive::Time) => "string".to_string(),
        }
    }
}

impl UnionVariant {
    fn typescript_name(&self) -> String {
        match (&self.name, &self.ty) {
            (_, FieldType::Named(_)) => self.ty.typescript_type(),
            (_, FieldType::Optional(_)) => self.ty.typescript_type(),
            (Some(name), _) => name.clone(),
            _ => todo!("Variant must be named or named type for now (fix this later)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;
    use crate::types::TypeRef;

    #[test]
    fn test_primitive_structs() {
        assert_snapshot!(
            TypeScriptType::Struct(&Struct {
                name: "MyStruct".into(),
                fields: vec![
                    Field {
                        name: "a_string".into(),
                        serialized_name: "a_string".into(),
                        ty: FieldType::Primitive(Primitive::String),
                    },
                    Field {
                        name: "an_int".into(),
                        serialized_name: "renamed_tho".into(),
                        ty: FieldType::Primitive(Primitive::Int),
                    },
                    Field {
                        name: "a_bool".into(),
                        serialized_name: "also_renamed".into(),
                        ty: FieldType::Primitive(Primitive::Bool),
                    },
                    Field {
                        name: "a_float".into(),
                        serialized_name: "a_float".into(),
                        ty: FieldType::Primitive(Primitive::Float),
                    },
                ],
            })
            .to_string(),
            @r###"
        type MyStruct = {
            a_string: string;
            renamed_tho: number;
            also_renamed: boolean;
            a_float: number;
        }
        "###
        );
    }

    #[test]
    fn test_newtype_output() {
        assert_snapshot!(TypeScriptType::NewType(&NewType {
            name: "UserId".into(),
            inner: FieldType::Primitive(Primitive::String),
        })
        .to_string(), @"type UserId = string;
");
    }

    #[test]
    fn test_newtype_struct_output() {
        // pub type Users = HashMap<UserId, UserData>;
        assert_snapshot!(TypeScriptType::NewType(&NewType {
            name: "Users".into(),
            inner: FieldType::Map{ key: Box::new(FieldType::Named(TypeRef {
                name: "UserId".into()
            })), value: Box::new(FieldType::Named(TypeRef {
                name: "UserData".into()
            }))},
        })
        .to_string(), @"type Users = Record<UserId, UserData>;
    ");
    }

    #[test]
    fn test_alias_output() {
        // pub type Users = HashMap<UserId, UserData>;
        assert_snapshot!(TypeScriptType::Alias(&Alias {
        name: "Users".into(),
        inner: FieldType::Map{ key: Box::new(FieldType::Named(TypeRef {
            name: "UserId".into()
        })), value: Box::new(FieldType::Named(TypeRef {
            name: "UserData".into()
        }))},
    })
    .to_string(), @"type Users = Record<UserId, UserData>;");
    }

    #[test]
    fn test_enum_output() {
        assert_snapshot!(TypeScriptType::Enum(&Enum {
            name: "FulfilmentType".into(),
            variants: vec![
                EnumVariant {
                    name: "Delivery".into(),
                    serialized_name: "DELIVERY".into(),
                },
                EnumVariant {
                    name: "Collection".into(),
                    serialized_name: "COLLECTION".into(),
                },
            ],
        })
        .to_string(), @r###"
        enum FulfilmentType {
            Delivery = "DELIVERY",
            Collection = "COLLECTION",
        }
        "###);
    }

    #[test]
    fn test_externally_tagged_union_output() {
        assert_snapshot!(TypeScriptType::Union(&Union {
            name: "MyUnion".into(),
            representation: UnionRepresentation::ExternallyTagged,
            variants: vec![
                UnionVariant {
                    name: Some("VarOne".into()),
                    ty: FieldType::Named(TypeRef {
                        name: "VarOne".into()
                    }),
                    serialized_name: "VAR_ONE".into(),
                },
                UnionVariant {
                    name: Some("VarTwo".into()),
                    ty: FieldType::Named(TypeRef {
                        name: "VarTwo".into()
                    }),
                    serialized_name: "VAR_TWO".into(),
                }
            ]
        })
        .to_string(), @r###"
        type MyUnion = { "VAR_ONE": VarOne } | { "VAR_TWO": VarTwo };
		"###);
    }

    #[test]
    fn test_interally_tagged_union_output() {
        assert_snapshot!(TypeScriptType::Union(&Union {
            name: "MyUnion".into(),
            representation: UnionRepresentation::InternallyTagged {
                tag: "type".to_string()
            },
            variants: vec![
                UnionVariant {
                    name: Some("VarOne".into()),
                    ty: FieldType::Named(TypeRef {
                        name: "VarOne".into()
                    }),
                    serialized_name: "VAR_ONE".into(),
                },
                UnionVariant {
                    name: Some("VarTwo".into()),
                    ty: FieldType::Named(TypeRef {
                        name: "VarTwo".into()
                    }),
                    serialized_name: "VAR_TWO".into(),
                }
            ]
        })
        .to_string(), @r###"
        type MyUnion = ({ "type": "VAR_ONE" } & VarOne) | ({ "type": "VAR_TWO" } & VarTwo);
		"###);
    }

    #[test]
    fn test_adjacently_tagged_union_output() {
        assert_snapshot!(TypeScriptType::Union(&Union {
            name: "MyUnion".into(),
            representation: UnionRepresentation::AdjacentlyTagged {
                tag: "type".into(),
                content: "data".into(),
            },
            variants: vec![
                UnionVariant {
                    name: Some("VarOne".into()),
                    ty: FieldType::Named(TypeRef {
                        name: "VarOne".into()
                    }),
                    serialized_name: "VAR_ONE".into(),
                },
                UnionVariant {
                    name: Some("VarTwo".into()),
                    ty: FieldType::Named(TypeRef {
                        name: "VarTwo".into()
                    }),
                    serialized_name: "VAR_TWO".into(),
                }
            ]
        })
        .to_string(), @r###"
        type MyUnion = { "type": "VAR_ONE", "data": VarOne } | { "type": "VAR_TWO", "data": VarTwo };
		"###);
    }

    #[test]
    fn test_untagged_union_output() {
        assert_snapshot!(TypeScriptType::Union(&Union {
            name: "MyUnion".into(),
            representation: UnionRepresentation::Untagged,
            variants: vec![
                UnionVariant {
                    name: Some("VarA".into()),
                    ty: FieldType::Named(TypeRef {
                        name: "VarOne".into()
                    }),
                    serialized_name: "VAR_A".into(),
                },
                UnionVariant {
                    name: Some("VarB".into()),
                    ty: FieldType::Named(TypeRef {
                        name: "VarTwo".into()
                    }),
                    serialized_name: "VAR_A".into(),
                }
            ]
        })
        .to_string(), @r###"
        type MyUnion = VarOne | VarTwo;
    	"###);
    }

    #[test]
    fn test_untagged_option_union_output() {
        assert_snapshot!(TypeScriptType::Union(&Union {
            name: "MyUnion".into(),
            representation: UnionRepresentation::Untagged,
            variants: vec![
                UnionVariant {
                    name: Some("VarA".into()),
                    ty: FieldType::Optional(Box::new(FieldType::Named(TypeRef {
                        name: "VarOne".into()
                    }))),
                    serialized_name: "VAR_A".into(),
                },
                UnionVariant {
                    name: Some("VarB".into()),
                    ty: FieldType::Named(TypeRef {
                        name: "VarTwo".into()
                    }),
                    serialized_name: "VAR_A".into(),
                }
            ]
        })
        .to_string(), @r###"
        type MyUnion = VarOne | null | VarTwo;
    	"###);
    }

    #[test]
    fn test_list_types() {
        assert_snapshot!(
            FieldType::List(Box::new(FieldType::Primitive(Primitive::String))).typescript_type(),
            @"string[]"
        );
    }

    #[test]
    fn test_map_types() {
        assert_snapshot!(
            FieldType::Map{
                key: Box::new(FieldType::Primitive(Primitive::String)),
                value: Box::new(FieldType::Primitive(Primitive::Int))
            }.typescript_type(),
            @"Record<string, number>"
        );
    }

    #[test]
    fn test_option_types() {
        assert_snapshot!(
            FieldType::Optional(Box::new(FieldType::Primitive(Primitive::String))).typescript_type(),
            @"string | null"
        );
    }
}
