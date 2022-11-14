use std::fmt::{self, Write};

use indenter::indented;
use indoc::writedoc;

use super::to_camel_case;
use crate::types::{self, FieldType};

pub struct SwiftStruct<'a> {
    name: &'a str,
    fields: Vec<SwiftField>,
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
                serialized_name: None,
            }],
            newtype: true,
        }
    }

    pub fn with_fields(mut self, fields: &'a [types::Field]) -> Self {
        self.fields.extend(fields.iter().map(Into::into));
        self
    }
}

struct SwiftField {
    name: String,
    ty: String,
    serialized_name: Option<String>,
}

impl From<&types::Field> for SwiftField {
    fn from(val: &types::Field) -> Self {
        SwiftField {
            name: to_camel_case(&val.name),
            ty: val.ty.swift_type(),
            serialized_name: Some(val.serialized_name.clone()),
        }
    }
}

impl fmt::Display for SwiftStruct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "public struct {}: ", self.name)?;
        if self.newtype {
            writeln!(f, "Hashable {{")?;
        } else {
            writeln!(f, "Hashable, Codable {{")?;
        }
        {
            let f = &mut indented(f);
            for field in &self.fields {
                writeln!(f, "public var {}: {}", field.name, field.ty)?;
            }
            writeln!(f, "\npublic init(")?;
            for field in &self.fields {
                writeln!(indented(f), "{}: {},", field.name, field.ty)?;
            }
            writeln!(f, ") {{")?;
            for field in &self.fields {
                let name = &field.name;
                writeln!(indented(f), "self.{name} = {name}")?;
            }
            writeln!(f, "}}\n")?;

            if !self.newtype {
                writeln!(f, "enum CodingKeys: String, CodingKey {{")?;
                for field in &self.fields {
                    writeln!(
                        indented(f),
                        r#"case {} = "{}""#,
                        &field.name,
                        field.serialized_name.as_ref().unwrap_or(&field.name)
                    )?;
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
