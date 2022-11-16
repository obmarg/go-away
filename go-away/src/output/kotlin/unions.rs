use crate::{
    output::prelude::*,
    types::{self, UnionRepresentation},
};

use super::{
    data_classes::{DataClass, NewTypeClass},
    to_camel_case,
};

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
    inner_serializer: String,
    serde_name: &'a str,
}

impl<'a> Variant<'a> {
    fn newtype(&'a self, enum_class: &'a str) -> NewTypeClass<'a> {
        NewTypeClass::new(&self.name, self.ty.clone(), self.inner_serializer.clone())
            .with_inheritance(enum_class)
    }
}

impl<'a> From<&'a types::UnionVariant> for Variant<'a> {
    fn from(val: &'a types::UnionVariant) -> Self {
        Variant {
            // TODO: This should not be camel cased, but removing that change causes
            // null pointer exceptions :sob:
            name: to_camel_case(
                val.name
                    .as_ref()
                    .expect("union variants to generally have names"),
            ),
            inner_serializer: val.ty.serializer(),
            ty: val.ty.kotlin_type(),
            serde_name: &val.serialized_name,
        }
    }
}

impl fmt::Display for Union<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = &self.name;
        let serializer_name = format!("{name}Serializer");
        writeln!(f, "@Serializable(with = {serializer_name}::class)")?;
        writeln!(f, "sealed interface {name} {{")?;
        writeln_for!(
            indented(f),
            newtype in self.variants.iter().map(|v| v.newtype(name)),
            "{newtype}"
        );
        writeln!(f, "}}")?;

        match &self.representation {
            UnionRepresentation::AdjacentlyTagged { tag, content } => todo!(),
            UnionRepresentation::InternallyTagged { tag } => todo!(),
            UnionRepresentation::ExternallyTagged => {
                writeln!(f, "object {serializer_name} : KSerializer<{name}> {{\n")?;
                let f2 = &mut indented(f);
                writeln!(
                    f2,
                    r#"override val descriptor: SerialDescriptor = buildClassSerialDescriptor("{name}") {{"#
                )?;
                writeln_for!(
                    indented(f2),
                    Variant { name: variant_name, serde_name, .. } in &self.variants,
                    r#"element<{name}.{variant_name}>("{serde_name}", isOptional = true)"#
                );
                writeln!(f2, "}};\n")?;
                let f2 = &mut indented(f);
                writeln!(
                    f2,
                    "override fun serialize(encoder: Encoder, value: {name}) {{",
                )?;
                let f3 = &mut indented(f2);
                writeln!(f3, "val composite = encoder.beginStructure(descriptor)")?;
                writeln!(f3, "when(value) {{")?;
                writedoc_for!(
                    indented(f3),
                    (i, Variant { name: variant_name, .. }) in self.variants.iter().enumerate(),
                    r#"
                        is {name}.{variant_name} ->
                          composite.encodeSerializableElement(descriptor, {i}, {name}.{variant_name}.serializer(), value as {name}.{variant_name})
                    "#
                );
                writeln!(f3, "}}")?;
                writeln!(f3, "composite.endStructure(descriptor)")?;
                writeln!(f2, "}}\n")?;
                writeln!(f2, "override fun deserialize(decoder: Decoder): {name} {{")?;
                let f3 = &mut indented(f2);
                writeln!(f3, "val composite = decoder.beginStructure(descriptor)")?;
                writeln!(
                    f3,
                    "val rv = when (val index = composite.decodeElementIndex(descriptor)) {{"
                )?;
                writedoc_for!(
                    indented(f3),
                    (i, Variant { name: variant_name, .. }) in self.variants.iter().enumerate(),
                    r#"
                        {i} -> composite.decodeSerializableElement(descriptor, {i}, {name}.{variant_name}.serializer())
                    "#
                );
                writeln!(indented(f3), r#"else -> error("Unexpected input")"#)?;
                writeln!(f3, "}}")?;
                writeln!(f3, "composite.endStructure(descriptor)")?;
                writeln!(f3, "return rv")?;
                writeln!(f2, "}}")?;
                writeln!(f, "}}")?;
            }
            UnionRepresentation::Untagged => todo!(),
        }

        // Note: We won't need a Codable impl _if_ we've got an externally tagged union that
        // has named struct fields inside it (which isn't even possible rn, :sigh:)
        //
        // In this case we'd just need some CodableKeys impls for each variant.

        /* TODO:
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
                   UnionRepresentation::Untagged => todo!("Support untagged variants for kotlin"),
               }
               writeln!(f, "{codable}")?;
        */
        Ok(())
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
            "var container = encoder.container(keyedBy: {coding_keys}.self)"
        )?;
        writeln!(f, "switch self {{")?;
        writedoc_for!(
            indented(f),
            Variant { name, ..  } in variants.iter(),
            r#"
                        case .{name}(let data):
                            return try container.encode(data, forKey: .{name})
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
            "let container = try decoder.container(keyedBy: {coding_keys}.self)"
        )?;
        writedoc_for!(
            &mut *f,
            Variant { name, ty, ..  } in variants.iter(),
            r#"
                if (container.contains(.{name})) {{
                    self = .{name}(try container.decode({ty}.self, forKey: .{name}))
                    return
                }}
            "#
        );
        writedoc!(
            f,
            r#"
                throw NSError(
                    domain: "",
                    code: 400,
                    userInfo: [ NSLocalizedDescriptionKey: "Unknown variant of {name}"]
                )"#
        )
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
                let keyContainer = try decoder.container(keyedBy: TagCoding.self)
                let key = try keyContainer.decode({coding_keys}.self, forKey: .tag)
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
                    self = .{name}(try container.decode({ty}.self))
            "#
        );
        writeln!(f, "}}")
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
                    case data = "{content}"
                }}
                var container = encoder.container(keyedBy: ContainerKeys.self)
                switch self {{
            "#
        )?;
        writedoc_for!(
            indented(f),
            Variant { name,  ..  } in *variants,
            r#"
                case .{name}(let data):
                    try container.encode({coding_keys}.{name}, forKey: .tag)
                    try container.encode(data, forKey: .data)
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
                let container = try decoder.container(keyedBy: ContainerKeys.self)
                let key = try container.decode({coding_keys}.self, forKey: .tag)
                switch key {{
            "#
        )?;
        writedoc_for!(
            indented(f),
            Variant { name, ty, ..  } in *variants,
            r#"
                case .{name}:
                    self = .{name}(try container.decode({ty}.self, forKey: .data))
            "#
        );
        writeln!(f, "}}")
    }
}
