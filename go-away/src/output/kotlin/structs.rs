use std::fmt::Display;

use types::NewType;

use super::to_camel_case;
use crate::{
    output::prelude::*,
    types::{self, FieldType},
};

pub enum KotlinStruct<'a> {
    Normal(NormalClass<'a>),
    NewType(NewTypeClass<'a>),
}

pub struct NormalClass<'a> {
    name: &'a str,
    fields: Vec<KotlinField<'a>>,
}

pub struct NewTypeClass<'a> {
    name: &'a str,
    ty: String,
    default_str: &'a str,
    serializer: String,
}

impl<'a> KotlinStruct<'a> {
    pub fn new(name: &'a str) -> KotlinStruct {
        KotlinStruct::Normal(NormalClass {
            name,
            fields: Vec::new(),
        })
    }

    pub fn newtype(name: &'a str, ty: &'a FieldType) -> Self {
        KotlinStruct::NewType(NewTypeClass {
            name,
            ty: ty.kotlin_type(),
            default_str: ty.default_str(),
            serializer: ty.serializer(),
        })
    }

    pub fn with_fields(mut self, new_fields: &'a [types::Field]) -> Self {
        let KotlinStruct::Normal(NormalClass { fields, .. }) = &mut self else {
            panic!("Called with_fields on a newtype");
        };
        fields.extend(new_fields.iter().map(Into::into));
        self
    }
}

struct KotlinField<'a> {
    name: String,
    ty: String,
    serde_name: &'a str,
    default_str: &'a str,
}

impl<'a> From<&'a types::Field> for KotlinField<'a> {
    fn from(val: &'a types::Field) -> Self {
        KotlinField {
            name: to_camel_case(&val.name),
            ty: val.ty.kotlin_type(),
            serde_name: &val.serialized_name,
            default_str: val.ty.default_str(),
        }
    }
}

impl fmt::Display for KotlinStruct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KotlinStruct::Normal(inner) => write!(f, "{inner}"),
            KotlinStruct::NewType(inner) => write!(f, "{inner}"),
        }
    }
}

impl fmt::Display for NormalClass<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name;
        writeln!(f, "@Serializable")?;
        writeln!(f, "data class {name}(")?;
        writedoc_for!(
            indented(f),
            KotlinField { name, ty, default_str, serde_name } in &self.fields,
            r#"
                @SerialName("{serde_name}")
                public var {name}: {ty}{default_str},
            "#
        );
        writeln!(f, ")\n")?;

        // TODO: any serializable stuff we need...

        Ok(())
    }
}

impl fmt::Display for NewTypeClass<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let NewTypeClass {
            name,
            ty,
            default_str,
            serializer: inner_serializer,
        } = self;

        writeln!(f, "@Serializable(with = {name}Serializer::class)")?;
        writeln!(
            f,
            "data class {name}(public var value: {ty} {default_str})\n"
        )?;

        writedoc!(
            f,
            r#"
                object {name}Serializer : KSerializer<{name}> {{
                    private val serializer = {inner_serializer};

                    override val descriptor: SerialDescriptor = serialDescriptor<{ty}>()

                    override fun serialize(encoder: Encoder, value: {name}) {{
                        encoder.serialize(serializer, value.value)
                    }}

                    override fun deserialize(decoder: Decoder): {name} {{
                        return {name}(decoder.decode(serializer))
                    }}
                }}
            "#
        )
    }
}

impl FieldType {
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
