use std::fmt::Display;

use crate::output::prelude::*;

pub struct KSerializer<'a> {
    target_name: &'a str,
    serializer_name: String,
    serialize_body: String,
    deserialize_body: String,
    descriptor: String,
    additional_members: String,
}

impl<'a> KSerializer<'a> {
    pub fn new(target_name: &'a str) -> Self {
        KSerializer {
            target_name,
            serialize_body: String::new(),
            deserialize_body: String::new(),
            serializer_name: format!("{}Serializer", target_name),
            descriptor: String::new(),
            additional_members: String::new(),
        }
    }

    pub fn descriptor(&mut self, descriptor: impl Display) {
        self.descriptor = descriptor.to_string();
    }

    pub fn with_descriptor(mut self, descriptor: impl Display) -> Self {
        self.descriptor(descriptor);
        self
    }

    pub fn serialize_body(&mut self, serialize_body: impl Display) {
        let mut f = indented(&mut self.serialize_body);
        write!(indented(&mut f), "{serialize_body}").unwrap();
    }

    pub fn with_serialize_body(mut self, serialize_body: impl Display) -> Self {
        self.serialize_body(serialize_body);
        self
    }

    pub fn deserialize_body(&mut self, deserialize_body: impl Display) {
        let mut f = indented(&mut self.deserialize_body);
        write!(indented(&mut f), "{deserialize_body}").unwrap();
    }

    pub fn with_deserialize_body(mut self, deserialize_body: impl Display) -> Self {
        self.deserialize_body(deserialize_body);
        self
    }

    pub fn additional_members(&mut self, additional_members: impl Display) {
        let mut f = indented(&mut self.additional_members);
        write!(f, "{additional_members}").unwrap();
    }

    pub fn with_additional_members(mut self, additional_members: impl Display) -> Self {
        self.additional_members(additional_members);
        self
    }
}

impl Display for KSerializer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let KSerializer {
            target_name,
            serializer_name,
            serialize_body,
            deserialize_body,
            descriptor,
            additional_members,
        } = self;
        writedoc!(
            f,
            r#"
                object {serializer_name} : KSerializer<{target_name}> {{
                {additional_members}
            "#
        )?;
        writeln!(
            indented(f),
            "override val descriptor: SerialDescriptor = {descriptor}"
        )?;
        writedoc!(
            f,
            r#"
                    override fun serialize(encoder: Encoder, value: {target_name}) {{
                {serialize_body}
                    }}

                    override fun deserialize(decoder: Decoder): {target_name} {{
                {deserialize_body}
                    }}
                }}
            "#
        )
    }
}
