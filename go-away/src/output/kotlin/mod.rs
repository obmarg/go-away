use crate::{
    output::prelude::*,
    types::{self, Alias, NewType, Struct},
};

use self::{enums::Enum, structs::KotlinStruct, unions::Union};

use super::go::FieldType;

mod data_classes;
mod enums;
mod kserializer;
mod structs;
mod unions;

#[cfg(test)]
mod tests;

/// An enum representing the possible top-level types in Kotlin
///
/// This shouldn't be instaniated directly but passed using turbofish operator
/// to registry_to_output enabling it to write out in Kotlin
pub enum KotlinType<'a> {
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

impl fmt::Display for KotlinType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KotlinType::Struct(details) => {
                let struct_ = KotlinStruct::new(&details.name).with_fields(&details.fields);
                writeln!(f, "{struct_}")?;
            }
            KotlinType::NewType(details) => {
                let struct_ = KotlinStruct::newtype(&details.name, &details.inner);
                writeln!(f, "{struct_}")?;
            }
            KotlinType::Alias(details) => {
                writeln!(
                    f,
                    "typealias {} = {}",
                    details.name,
                    details.inner.kotlin_type()
                )?;
            }
            KotlinType::Enum(details) => {
                let enum_ = Enum::new(&details.name).with_variants(&details.variants);
                writeln!(f, "{enum_}")?;
            }
            KotlinType::Union(details) => {
                let union_ = Union::new(&details.name, details.representation.clone())
                    .with_variants(&details.variants);
                writeln!(f, "{union_}")?;
            }
        }
        Ok(())
    }
}

impl FieldType {
    fn kotlin_type(&self) -> String {
        use crate::types::Primitive;

        match self {
            FieldType::Named(type_ref) => type_ref.name().to_string(),
            FieldType::Optional(inner) => format!("{}?", inner.kotlin_type()),
            FieldType::List(inner) => format!("List<{}>", inner.kotlin_type()),
            FieldType::Map { key, value } => {
                format!("Map<{}, {}>", key.kotlin_type(), value.kotlin_type())
            }
            FieldType::Primitive(Primitive::String) => "String".to_string(),
            FieldType::Primitive(Primitive::Float) => "Double".to_string(),
            FieldType::Primitive(Primitive::Int) => "Long".to_string(),
            FieldType::Primitive(Primitive::Bool) => "Boolean".to_string(),
            FieldType::Primitive(Primitive::Time) => {
                // Also: is this a datetime or just a time.  Might need to expand the primitive support somewhat...
                todo!("Need to implement time support for kotlin")
            }
        }
    }

    fn default_str(&self) -> &'static str {
        match self {
            FieldType::Optional(_) => " = null",
            _ => "",
        }
    }

    fn serializer(&self) -> String {
        match self {
            FieldType::Optional(inner) => {
                format!("{}.nullable", inner.serializer())
            }
            FieldType::List(inner) => {
                format!("ListSerializer({})", inner.serializer())
            }
            FieldType::Map { key, value } => {
                format!(
                    "MapSerializer({}, {})",
                    key.serializer(),
                    value.serializer()
                )
            }
            FieldType::Named(_) | FieldType::Primitive(_) => {
                format!("{}.serializer()", self.kotlin_type())
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

fn to_screaming_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut first = true;
    for c in s.chars() {
        if c.is_uppercase() {
            if !first {
                buf.push('_');
            }
            buf.push(c);
        } else {
            buf.push(c.to_ascii_uppercase());
        }
        first = false;
    }
    buf
}
