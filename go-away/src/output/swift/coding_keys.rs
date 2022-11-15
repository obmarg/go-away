use std::fmt::{self, Write};

use indenter::indented;

#[derive(Default)]
pub struct CodingKeys<'a> {
    name: Option<&'a str>,
    fields: Vec<CodingKey<'a>>,
}

pub struct CodingKey<'a> {
    pub name: &'a str,
    pub serde_name: &'a str,
}

impl<'a> CodingKeys<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    #[allow(dead_code)]
    pub fn with_name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_fields<I, T>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<CodingKey<'a>>,
    {
        self.fields.extend(fields.into_iter().map(Into::into));
        self
    }
}

impl fmt::Display for CodingKeys<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.unwrap_or("CodingKeys");

        writeln!(f, "enum {name}: String, CodingKey, Codable {{")?;
        for CodingKey { name, serde_name } in &self.fields {
            writeln!(indented(f), r#"case {name} = "{serde_name}""#,)?;
        }
        write!(f, "}}")
    }
}
