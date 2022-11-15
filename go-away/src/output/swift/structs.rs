use std::fmt::{self, Write};

use indenter::indented;
use indoc::writedoc;

use super::to_camel_case;
use crate::types::{self, FieldType};

pub struct SwiftStruct<'a> {
    name: &'a str,
    fields: Vec<SwiftField<'a>>,
    newtype: bool,
}

impl<'a> SwiftStruct<'a> {
    pub fn new(name: &'a str) -> SwiftStruct {
        SwiftStruct {
            name,
            fields: Vec::new(),
            newtype: false,
        }
    }

    pub fn newtype(name: &'a str, ty: &'a FieldType) -> Self {
        SwiftStruct {
            name,
            fields: vec![SwiftField {
                name: "value".to_string(),
                ty: ty.swift_type(),
                serde_name: "",
            }],
            newtype: true,
        }
    }

    pub fn with_fields(mut self, fields: &'a [types::Field]) -> Self {
        self.fields.extend(fields.iter().map(Into::into));
        self
    }
}

struct SwiftField<'a> {
    name: String,
    ty: String,
    serde_name: &'a str,
}

impl<'a> From<&'a types::Field> for SwiftField<'a> {
    fn from(val: &'a types::Field) -> Self {
        SwiftField {
            name: to_camel_case(&val.name),
            ty: val.ty.swift_type(),
            serde_name: &val.serialized_name,
        }
    }
}

impl fmt::Display for SwiftStruct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let impls = if self.newtype {
            "Hashable"
        } else {
            "Hashable, Codable"
        };
        writeln!(f, "public struct {}: {impls} {{", self.name)?;
        {
            let f = &mut indented(f);
            for SwiftField { name, ty, .. } in &self.fields {
                writeln!(f, "public var {name}: {ty}")?;
            }
            writeln!(f, "\npublic init(")?;
            for SwiftField { name, ty, .. } in &self.fields {
                writeln!(indented(f), "{name}: {ty},")?;
            }
            writeln!(f, ") {{")?;
            for SwiftField { name, .. } in &self.fields {
                writeln!(indented(f), "self.{name} = {name}")?;
            }
            writeln!(f, "}}\n")?;

            if !self.newtype {
                writeln!(f, "enum CodingKeys: String, CodingKey {{")?;
                for SwiftField {
                    name, serde_name, ..
                } in &self.fields
                {
                    writeln!(indented(f), r#"case {name} = "{serde_name}""#,)?;
                }
                writeln!(f, "}}")?;
            }
        }
        writeln!(f, "}}")?;

        if self.newtype {
            let name = self.name;
            let field = self
                .fields
                .first()
                .expect("new types to have a single field");
            let ty = &field.ty;
            let field_name = &field.name;
            writedoc!(
                f,
                r#"

                    extension {name}: Decodable {{
                        init(from decoder: Decoder) throws {{
                            let container = try decoder.singleValueContainer()
                            let value = try decoder.decode({ty}.self)
                            {name}(value)
                        }}
                    }}

                    extension {name}: Encodable {{
                        func encode(to encoder: Encoder) throws {{
                            var container = try encoder.singleValueContainer()
                            try container.encode(self.{field_name})
                        }}
                    }}
                "#,
            )?;
        }

        Ok(())
    }
}
