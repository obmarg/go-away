use std::fmt::{self, Write};

use indenter::indented;
use indoc::formatdoc;

use super::{codable::Codable, to_camel_case, CodingKey, CodingKeys};
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
        let name = self.name;
        let impls = if self.newtype {
            "Hashable"
        } else {
            "Hashable, Codable"
        };
        writeln!(f, "public struct {name}: {impls} {{")?;
        {
            let f = &mut indented(f);
            for SwiftField { name, ty, .. } in &self.fields {
                writeln!(f, "public var {name}: {ty}")?;
            }
            writeln!(f, "\npublic init(")?;

            let fields = self
                .fields
                .iter()
                .map(|SwiftField { name, ty, .. }| format!("{name}: {ty}"))
                .collect::<Vec<_>>()
                .join(",\n");
            write!(indented(f), "{fields}")?;

            writeln!(f, ") {{")?;

            for SwiftField { name, .. } in &self.fields {
                writeln!(indented(f), "self.{name} = {name}")?;
            }
            writeln!(f, "}}")?;
        }
        writeln!(f, "}}\n")?;

        if !self.newtype {
            let coding_keys = CodingKeys::new().with_fields(&self.fields);
            writeln!(f, "extension {name} {{")?;
            writeln!(indented(f), "{coding_keys}")?;
            writeln!(f, "}}")?;
        } else {
            let field = self
                .fields
                .first()
                .expect("new types to have a single field");
            let ty = &field.ty;
            let field_name = &field.name;
            let codable = Codable::new(self.name)
                .with_decodable(formatdoc!(
                    r#"
                        let container = try decoder.singleValueContainer()
                        let value = try decoder.decode({ty}.self)
                        {name}(value)
                    "#
                ))
                .with_encodable(formatdoc!(
                    r#"
                        var container = try encoder.singleValueContainer()
                        try container.encode(self.{field_name})
                    "#
                ));
            writeln!(f, "\n{codable}")?;
        }

        Ok(())
    }
}

impl<'a> From<&'a SwiftField<'a>> for CodingKey<'a> {
    fn from(field: &'a SwiftField<'a>) -> Self {
        CodingKey {
            name: &field.name,
            serde_name: field.serde_name,
        }
    }
}
