use std::default;

use crate::output::prelude::*;

use super::to_camel_case;

pub struct DataClass<'a> {
    name: &'a str,
    inherits: Vec<&'a str>,
    fields: Vec<Field<'a>>,
    serializer: Option<String>,
}

pub struct Field<'a> {
    name: String,
    ty: String,
    serde_name: &'a str,
    default_str: &'a str,
}

impl<'a> DataClass<'a> {
    pub fn new(name: &'a str) -> Self {
        DataClass {
            name,
            inherits: Vec::new(),
            fields: Vec::new(),
            serializer: None,
        }
    }

    pub fn add_fields(&mut self, new_fields: impl IntoIterator<Item = Field<'a>>) {
        self.fields.extend(new_fields);
    }

    pub fn with_fields(mut self, new_fields: impl IntoIterator<Item = Field<'a>>) -> Self {
        self.add_fields(new_fields);
        self
    }

    pub fn with_inheritance(mut self, superclass: &'a str) -> Self {
        self.inherits.push(superclass);
        self
    }

    pub fn serialize_with(mut self, name: String) -> Self {
        self.serializer = Some(name);
        self
    }
}

impl fmt::Display for DataClass<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name;
        write!(f, "@Serializable")?;
        if let Some(serializer) = &self.serializer {
            write!(f, "(with = {serializer})")?;
        }
        writeln!(f, "\ndata class {name}(")?;
        writedoc_for!(
            indented(f),
            Field { name, ty, default_str, serde_name } in &self.fields,
            r#"
                @SerialName("{serde_name}")
                public var {name}: {ty}{default_str},
            "#
        );
        write!(f, ")")?;
        if !self.inherits.is_empty() {
            let inherits = self.inherits.join(", ");
            write!(f, ": {inherits}")?;
        }
        writeln!(f, "\n")
    }
}

pub struct NewTypeClass<'a> {
    name: &'a str,
    inner_serializer: String,
    dataclass: DataClass<'a>,
}

impl<'a> NewTypeClass<'a> {
    pub fn new(name: &'a str, ty: String, inner_serializer: String) -> Self {
        NewTypeClass {
            name,
            inner_serializer,
            dataclass: DataClass::new(name)
                .with_fields([Field {
                    name: "value".to_string(),
                    ty,
                    serde_name: "value",
                    default_str: "",
                }])
                .serialize_with(format!("{}::class", serializer_name(name))),
        }
    }

    pub fn with_default_string(mut self, default_str: &'a str) -> Self {
        self.dataclass.fields.first_mut().unwrap().default_str = default_str;
        self
    }

    pub fn with_inheritance(mut self, superclass: &'a str) -> Self {
        self.dataclass.inherits.push(superclass);
        self
    }
}

fn serializer_name(name: &str) -> String {
    format!("{name}Serializer")
}

impl fmt::Display for NewTypeClass<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let NewTypeClass {
            name,
            inner_serializer,
            dataclass,
        } = self;
        let serializer_name = serializer_name(name);

        writeln!(f, "{dataclass}")?;

        writedoc!(
            f,
            r#"
                object {serializer_name} : KSerializer<{name}> {{
                    private val serializer = {inner_serializer};

                    override val descriptor: SerialDescriptor = serializer.descriptor;

                    override fun serialize(encoder: Encoder, value: {name}) {{
                        encoder.encodeSerializableValue(serializer, value.value)
                    }}

                    override fun deserialize(decoder: Decoder): {name} {{
                        return {name}(decoder.decodeSerializableValue(serializer))
                    }}
                }}
            "#
        )
    }
}

impl<'a> From<&'a crate::types::Field> for Field<'a> {
    fn from(val: &'a crate::types::Field) -> Self {
        Field {
            name: to_camel_case(&val.name),
            ty: val.ty.kotlin_type(),
            serde_name: &val.serialized_name,
            default_str: val.ty.default_str(),
        }
    }
}
