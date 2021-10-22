use std::{fmt, fmt::Write};

use indenter::indented;
use indoc::writedoc;

mod tabify;
mod validate;

use crate::output::validate::UnionValidate;

pub use super::types::*;

pub enum GoType<'a> {
    Struct(&'a Struct),
    NewType(&'a NewType),
    Alias(&'a Alias),
    Enum(&'a Enum),
    Union(&'a Union),
}

impl<'a> fmt::Display for GoType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let f = &mut tabify::tabify(f);
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
            GoType::Alias(details) => {
                writeln!(f, "type {} {}", details.name, details.inner.go_type())?;
            }
            GoType::Enum(details) => {
                writeln!(f, "type {} string\n", details.name)?;
                writeln!(f, "const (")?;
                for variant in &details.variants {
                    writeln!(
                        indented(f),
                        "{}{} {} = \"{}\"",
                        details.name,
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
                    writeln!(
                        indented(f),
                        "{} *{}",
                        variant.go_name(),
                        variant.ty.go_type()
                    )?;
                }
                writeln!(f, "}}\n")?;
                write!(f, "{}", UnionMarshal(&details))?;
                write!(f, "{}", UnionUnmarshal(&details))?;
                write!(f, "{}", UnionValidate(&details))?;
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
            FieldType::Primitive(Primitive::Time) => "time.Time".to_string(),
        }
    }
}

struct UnionMarshal<'a>(&'a Union);
struct UnionUnmarshal<'a>(&'a Union);

impl<'a> fmt::Display for UnionMarshal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let details = self.0;
        writedoc!(
            f,
            r#"
                func (self {}) MarshalJSON() ([]byte, error) {{
                    if err := self.Validate(); err != nil {{
                        return nil, fmt.Errorf("Validate Failed: %w", err)
                    }}
            "#,
            details.name
        )?;
        for variant in details.variants.iter() {
            let f = &mut indented(f);
            writeln!(f, "if self.{} != nil {{", variant.go_name())?;
            match &details.representation {
                UnionRepresentation::AdjacentlyTagged { tag, content } => {
                    write!(
                        indented(f),
                        "{}",
                        AdjacentlyTaggedMarshaller {
                            tag,
                            content,
                            variant: &variant
                        }
                    )?;
                }
                UnionRepresentation::InternallyTagged { tag } => {
                    write!(
                        indented(f),
                        "{}",
                        InternallyTaggedMarshaller {
                            tag,
                            variant: &variant
                        }
                    )?;
                }
                _ => todo!("Implement the other tagging enum representations"),
            }
            write!(f, "}}")?;
            write!(f, " else ")?;
        }
        writeln!(indented(f), "{{")?;
        writeln!(
            indented(f),
            "\treturn nil, fmt.Errorf(\"No variant was present\")"
        )?;
        writeln!(indented(f), "}}")?;
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl UnionVariant {
    fn go_name(&self) -> String {
        match (&self.name, &self.ty) {
            (Some(name), _) => name.clone(),
            (_, FieldType::Named(type_ref)) => type_ref.name().to_string(),
            _ => todo!("Variant must be named or named type for now (fix this later)"),
        }
    }
}

struct AdjacentlyTaggedMarshaller<'a> {
    tag: &'a str,
    content: &'a str,
    variant: &'a UnionVariant,
}

impl<'a> fmt::Display for AdjacentlyTaggedMarshaller<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writedoc! {
            f,
            r#"
                output := make(map[string]interface{{}})
                output["{tag}"] = "{serialized_name}"
                output["{content}"] = self.{variant_go_name}
                return json.Marshal(output)
            "#,
            tag = self.tag,
            serialized_name = self.variant.serialized_name,
            content = self.content,
            variant_go_name = self.variant.go_name()
        }
    }
}

struct InternallyTaggedMarshaller<'a> {
    tag: &'a str,
    variant: &'a UnionVariant,
}

impl<'a> fmt::Display for InternallyTaggedMarshaller<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writedoc! {
            f,
            r#"
                return json.Marshal(struct{{
                    Tag string `json:"{tag}"`
                    {variant_type}
                }}{{
                    Tag: "{serialized_name}",
                    {variant_type}: *self.{variant_go_name},
                }})
            "#,
            tag = self.tag,
            serialized_name = self.variant.serialized_name,
            variant_go_name = self.variant.go_name(),
            variant_type = self.variant.ty.go_type()
        }
    }
}

// TODO: Adjacent needs {"t": "tag", "d": data}
// Internal needs {"type": "tag", ..rest_fields}
// External needs {"tag": {...}}

