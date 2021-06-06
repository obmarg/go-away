#![allow(clippy::unit_arg, clippy::blacklisted_name)]

use std::{
    fmt::Debug,
    fs::File,
    io::Write,
    process::{Command, Stdio},
};

use indoc::writedoc;
use serde::{Deserialize, Serialize};

use go_away::{registry_to_output, TypeMetadata, TypeRegistry};

#[derive(TypeMetadata, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
enum InternallyTaggedTupleEnum {
    One(One),
    Two(Two),
}

#[derive(TypeMetadata, Debug, Serialize, Deserialize, PartialEq)]
struct One {
    x: f32,
}

#[derive(TypeMetadata, Debug, Serialize, Deserialize, PartialEq)]
struct Two {
    y: bool,
}

#[test]
fn test_internally_tagged_tuple_enum() {
    run_test(
        "internally_tagged_tuple_enum",
        "InternallyTaggedTupleEnum",
        &[
            InternallyTaggedTupleEnum::One(One { x: 1.0 }),
            InternallyTaggedTupleEnum::Two(Two { y: true }),
        ],
    );
}

#[derive(TypeMetadata, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
enum StructEnum {
    OptionOne { x: String, y: i32 },
    OptionTwo { foo: String, bar: Nested },
}

#[derive(TypeMetadata, Debug, Serialize, Deserialize, PartialEq)]
struct Nested {
    #[serde(rename = "some_other_name")]
    a_string: String,
    an_int: i64,
    fulfilment_type: FulfilmentType,
}

#[derive(TypeMetadata, Debug, Serialize, Deserialize, PartialEq)]
enum FulfilmentType {
    Delivery,
    Collection,
}

#[test]
fn test_struct_enum() {
    run_test(
        "struct_enum",
        "StructEnum",
        &[
            StructEnum::OptionOne {
                x: "hello".into(),
                y: 100,
            },
            StructEnum::OptionTwo {
                foo: "bar".into(),
                bar: Nested {
                    a_string: "hello".into(),
                    an_int: 65536,
                    fulfilment_type: FulfilmentType::Collection,
                },
            },
            StructEnum::OptionTwo {
                foo: "bar".into(),
                bar: Nested {
                    a_string: "hello".into(),
                    an_int: 65536,
                    fulfilment_type: FulfilmentType::Delivery,
                },
            },
        ],
    );
}

fn run_test<T>(test_name: &str, type_name: &str, test_data: &[T])
where
    T: TypeMetadata + Serialize + serde::de::DeserializeOwned + PartialEq + Debug,
{
    let mut registry = TypeRegistry::new();
    T::metadata(&mut registry);
    let go_code = registry_to_output(registry);
    let path = format!("../go-temp/{}.go", test_name);
    let mut file = File::create(&path).unwrap();

    writedoc!(
        &mut file,
        r#"
		package main

		import (
			"encoding/json"
			"errors"
			"os"
			"log"
			"io"
			"fmt"
		)

		{}

		func main() {{
			var input {}
			dec := json.NewDecoder(os.Stdin)
			for {{
				err := dec.Decode(&input)
				if err == io.EOF {{
					return
				}}
				if err != nil {{
					log.Fatalf("Decoding error: %v", err)
				}}

				output, err := json.Marshal(input)
				if err != nil {{
					log.Fatalf("Encoding error: %v", err)
				}}
				fmt.Println(string(output))
			}}
		}}
		"#,
        go_code,
        type_name
    )
    .unwrap();

    let process = Command::new("go")
        .args(&["run", &path])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = process.stdin.as_ref().unwrap();
    for item in test_data {
        serde_json::to_writer(&mut stdin, item).unwrap();
    }
    stdin.flush().unwrap();

    let output = process.wait_with_output().unwrap();
    assert!(output.status.success());

    let output = String::from_utf8(output.stdout).unwrap();
    let lines = output.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), test_data.len());

    for (line, expected) in lines.into_iter().zip(test_data) {
        match serde_json::from_str::<T>(line) {
            Ok(result) => {
                assert_eq!(result, *expected);
            }
            Err(e) => {
                panic!(
                    "\n\nFailed to decode.\n\nExpected: {}\nActual:   {}\nError:    {}\n\n",
                    serde_json::to_value(expected).unwrap(),
                    line,
                    e
                );
            }
        }
    }
}
