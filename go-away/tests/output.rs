#![allow(dead_code)]

use insta::assert_snapshot;

use go_away::{TypeMetadata, TypeRegistry};

#[derive(TypeMetadata)]
struct MyData {
    field_one: String,
    nested: Nested,
}

#[derive(TypeMetadata)]
struct Nested {
    #[serde(rename = "some_other_name")]
    a_string: String,
    an_int: i64,
}

#[test]
fn test_struct_output() {
    let mut registry = TypeRegistry::new();
    MyData::metadata(&mut registry);

    assert_snapshot!(go_away::registry_to_output(registry));
}
