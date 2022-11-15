use crate::{
    output::{prelude::*, swift::codable::Codable},
    types::{self, UnionRepresentation},
};

use super::{to_camel_case, CodingKey, CodingKeys};

pub struct Union<'a> {
    name: &'a str,
    variants: Vec<Variant<'a>>,
    representation: UnionRepresentation,
}

impl<'a> Union<'a> {
    pub fn new(name: &'a str, representation: UnionRepresentation) -> Union {
        Union {
            name,
            variants: Vec::new(),
            representation,
        }
    }

    pub fn with_variants(mut self, fields: &'a [types::UnionVariant]) -> Self {
        self.variants.extend(fields.iter().map(Into::into));
        self
    }
}

struct Variant<'a> {
    name: String,
    ty: String,
    serde_name: &'a str,
}

impl<'a> From<&'a types::UnionVariant> for Variant<'a> {
    fn from(val: &'a types::UnionVariant) -> Self {
        Variant {
            name: to_camel_case(
                val.name
                    .as_ref()
                    .expect("union variants to generally have names"),
            ),
            ty: val.ty.swift_type(),
            serde_name: &val.serialized_name,
        }
    }
}

impl fmt::Display for Union<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = &self.name;
        writeln!(f, "public enum {name} {{")?;
        writeln_for!(
            indented(f),
            Variant{ name, ty, .. } in &self.variants,
            "case {name}({ty})"
        );
        let coding_keys = CodingKeys::new().with_fields(&self.variants);
        writeln!(indented(f), "{coding_keys}")?;
        writeln!(f, "}}\n")?;

        // Note: We won't need a Codable impl _if_ we've got an externally tagged union that
        // has named struct fields inside it (which isn't even possible rn, :sigh:)
        //
        // In this case we'd just need some CodableKeys impls for each variant.

        let mut codable = Codable::new(self.name);

        match &self.representation {
            UnionRepresentation::AdjacentlyTagged { tag, content } => {
                codable.encodable(AdjacentlyTaggedEncodable {
                    name: self.name,
                    variants: &self.variants,
                    tag,
                    content,
                });
                codable.decodable(AdjacentlyTaggedDecodable {
                    name: self.name,
                    variants: &self.variants,
                    tag,
                    content,
                });
            }
            UnionRepresentation::InternallyTagged { tag } => {
                codable.encodable(InternallyTaggedEncodable {
                    name: self.name,
                    variants: &self.variants,
                    tag,
                });
                codable.decodable(InternallyTaggedDecodable {
                    name: self.name,
                    variants: &self.variants,
                    tag,
                });
            }
            UnionRepresentation::ExternallyTagged => {
                codable.encodable(ExternallyTaggedEncodable {
                    name,
                    variants: &self.variants,
                });
                codable.decodable(ExternallyTaggedDecodable {
                    name,
                    variants: &self.variants,
                });
            }
            UnionRepresentation::Untagged => todo!("Support untagged variants for swift"),
        }
        writeln!(f, "{codable}")?;

        Ok(())
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

struct ExternallyTaggedEncodable<'a> {
    name: &'a str,
    variants: &'a [Variant<'a>],
}

impl fmt::Display for ExternallyTaggedEncodable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ExternallyTaggedEncodable { name, variants } = self;
        let coding_keys = format!("{name}.CodingKeys");

        writeln!(
            f,
            "let container = try encoder.container({coding_keys}.self)"
        )?;
        writeln!(f, "switch self {{")?;
        writedoc_for!(
            indented(f),
            Variant { name, ..  } in variants.iter(),
            r#"
                        case .{name}(data):
                            return container.encode(data, forKey: .{name})
                    "#
        );
        write!(f, "}}")
    }
}

struct ExternallyTaggedDecodable<'a> {
    name: &'a str,
    variants: &'a [Variant<'a>],
}

