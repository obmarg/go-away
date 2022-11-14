use std::fmt::{self, Write};

use indenter::indented;

use super::to_camel_case;
use crate::types::{self, FieldType};

pub struct SwiftStruct<'a> {
    name: &'a str,
    fields: Vec<SwiftField>,
}

impl<'a> SwiftStruct<'a> {
    pub fn new(name: &'a str) -> SwiftStruct {
        SwiftStruct {
            name,
            fields: Vec::new(),
        }
    }

    pub fn with_field(mut self, name: &'a str, ty: &'a FieldType) -> Self {
        self.fields.push(SwiftField {
            name: to_camel_case(name),
            ty: ty.swift_type(),
        });
        self
    }

    pub fn with_fields(mut self, fields: &'a [types::Field]) -> Self {
        self.fields.extend(fields.iter().map(Into::into));
        self
    }
}

struct SwiftField {
    name: String,
    ty: String,
}

impl From<&types::Field> for SwiftField {
    fn from(val: &types::Field) -> Self {
        SwiftField {
            name: to_camel_case(&val.name),
            ty: val.ty.swift_type(),
        }
    }
}

impl fmt::Display for SwiftStruct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "public struct {}: Hashable {{", self.name)?;
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
            writeln!(f, "}}")?;
        }
        writeln!(f, "}}")?;

        // TODO: encodable/decodable etc.

        Ok(())
    }
}
