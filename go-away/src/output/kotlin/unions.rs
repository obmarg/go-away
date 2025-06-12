use crate::{
    output::{kotlin::kserializer::KSerializer, prelude::*},
    types::{self, UnionRepresentation},
};

use super::{data_classes::NewTypeClass, to_camel_case};

pub struct Union<'a> {
    name: &'a str,
    variants: Vec<Variant<'a>>,
    representation: UnionRepresentation,
}

impl<'a> Union<'a> {
    pub fn new(name: &'a str, representation: UnionRepresentation) -> Union<'a> {
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

        let mut serializer = KSerializer::new(name);
        match &self.representation {
            UnionRepresentation::AdjacentlyTagged { tag, content } => {
                serializer.serialize_body(AdjacentlyTaggedSerialize {
                    name: self.name,
                    variants: &self.variants,
                    tag,
                    content,
                });
                serializer.deserialize_body(AdjacentlyTaggedDeserialize {
                    name: self.name,
                    variants: &self.variants,
                    tag,
                    content,
                });
                serializer.descriptor(AdjacentlyTaggedDescriptor {
                    name: self.name,
                    variants: &self.variants,
                    tag,
                    content,
                });
            }
            UnionRepresentation::InternallyTagged { tag } => {
                serializer.serialize_body(InternallyTaggedSerialize {
                    name: self.name,
                    variants: &self.variants,
                    tag,
                });
                serializer.deserialize_body(InternallyTaggedDeserialize {
                    name: self.name,
                    variants: &self.variants,
                    tag,
                });
                serializer.descriptor(InternallyTaggedDescriptor {
                    name: self.name,
                    variants: &self.variants,
                    tag,
                });
            }
            UnionRepresentation::ExternallyTagged => {
                serializer.serialize_body(ExternallyTaggedSerialize {
                    name,
                    variants: &self.variants,
                });
                serializer.deserialize_body(ExternallyTaggedDeserialize {
                    name,
                    variants: &self.variants,
                });
                serializer.descriptor(ExternallyTaggedDescriptor {
                    name,
                    variants: &self.variants,
                });
            }
            UnionRepresentation::Untagged => todo!(),
        }
        writeln!(f, "{serializer}")?;
        Ok(())
    }
}

struct ExternallyTaggedSerialize<'a> {
    name: &'a str,
    variants: &'a [Variant<'a>],
}

impl fmt::Display for ExternallyTaggedSerialize<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ExternallyTaggedSerialize { name, variants } = self;
        writeln!(f, "val composite = encoder.beginStructure(descriptor)")?;
        writeln!(f, "when(value) {{")?;
        writedoc_for!(
            indented(f),
            (i, Variant { name: variant_name, .. }) in variants.iter().enumerate(),
            r#"
                is {name}.{variant_name} ->
                    composite.encodeSerializableElement(descriptor, {i}, {name}.{variant_name}.serializer(), value as {name}.{variant_name})
            "#
        );
        writeln!(f, "}}")?;
        writeln!(f, "composite.endStructure(descriptor)")
    }
}

struct ExternallyTaggedDeserialize<'a> {
    name: &'a str,
    variants: &'a [Variant<'a>],
}

impl fmt::Display for ExternallyTaggedDeserialize<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ExternallyTaggedDeserialize { name, variants } = self;
        writeln!(f, "val composite = decoder.beginStructure(descriptor)")?;
        writeln!(
            f,
            "val rv = when (val index = composite.decodeElementIndex(descriptor)) {{"
        )?;
        writedoc_for!(
            indented(f),
            (i, Variant { name: variant_name, .. }) in variants.iter().enumerate(),
            r#"
                        {i} -> composite.decodeSerializableElement(descriptor, {i}, {name}.{variant_name}.serializer())
                    "#
        );
        writeln!(indented(f), r#"else -> error("Unexpected input")"#)?;
        writeln!(f, "}}")?;
        writeln!(f, "composite.endStructure(descriptor)")?;
        writeln!(f, "return rv")
    }
}
struct ExternallyTaggedDescriptor<'a> {
    name: &'a str,
    variants: &'a [Variant<'a>],
}

impl fmt::Display for ExternallyTaggedDescriptor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ExternallyTaggedDescriptor { name, variants } = self;

        writeln!(f, r#"buildClassSerialDescriptor("{name}") {{"#)?;
        writeln_for!(
            indented(f),
            Variant { name: variant_name, serde_name, .. } in *variants,
            r#"element<{name}.{variant_name}>("{serde_name}", isOptional = true)"#
        );
        writeln!(f, "}};\n")
    }
}

#[allow(dead_code)]
struct InternallyTaggedSerialize<'a> {
    name: &'a str,
    tag: &'a str,
    variants: &'a [Variant<'a>],
}

impl fmt::Display for InternallyTaggedSerialize<'_> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Finish internally tagged enum support")
    }
}

#[allow(dead_code)]
struct InternallyTaggedDeserialize<'a> {
    name: &'a str,
    tag: &'a str,
    variants: &'a [Variant<'a>],
}

impl fmt::Display for InternallyTaggedDeserialize<'_> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Finish internally tagged enum support")
    }
}

#[allow(dead_code)]
struct InternallyTaggedDescriptor<'a> {
    name: &'a str,
    tag: &'a str,
    variants: &'a [Variant<'a>],
}

impl fmt::Display for InternallyTaggedDescriptor<'_> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Finish internally tagged enum support")
    }
}

#[allow(dead_code)]
struct AdjacentlyTaggedSerialize<'a> {
    name: &'a str,
    tag: &'a str,
    content: &'a str,
    variants: &'a [Variant<'a>],
}

impl fmt::Display for AdjacentlyTaggedSerialize<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let AdjacentlyTaggedSerialize { name, variants, .. } = self;
        writeln!(f, "val composite = encoder.beginStructure(descriptor)")?;
        writeln!(f, "when(value) {{")?;
        writedoc_for!(
            indented(f),
            Variant { name: variant_name, .. } in variants.iter(),
            r#"
                is {name}.{variant_name} ->
                    composite.encodeSerializableElement(descriptor, 0, String.serializer(), "{name}")
                    composite.encodeSerializableElement(descriptor, 1, {name}.{variant_name}.serializer(), value as {name}.{variant_name})
            "#
        );
        writeln!(f, "}}")?;
        writeln!(f, "composite.endStructure(descriptor)")
    }
}

#[allow(dead_code)]
struct AdjacentlyTaggedDeserialize<'a> {
    name: &'a str,
    tag: &'a str,
    content: &'a str,
    variants: &'a [Variant<'a>],
}

impl fmt::Display for AdjacentlyTaggedDeserialize<'_> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("impl adjacent tagging for kotlin")
    }
}

#[allow(dead_code)]
struct AdjacentlyTaggedDescriptor<'a> {
    name: &'a str,
    tag: &'a str,
    content: &'a str,
    variants: &'a [Variant<'a>],
}

impl fmt::Display for AdjacentlyTaggedDescriptor<'_> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("impl adjacent tagging for kotlin")
    }
}