impl fmt::Display for ExternallyTaggedDecodable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ExternallyTaggedDecodable { name, variants } = self;
        let coding_keys = format!("{name}.CodingKeys");

        writeln!(
            f,
            "let container = try decoder.container({coding_keys}.self)"
        )?;
        writedoc_for!(
            f,
            Variant { name, ty, ..  } in variants.iter(),
            r#"
                if (container.contains(.{name})) {{
                    return try container.decode({ty}.self, forKey: .{name})
                }}
            "#
        );
        Ok(())
    }
}

#[allow(dead_code)]
struct InternallyTaggedEncodable<'a> {
    name: &'a str,
    tag: &'a str,
    variants: &'a [Variant<'a>],
}

impl fmt::Display for InternallyTaggedEncodable<'_> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Finish internally tagged enum support")
    }
}

struct InternallyTaggedDecodable<'a> {
    name: &'a str,
    tag: &'a str,
    variants: &'a [Variant<'a>],
}

impl fmt::Display for InternallyTaggedDecodable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let InternallyTaggedDecodable {
            name,
            variants,
            tag,
        } = self;
        let coding_keys = format!("{name}.CodingKeys");

        writedoc!(
            f,
            r#"
                enum TagCoding: String, CodingKey {{
                    case tag = "{tag}"
                }}
                let keyContainer = try decoder.container(TagCoding.self)
                let key = try decoder.decode({coding_keys}.self, forKey: .tag)
                switch key {{
            "#
        )?;
        writedoc_for!(
            indented(f),
            Variant { name, ty, ..  } in *variants,
            r#"
                case .{name}:
                    // Not 100% sure this'll work but
                    let container = try decoder.singleValueContainer()
                    return try container.decode({ty}.self)
                }}
            "#
        );
        writedoc!(
            f,
            r#"
                    default:
                        throw "Unknown variant"
                }}
            "#
        )
    }
}

#[allow(dead_code)]
struct AdjacentlyTaggedEncodable<'a> {
    name: &'a str,
    tag: &'a str,
    content: &'a str,
    variants: &'a [Variant<'a>],
}

impl fmt::Display for AdjacentlyTaggedEncodable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let AdjacentlyTaggedEncodable {
            name,
            variants,
            tag,
            content,
        } = self;
        let coding_keys = format!("{name}.CodingKeys");

        writedoc!(
            f,
            r#"
                enum ContainerKeys: String, CodingKey {{
                    case tag = "{tag}"
                    case data = "{content}
                }}
                let container = try encoder.container(ContainerKeys.self)
                switch this {{
            "#
        )?;
        writedoc_for!(
            indented(f),
            Variant { name,  ..  } in *variants,
            r#"
                case .{name}(data):
                    try container.encode({coding_keys}.name, forKey: .tag)
                    try container.encode(data, forKey: .data)
                }}
            "#
        );
        writeln!(f, "}}")
    }
}

struct AdjacentlyTaggedDecodable<'a> {
    name: &'a str,
    tag: &'a str,
    content: &'a str,
    variants: &'a [Variant<'a>],
}

impl fmt::Display for AdjacentlyTaggedDecodable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let AdjacentlyTaggedDecodable {
            name,
            variants,
            tag,
            content,
        } = self;
        let coding_keys = format!("{name}.CodingKeys");

        writedoc!(
            f,
            r#"
                enum ContainerKeys: String, CodingKey {{
                    case tag = "{tag}"
                    case data = "{content}"
                }}
                let container = try decoder.container(ContainerKeys.self)
                let key = try decoder.decode({coding_keys}.self, forKey: .tag)
                switch key {{
            "#
        )?;
        writedoc_for!(
            indented(f),
            Variant { name, ty, ..  } in *variants,
            r#"
                case .{name}:
                    return try container.decode({ty}.self, forKey: .data)
                }}
            "#
        );
        writedoc!(
            f,
            r#"
                    default:
                        throw "Unknown variant"
                }}
            "#
        )
    }
}
