use insta::assert_snapshot;

use super::*;
use crate::types::{EnumVariant, Primitive, TypeRef, UnionRepresentation, UnionVariant};

#[test]
fn test_primitive_structs() {
    assert_snapshot!(
        SwiftType::Struct(&Struct {
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
            ],
        })
        .to_string(),
        @r###"
    public struct MyStruct: Hashable, Codable {
        public var aString: String
        public var anInt: Int64
        public var aBool: Bool
        public var aFloat: Double

        public init(
            aString: String,
            anInt: Int64,
            aBool: Bool,
            aFloat: Double,
        ) {
            self.aString = aString
            self.anInt = anInt
            self.aBool = aBool
            self.aFloat = aFloat
        }

        enum CodingKeys: String, CodingKey {
            case aString = "a_string"
            case anInt = "renamed_tho"
            case aBool = "also_renamed"
            case aFloat = "a_float"
        }
    }

    "###
    );
}

#[test]
fn test_newtype_output() {
    assert_snapshot!(SwiftType::NewType(&NewType {
            name: "UserId".into(),
            inner: FieldType::Primitive(Primitive::String),
        })
        .to_string(), @r###"
    public struct UserId: Hashable {
        public var value: String

        public init(
            value: String,
        ) {
            self.value = value
        }

    }

    extension UserId: Decodable {
        init(from decoder: Decoder) throws {
            let container = try decoder.singleValueContainer()
            let value = try decoder.decode(String.self)
            UserId(value)
        }
    }

    extension UserId: Encodable {
        func encode(to encoder: Encoder) throws {
            var container = try encoder.singleValueContainer()
            try container.encode(self.value)
        }
    }

    "###);
}

#[test]
fn test_enum_output() {
    assert_snapshot!(SwiftType::Enum(&Enum {
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
    public enum FulfilmentType {
        case delivery
        case collection
    }

    "###);
}

#[test]
fn test_adjacently_tagged_union_output() {
    assert_snapshot!(SwiftType::Union(&Union {
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
        FieldType::List(Box::new(FieldType::Primitive(Primitive::String))).swift_type(),
        @"[String]"
    );
}

#[test]
fn test_map_types() {
    assert_snapshot!(
        FieldType::Map{
            key: Box::new(FieldType::Primitive(Primitive::String)),
            value: Box::new(FieldType::Primitive(Primitive::Int))
        }.swift_type(),
        @"[String: Int64]"
    );
}

#[test]
fn test_option_types() {
    assert_snapshot!(
        FieldType::Optional(Box::new(FieldType::Primitive(Primitive::String))).swift_type(),
        @"String?"
    );
}
