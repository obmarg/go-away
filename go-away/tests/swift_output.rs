#![allow(dead_code)]

use insta::assert_snapshot;

use go_away::{TypeAlias, TypeMetadata, TypeRegistry};

#[derive(TypeMetadata)]
struct MyData {
    field_one: String,
    nested: Nested,
}

#[test]
fn test_struct_output() {
    let mut registry = TypeRegistry::new();
    MyData::metadata(&mut registry);

    assert_snapshot!(go_away::registry_to_output::<go_away::SwiftType>(&registry));
}

#[derive(TypeMetadata)]
#[serde(tag = "type", content = "data")]
enum AdjacentlyTaggedTupleEnums {
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
fn test_adjacently_tagged_tuple_enums() {
    let mut registry = TypeRegistry::new();
    AdjacentlyTaggedTupleEnums::metadata(&mut registry);

    assert_snapshot!(go_away::registry_to_output::<go_away::SwiftType>(&registry));
}

/*
// TODO: Add StructEnum support...

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

    assert_snapshot!(go_away::registry_to_output::<go_away::SwiftType>(&registry));
}
*/

/*

// TODO: Finish this.

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

    assert_snapshot!(go_away::registry_to_output::<go_away::SwiftType>(&registry));
}
*/

#[derive(TypeMetadata)]
enum ExternallyTaggedTupleEnum {
    One(One),
    Two(Two),
}

#[test]
fn test_externally_tagged_tuple_enum() {
    let mut registry = TypeRegistry::new();
    ExternallyTaggedTupleEnum::metadata(&mut registry);

    assert_snapshot!(go_away::registry_to_output::<go_away::SwiftType>(&registry));
}

#[derive(TypeMetadata)]
struct TypeWithLifetimes<'a, 'b> {
    data: &'a str,
    other: &'b str,
}

#[test]
fn lifetimes_and_strs() {
    let mut registry = TypeRegistry::new();
    TypeWithLifetimes::metadata(&mut registry);

    assert_snapshot!(go_away::registry_to_output::<go_away::SwiftType>(&registry));
}

#[test]
fn type_deduplication() {
    let mut registry = TypeRegistry::new();

    // These both contain `Nested` so there should be one `Nested` type in the output
    //StructEnum::metadata(&mut registry);
    Nested::metadata(&mut registry);
    MyData::metadata(&mut registry);

    assert_snapshot!(go_away::registry_to_output::<go_away::SwiftType>(&registry));
}

#[test]
fn type_aliases() {
    type MyType = std::collections::HashMap<String, i64>;

    let mut registry = TypeRegistry::new();

    MyType::register_alias("MyType", &mut registry);

    assert_snapshot!(go_away::registry_to_output::<go_away::SwiftType>(&registry), @r###"
    typealias MyType = [String: Int64]
    "###);
}
