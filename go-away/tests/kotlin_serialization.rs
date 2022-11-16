#![allow(clippy::unit_arg, clippy::disallowed_names)]

use std::{
    fmt::Debug,
    fs::File,
    io::Write,
    process::{Command, Stdio},
};

use indoc::writedoc;
use serde::{Deserialize, Serialize};

use go_away::{registry_to_output, TypeMetadata, TypeRegistry};

/* TODO
#[cfg(feature = "chrono")]
use chrono::{DateTime, NaiveDateTime, Utc};

*/

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

/*

TODO: Fix this

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
 */

#[derive(TypeMetadata, Debug, Serialize, Deserialize, PartialEq)]
enum ExternallyTaggedTupleEnum {
    One(One),
    Two(Two),
}

#[test]
fn test_externally_tagged_tuple_enum() {
    run_test(
        "externally_tagged_tuple_enum",
        "ExternallyTaggedTupleEnum",
        &[
            ExternallyTaggedTupleEnum::One(One { x: 1.0 }),
            ExternallyTaggedTupleEnum::Two(Two { y: true }),
        ],
    );
}

#[derive(TypeMetadata, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
enum AdjacentlyTaggedTupleEnum {
    One(One),
    Two(Two),
}

#[test]
fn test_adjacently_tagged_tuple_enum() {
    run_test(
        "adjacently_tagged_tuple_enum",
        "AdjacentlyTaggedTupleEnum",
        &[
            AdjacentlyTaggedTupleEnum::One(One { x: 1.0 }),
            AdjacentlyTaggedTupleEnum::Two(Two { y: true }),
        ],
    );
}

#[derive(TypeMetadata, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
enum StructEnum {
    OptionOne { x: String, y: i32 },
    OptionTwo { foo: String, bar: Nested },
}

/*

TODO:

#[cfg(feature = "chrono")]
#[derive(TypeMetadata, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
enum DateTimeEnum {
    One { a: DateTime<chrono::Utc> },
}
*/

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

/* TODO:
#[cfg(feature = "chrono")]
#[test]
fn test_datetime() {
    run_test(
        "datetime",
        "DateTimeEnum",
        &[DateTimeEnum::One {
            a: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc),
        }],
    );
}
*/

// TODO: Can ditch this one once we've got the rest working
#[test]
fn test_struct() {
    run_test(
        "struct",
        "Nested",
        &[Nested {
            a_string: "hello".into(),
            an_int: 65536,
            fulfilment_type: FulfilmentType::Collection,
        }],
    );
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
    let kotlin_code = registry_to_output::<go_away::KotlinType>(&registry);

    let path = tempfile::tempdir().unwrap();
    fs_extra::dir::copy("tests/gradle-template", &path, &Default::default()).unwrap();
    let path = path.path().join("gradle-template");
    let file_path = path.join("app/src/main/kotlin/go/away/test/App.kt");
    let mut file = File::create(&file_path).unwrap();

    writedoc!(
        &mut file,
        r#"
        package go.away.test

        import kotlinx.serialization.Serializable
        import kotlinx.serialization.SerialName
        import kotlinx.serialization.decodeFromString
        import kotlinx.serialization.encodeToString
        import kotlinx.serialization.json.Json

        {kotlin_code}

        fun main() {{
            var line = readLine()
            while (line != null) {{
                val output = Json.encodeToString(Json.decodeFromString<{type_name}>(line))
                println(output)
                line = readLine()
            }}
        }}
		"#,
    )
    .unwrap();

    let compile_status = Command::new("./gradlew")
        .args(["build"])
        .current_dir(&path)
        .status()
        .unwrap();

    if !compile_status.success() {
        println!("Error when compiling {test_name}");
        println!(
            "Contents of kotlin file: {}",
            std::fs::read_to_string(file_path).unwrap()
        );
        panic!("compilation failed");
    }

    let dist_path = path.join("app/build/distributions/");
    let unzip_status = Command::new("unzip")
        .args(["app.zip"])
        .current_dir(&dist_path)
        .status()
        .unwrap();
    if !unzip_status.success() {
        panic!("Couldn't unzip distribution");
    }

    let script_path = dist_path.join("app/bin");
    let process = Command::new("./app")
        .current_dir(script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = process.stdin.as_ref().unwrap();
    for item in test_data {
        serde_json::to_writer(&mut stdin, item).unwrap();
        writeln!(&mut stdin).unwrap();
    }
    stdin.flush().unwrap();

    let output = process.wait_with_output().unwrap();

    // Uncomment if you wish to output the result of the process
    // println!("{}", String::from_utf8_lossy(&output.stdout));
    // println!("{}", String::from_utf8_lossy(&output.stderr));
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
