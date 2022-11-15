use std::fmt::Display;

use crate::output::prelude::*;

pub struct Codable<'a> {
    name: &'a str,
    encodable_impl: String,
    decodable_impl: String,
}

impl<'a> Codable<'a> {
    pub fn new(name: &'a str) -> Self {
        Codable {
            name,
            encodable_impl: String::new(),
            decodable_impl: String::new(),
        }
    }

    pub fn encodable(&mut self, encodable: impl Display) {
        let mut f = indented(&mut self.encodable_impl);
        write!(indented(&mut f), "{encodable}").unwrap();
    }

    pub fn with_encodable(mut self, encodable: impl Display) -> Self {
        self.encodable(encodable);
        self
    }

    pub fn decodable(&mut self, decodable: impl Display) {
        let mut f = indented(&mut self.decodable_impl);
        write!(indented(&mut f), "{decodable}").unwrap();
    }

    pub fn with_decodable(mut self, decodable: impl Display) -> Self {
        self.decodable(decodable);
        self
    }
}

impl Display for Codable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Codable {
            name,
            encodable_impl,
            decodable_impl,
        } = self;

        writedoc!(
            f,
            r#"
                extension {name}: Decodable {{
                    public init(from decoder: Decoder) throws {{
                {decodable_impl}
                    }}
                }}

                extension {name}: Encodable {{
                    public func encode(to encoder: Encoder) throws {{
                {encodable_impl}
                    }}
                }}
            "#
        )
    }
}
