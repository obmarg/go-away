use insta::assert_snapshot;

use super::*;
use crate::types::{EnumVariant, Primitive, TypeRef, UnionRepresentation, UnionVariant};

#[test]
fn test_primitive_structs() {
    assert_snapshot!(
        KotlinType::Struct(&Struct {
            name: "MyStruct".into(),
            fields: vec![
                Field {
                    name: "a_string".into(),
                    serialized_name: "a_string".into(),
                    ty: FieldType::Primitive(Primitive::String),
                },
                Field {
                    name: "an_int".into(),
                    serialized_name: "renamed_tho".into(),
                    ty: FieldType::Primitive(Primitive::Int),
                },
                Field {
                    name: "a_bool".into(),
                    serialized_name: "also_renamed".into(),
                    ty: FieldType::Primitive(Primitive::Bool),
                },
                Field {
                    name: "a_float".into(),
                    serialized_name: "a_float".into(),
                    ty: FieldType::Primitive(Primitive::Float),
                },
                Field {
                    name: "optionalFloat".into(),
                    serialized_name: "a_float".into(),
                    ty: FieldType::Optional(Box::new(FieldType::Primitive(Primitive::Float))),
                },
            ],
        })
        .to_string(),
        @r###"
    @Serializable
    data class MyStruct(
        @SerialName("a_string")
        public var aString: String,
        @SerialName("renamed_tho")
        public var anInt: Long,
        @SerialName("also_renamed")
        public var aBool: Boolean,
        @SerialName("a_float")
        public var aFloat: Double,
        @SerialName("a_float")
        public var optionalFloat: Double? = null,
    )


    "###
    );
}

#[test]
fn test_newtype_output() {
    assert_snapshot!(KotlinType::NewType(&NewType {
            name: "UserId".into(),
            inner: FieldType::Primitive(Primitive::String),
        })
        .to_string(), @r###"
    @Serializable(with = UserIdSerializer::class)
    data class UserId(
        @SerialName("value")
        public var value: String,
    )


    object UserIdSerializer : KSerializer<UserId> {
        private val serializer = String.serializer();

        override val descriptor: SerialDescriptor = serializer.descriptor;

        override fun serialize(encoder: Encoder, value: UserId) {
            encoder.encodeSerializableValue(serializer, value.value)
        }

        override fun deserialize(decoder: Decoder): UserId {
            return UserId(decoder.decodeSerializableValue(serializer))
        }
    }

    "###);
}

#[test]
fn test_enum_output() {
    assert_snapshot!(KotlinType::Enum(&types::Enum {
            name: "FulfilmentType".into(),
            variants: vec![
                EnumVariant {
                    name: "Delivery".into(),
                    serialized_name: "DELIVERY".into(),
                },
                EnumVariant {
                    name: "Collection".into(),
                    serialized_name: "COLLECTION".into(),
                },
            ],
        })
        .to_string(), @r###"
    @Serializable
    public enum class FulfilmentType {
        @SerialName("DELIVERY") DELIVERY,
        @SerialName("COLLECTION") COLLECTION,
    }


    "###);
}

#[test]
fn test_adjacently_tagged_union_output() {
    assert_snapshot!(KotlinType::Union(&types::Union {
        name: "MyUnion".into(),
        representation: UnionRepresentation::AdjacentlyTagged {
            tag: "type".into(),
            content: "data".into(),
        },
        variants: vec![
            UnionVariant {
                name: Some("VarOne".into()),
                ty: FieldType::Named(TypeRef {
                    name: "VarOne".into()
                }),
                serialized_name: "VAR_ONE".into(),
            },
            UnionVariant {
                name: Some("VarTwo".into()),
                ty: FieldType::Named(TypeRef {
                    name: "VarTwo".into()
                }),
                serialized_name: "VAR_TWO".into(),
            }
        ]
    })
    .to_string());
}

#[test]
fn test_list_types() {
    assert_snapshot!(
        FieldType::List(Box::new(FieldType::Primitive(Primitive::String))).kotlin_type(),
        @"List<String>"
    );
}

#[test]
fn test_map_types() {
    assert_snapshot!(
        FieldType::Map{
            key: Box::new(FieldType::Primitive(Primitive::String)),
            value: Box::new(FieldType::Primitive(Primitive::Int))
        }.kotlin_type(),
        @"Map<String, Long>"
    );
}

#[test]
fn test_option_types() {
    assert_snapshot!(
        FieldType::Optional(Box::new(FieldType::Primitive(Primitive::String))).kotlin_type(),
        @"String?"
    );
}