impl<'a> fmt::Display for UnionUnmarshal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let details = self.0;
        writeln!(
            f,
            "func (self *{}) UnmarshalJSON(data []byte) error {{",
            details.name
        )?;
        match &details.representation {
            UnionRepresentation::AdjacentlyTagged { tag, content } => {
                let f = &mut indented(f);
                writeln!(
                    f,
                    "temp := struct {{\n\tTag string `json:\"{}\"`\n}}{{}}",
                    tag
                )?;
                writeln!(f, "if err := json.Unmarshal(data, &temp); err != nil {{")?;
                writeln!(f, "\treturn err")?;
                writeln!(f, "}}")?;
                for variant in &details.variants {
                    write!(
                        f,
                        "{}",
                        AdjacentlyTaggedVariantUnmarshaller {
                            content,
                            variant,
                            all_variants: &details.variants
                        }
                    )?;
                }
                writeln!(f, "{{")?;
                writeln!(indented(f), "return errors.New(\"Unknown type tag\")")?;
                writeln!(f, "}}")?;
                writeln!(f, "return nil")?;
            }
            UnionRepresentation::InternallyTagged { tag } => {
                let f = &mut indented(f);
                writeln!(
                    f,
                    "temp := struct{{\n\tTag string `json:\"{}\"`\n}}{{}}",
                    tag
                )?;
                writeln!(f, "if err := json.Unmarshal(data, &temp); err != nil {{")?;
                writeln!(f, "\treturn err")?;
                writeln!(f, "}}")?;
                for variant in &details.variants {
                    write!(
                        f,
                        "{}",
                        InternallyTaggedVariantUnmarshaller {
                            variant,
                            all_variants: &details.variants
                        }
                    )?;
                }
                writeln!(f, "{{")?;
                writeln!(indented(f), "return errors.New(\"Unknown type tag\")")?;
                writeln!(f, "}}")?;
                writeln!(f, "return nil")?;
            }

            _ => todo!("Support other enum representaitons"),
        }
        writeln!(f, "}}")?;

        // TODO: Support anything other than Adjacent tagging
        //todo!("Write UnionUnmarshal")
        Ok(())
    }
}

struct AdjacentlyTaggedVariantUnmarshaller<'a> {
    content: &'a str,
    variant: &'a UnionVariant,
    all_variants: &'a [UnionVariant],
}

impl<'a> fmt::Display for AdjacentlyTaggedVariantUnmarshaller<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "if temp.Tag == \"{}\" {{", self.variant.serialized_name)?;
        writeln!(
            indented(f),
            "rv := struct {{\n\tData {} `json:\"{}\"`\n}}{{}}",
            self.variant.ty.go_type(),
            self.content,
        )?;
        writeln!(
            indented(f),
            "if err := json.Unmarshal(data, &rv); err != nil {{"
        )?;
        writeln!(indented(f), "\treturn err")?;
        writeln!(indented(f), "}}")?;
        writeln!(indented(f), "self.{} = &rv.Data", self.variant.go_name())?;
        for other_variant in self.all_variants {
            if other_variant == self.variant {
                continue;
            }
            writeln!(indented(f), "self.{} = nil", other_variant.go_name())?;
        }
        write!(f, "}} else ")
    }
}

struct InternallyTaggedVariantUnmarshaller<'a> {
    variant: &'a UnionVariant,
    all_variants: &'a [UnionVariant],
}

impl<'a> fmt::Display for InternallyTaggedVariantUnmarshaller<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writedoc!(
            f,
            r#"
            if temp.Tag == "{serialized_name}" {{
                var rv {go_type}
                if err := json.Unmarshal(data, &rv); err != nil {{
                    return err
                }}
                self.{go_name} = &rv
            "#,
            serialized_name = self.variant.serialized_name,
            go_type = self.variant.ty.go_type(),
            go_name = self.variant.go_name()
        )?;
        for other_variant in self.all_variants {
            if other_variant == self.variant {
                continue;
            }
            writeln!(indented(f), "self.{} = nil", other_variant.go_name())?;
        }
        write!(f, "}} else ")
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

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;
    use crate::types::TypeRef;

    #[test]
    fn test_primitive_structs() {
        assert_snapshot!(
            GoType::Struct(&Struct {
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
        assert_snapshot!(GoType::NewType(&NewType {
            name: "UserId".into(),
            inner: FieldType::Primitive(Primitive::String),
        })
        .to_string(), @"type UserId string
");
    }

    #[test]
    fn test_enum_output() {
        assert_snapshot!(GoType::Enum(&Enum {
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
        	FulfilmentTypeDelivery FulfilmentType = "DELIVERY"
        	FulfilmentTypeCollection FulfilmentType = "COLLECTION"
        )
        "###);
    }

    #[test]
    fn test_adjacently_tagged_union_output() {
        assert_snapshot!(GoType::Union(&Union {
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
