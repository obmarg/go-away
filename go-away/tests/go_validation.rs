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
enum SomeUnion {
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
fn validate_with_no_variants_fails() {
    assert!(run_test::<SomeUnion>(
        "validate_with_no_variants_fails",
        r#"SomeUnion {}"#
    ))
}

#[test]
fn validate_with_one_variant_passes() {
    assert!(!run_test::<SomeUnion>(
        "validate_with_one_variant_passes",
        r#"SomeUnion {One: &One {X: 1.0}}"#,
    ),);
}

#[test]
fn validate_with_two_variants_fails() {
    assert!(run_test::<SomeUnion>(
        "validate_with_two_variants_fails",
        r#"SomeUnion {One: &One {X: 1.0}, Two: &Two {Y: true}}"#,
    ),);
}

fn run_test<T>(test_name: &str, data: &str) -> bool
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
			"log"
			"fmt"
			"time"
		)

		{}

		func main() {{
		    // this is so that go compiler does not complain if time is not used
		    time.Now()
			output, err := json.Marshal({})
			if err == nil {{
				log.Fatalf("Expected an error, did not get one")
			}}
			if output != nil {{
				log.Fatalf("Expected no output, but got some")
			}}
		}}
		"#,
        go_code,
        data
    )
    .unwrap();

    let process = Command::new("go")
        .args(&["run", &path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Uncomment if you wish to output the result of the process
    // println!("{}", String::from_utf8_lossy(&output.stdout));
    // println!("{}", String::from_utf8_lossy(&output.stderr));
    let output = process.wait_with_output().unwrap();

    output.status.success()
}
