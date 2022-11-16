use crate::{
    output::prelude::*,
    types::{self},
};

use super::to_screaming_snake_case;

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
            name: to_screaming_snake_case(&val.name),
            serde_name: &val.serialized_name,
        }
    }
}

impl fmt::Display for Enum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = &self.name;
        writeln!(f, "@Serializable")?;
        writeln!(f, "public enum class {name} {{")?;
        writeln_for!(
            indented(f),
            Variant{name, serde_name } in &self.variants,
            r#"@SerialName("{serde_name}") {name},"#
        );
        writeln!(f, "}}\n")
    }
}
