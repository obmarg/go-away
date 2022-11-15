use crate::{
    output::prelude::*,
    types::{self, UnionRepresentation},
};

use super::{to_camel_case, CodingKey, CodingKeys};

pub struct Enum<'a> {
    name: &'a str,
    variants: Vec<Variant<'a>>,
}

impl<'a> Enum<'a> {
    pub fn new(name: &'a str) -> Enum {
        Enum {
            name,
            variants: Vec::new(),
        }
    }

    pub fn with_variants(mut self, fields: &'a [types::EnumVariant]) -> Self {
        self.variants.extend(fields.iter().map(Into::into));
        self
    }
}

struct Variant<'a> {
    name: String,
    serde_name: &'a str,
}

impl<'a> From<&'a types::EnumVariant> for Variant<'a> {
    fn from(val: &'a types::EnumVariant) -> Self {
        Variant {
            name: to_camel_case(&val.name),
            serde_name: &val.serialized_name,
        }
    }
}

impl fmt::Display for Enum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = &self.name;
        writeln!(f, "public enum {name} : Codable {{")?;
        writeln_for!(indented(f), Variant{ name, ..} in &self.variants, "case {name}");
        let coding_keys = CodingKeys::new().with_fields(&self.variants);
        writeln!(indented(f), "{coding_keys}")?;
        writeln!(f, "}}\n")
    }
}

impl<'a> From<&'a Variant<'a>> for CodingKey<'a> {
    fn from(variant: &'a Variant<'a>) -> Self {
        CodingKey {
            name: &variant.name,
            serde_name: variant.serde_name,
        }
    }
}
