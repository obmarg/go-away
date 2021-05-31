#![allow(dead_code)]

use insta::assert_snapshot;

use go_away::{TypeMetadata, TypeRegistry};

#[derive(TypeMetadata)]
struct MyData {
    field_one: String,
    nested: Nested,
}

#[test]
fn test_struct_output() {
    let mut registry = TypeRegistry::new();
    MyData::metadata(&mut registry);

    assert_snapshot!(go_away::registry_to_output(registry));
}

#[derive(TypeMetadata)]
#[serde(tag = "type", content = "data")]
enum NewTypeEnum {
    OptionOne(One),
    OptionTwo(Two),
}

#[derive(TypeMetadata)]
struct Nested {
    #[serde(rename = "some_other_name")]
    a_string: String,
    an_int: i64,
    fulfilment_type: FulfilmentType,
}

#[derive(TypeMetadata)]
enum FulfilmentType {
    Delivery,
    Collection,
}

#[derive(TypeMetadata)]
struct One {
    x: f32,
}

#[derive(TypeMetadata)]
struct Two {
    y: bool,
}

#[test]
fn test_newtype_enum() {
    let mut registry = TypeRegistry::new();
    NewTypeEnum::metadata(&mut registry);

    assert_snapshot!(go_away::registry_to_output(registry));
}

#[derive(TypeMetadata)]
#[serde(tag = "type", content = "data")]
enum StructEnum {
    OptionOne { x: String, y: i32 },
    OptionTwo { foo: String, bar: Nested },
}

#[test]
fn test_struct_enum() {
    let mut registry = TypeRegistry::new();
    StructEnum::metadata(&mut registry);

    assert_snapshot!(go_away::registry_to_output(registry));
}

#[derive(TypeMetadata)]
#[serde(tag = "type")]
enum InternallyTaggedTupleEnum {
    One(One),
    Two(Two),
}

#[test]
fn test_internally_tagged_tuple_enum() {
    let mut registry = TypeRegistry::new();
    InternallyTaggedTupleEnum::metadata(&mut registry);

    assert_snapshot!(go_away::registry_to_output(registry));
}
