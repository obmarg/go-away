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
    public struct MyStruct: Hashable {
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
        public var userId: String

        public init(
            userId: String,
        ) {
            self.userId = userId
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
