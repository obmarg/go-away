use crate::{
    output::{prelude::*, swift::enums::Enum},
    types::{self, Alias, Field, NewType, Struct},
};

use self::{
    coding_keys::{CodingKey, CodingKeys},
    structs::SwiftStruct,
    unions::Union,
};

use super::go::FieldType;

mod coding_keys;
mod enums;
mod structs;
mod unions;

mod codable;
#[cfg(test)]
mod tests;

/// An enum representing the possible top-level types in Kotlin
///
/// This shouldn't be instaniated directly but passed using turbofish operator
/// to registry_to_output enabling it to write out in Kotlin
pub enum SwiftType<'a> {
    /// A struct variant
    Struct(&'a Struct),
    /// A new type variant
    NewType(&'a NewType),
    /// A type alias variant
    Alias(&'a Alias),
    /// A simple enum variant (does not contain data)
    Enum(&'a types::Enum),
    /// A union variant (enums with data)
    Union(&'a types::Union),
}

impl fmt::Display for SwiftType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwiftType::Struct(details) => {
                let struct_ = SwiftStruct::new(&details.name).with_fields(&details.fields);
                writeln!(f, "{struct_}")?;
            }
            SwiftType::NewType(details) => {
                let struct_ = SwiftStruct::newtype(&details.name, &details.inner);
                writeln!(f, "{struct_}")?;
            }
            SwiftType::Alias(details) => {
                writeln!(
                    f,
                    "typealias {} = {}",
                    details.name,
                    details.inner.swift_type()
                )?;
            }
            SwiftType::Enum(details) => {
                let enum_ = Enum::new(&details.name).with_variants(&details.variants);
                writeln!(f, "{enum_}")?;
            }
            SwiftType::Union(details) => {
                let union_ = Union::new(&details.name, details.representation.clone())
                    .with_variants(&details.variants);
                writeln!(f, "{union_}")?;
            }
        }
        Ok(())
    }
}

pub struct SwiftField<'a>(&'a Field);

impl fmt::Display for SwiftField<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let details = self.0;
        write!(
            f,
            r#"public var {}: {}"#,
            to_camel_case(&details.name),
            details.ty.swift_type(),
        )
    }
}

impl FieldType {
    fn swift_type(&self) -> String {
        use crate::types::Primitive;

        match self {
            FieldType::Named(type_ref) => type_ref.name().to_string(),
            FieldType::Optional(inner) => format!("{}?", inner.swift_type()),
            FieldType::List(inner) => format!("[{}]", inner.swift_type()),
            FieldType::Map { key, value } => {
                format!("[{}: {}]", key.swift_type(), value.swift_type())
            }
            FieldType::Primitive(Primitive::String) => "String".to_string(),
            FieldType::Primitive(Primitive::Float) => "Double".to_string(),
            FieldType::Primitive(Primitive::Int) => "Int64".to_string(),
            FieldType::Primitive(Primitive::Bool) => "Bool".to_string(),
            FieldType::Primitive(Primitive::Time) => {
                // Also: is this a datetime or just a time.  Might need to expand the primitive support somewhat...
                todo!("Need to implement time support for swift")
            }
        }
    }
}

fn to_camel_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev_is_underscore = false;
    let mut first = true;
    for c in s.chars() {
        if c == '_' {
            prev_is_underscore = true;
        } else if prev_is_underscore {
            buf.push(c.to_ascii_uppercase());
            prev_is_underscore = false;
        } else if c.is_uppercase() && !first {
            buf.push(c);
        } else {
            buf.push(c.to_ascii_lowercase());
        }
        first = false;
    }
    buf
}
