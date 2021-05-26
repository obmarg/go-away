use std::{fmt, fmt::Write};

pub use super::{types::*, FieldType, Primitive};

pub enum GoType {
    Struct(Struct),
    NewType(NewType),
    Enum(Enum),
    Union(Union),
}

impl fmt::Display for GoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            GoType::Struct(details) => {
                writeln!(f, "type {} struct {{", details.name)?;
                for field in &details.fields {
                    writeln!(indented(f), "{}", GoField(field))?;
                }
                writeln!(f, "}}")?;
            }
            GoType::NewType(details) => {
                writeln!(f, "type {} {}", details.name, details.inner.go_type())?;
            }
            GoType::Enum(details) => {
                writeln!(f, "type {} string\n", details.name)?;
                writeln!(f, "const (")?;
                for variant in &details.variants {
                    writeln!(
                        indented(f),
                        "{} {} = \"{}\"",
                        variant.name,
                        details.name,
                        variant.serialized_name
                    )?;
                }
                writeln!(f, ")")?;
            }
            GoType::Union(details) => {
                writeln!(f, "type {} struct {{", details.name)?;
                for variant in &details.variants {
                    writeln!(indented(f), "*{}", variant.name)?;
                }
                writeln!(f, "}}\n")?;
                write!(f, "{}", UnionMarshal(&details))?;
                write!(f, "{}", UnionUnmarshal(&details))?;
            }
        }

        Ok(())
    }
}

pub struct GoField<'a>(&'a Field);

impl<'a> fmt::Display for GoField<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let details = self.0;
        write!(
            f,
            r#"{} {} `json:"{}"`"#,
            to_pascal_case(&details.name),
            details.ty.go_type(),
            details.serialized_name
        )
    }
}

impl FieldType {
    fn go_type(&self) -> String {
        match self {
            FieldType::Named(type_ref) => type_ref.name().to_string(),
            FieldType::Optional(inner) => format!("*{}", inner.go_type()),
            FieldType::List(inner) => format!("[]{}", inner.go_type()),
            FieldType::Map { key, value } => format!("map[{}]{}", key.go_type(), value.go_type()),
            FieldType::Primitive(Primitive::String) => "string".to_string(),
            FieldType::Primitive(Primitive::Float) => "float64".to_string(),
            FieldType::Primitive(Primitive::Int) => "int".to_string(),
            FieldType::Primitive(Primitive::Bool) => "bool".to_string(),
        }
    }
}

struct UnionMarshal<'a>(&'a Union);
struct UnionUnmarshal<'a>(&'a Union);

impl<'a> fmt::Display for UnionMarshal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let details = self.0;
        writeln!(
            f,
            "func (self *{}) MarshalJSON() ([]byte, error) {{",
            details.name
        )?;
        let last_i = details.variants.len() - 1;
        for (i, variant) in details.variants.iter().enumerate() {
            let f = &mut indented(f);
            writeln!(f, "if self.{} != nil {{", variant.name)?;
            // TODO: Use the correct marshallar
            write!(
                indented(f),
                "{}",
                AdjacentlyTaggedMarshaller {
                    tag: "k",
                    content: "c",
                    variant: &variant
                }
            )?;
            write!(f, "}}")?;
            if i != last_i {
                write!(f, " else ")?;
            }
        }
        writeln!(f, "\n}}")?;

        Ok(())
    }
}

struct AdjacentlyTaggedMarshaller<'a> {
    tag: &'a str,
    content: &'a str,
    variant: &'a UnionVariant,
}

impl<'a> fmt::Display for AdjacentlyTaggedMarshaller<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "var output map[string]interface{{}}")?;
        writeln!(
            f,
            r#"output["{}"] = "{}""#,
            self.tag, self.variant.serialized_name
        )?;
        writeln!(
            f,
            r#"output["{}"] = self.{}"#,
            self.content, self.variant.name
        )?;
        writeln!(f, "return json.Marshal(output)")
    }
}
// TODO: Adjacent needs {"t": "tag", "d": data}
// Internal needs {"type": "tag", ..rest_fields}
// External needs {"tag": {...}}

impl<'a> fmt::Display for UnionUnmarshal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

fn to_pascal_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev_is_underscore = true;
    for c in s.chars() {
        if c == '_' {
            prev_is_underscore = true;
        } else if prev_is_underscore {
            buf.push(c.to_ascii_uppercase());
            prev_is_underscore = false;
        } else {
            buf.push(c.to_ascii_lowercase());
        }
    }
    buf
}

pub fn indented<D: fmt::Write>(f: &mut D) -> indenter::Indented<'_, D> {
    indenter::indented(f).with_str("\t")
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn test_primitive_structs() {
        assert_snapshot!(
            GoType::Struct(Struct {
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
        type MyStruct struct {
        	AString string `json:"a_string"`
        	AnInt int `json:"renamed_tho"`
        	ABool bool `json:"also_renamed"`
        	AFloat float64 `json:"a_float"`
        }
        "###
        );
    }

    #[test]
    fn test_newtype_output() {
        assert_snapshot!(GoType::NewType(NewType {
            name: "UserId".into(),
            inner: FieldType::Primitive(Primitive::String),
        })
        .to_string(), @"type UserId string
");
    }

    #[test]
    fn test_enum_output() {
        assert_snapshot!(GoType::Enum(Enum {
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
        type FulfilmentType string

        const (
        	Delivery FulfilmentType = "DELIVERY"
        	Collection FulfilmentType = "COLLECTION"
        )
        "###);
    }

    #[test]
    fn test_adjacently_tagged_union_output() {
        assert_snapshot!(GoType::Union(Union {
            name: "MyUnion".into(),
            representation: UnionRepresentation::AdjacentlyTagged {
                tag: "type".into(),
                content: "data".into(),
            },
            variants: vec![
                UnionVariant {
                    name: "VarOne".into(),
                    serialized_name: "VAR_ONE".into(),
                },
                UnionVariant {
                    name: "VarTwo".into(),
                    serialized_name: "VAR_TWO".into(),
                }
            ]
        })
        .to_string());
    }

    #[test]
    fn test_list_types() {
        assert_snapshot!(
            FieldType::List(Box::new(FieldType::Primitive(Primitive::String))).go_type(),
            @"[]string"
        );
    }

    #[test]
    fn test_map_types() {
        assert_snapshot!(
            FieldType::Map{
                key: Box::new(FieldType::Primitive(Primitive::String)),
                value: Box::new(FieldType::Primitive(Primitive::Int))
            }.go_type(),
            @"map[string]int"
        );
    }

    #[test]
    fn test_option_types() {
        assert_snapshot!(
            FieldType::Optional(Box::new(FieldType::Primitive(Primitive::String))).go_type(),
            @"*string"
        );
    }
}
