//! # Go Away
//!
//! Go Away is a small library for generating go types & marshalling code from Rust type
//! definitions.  It's intended for use when you have existing rust code that is using serde for
//! JSON serialization and you want to allow go services or clients to interact with that code.
//!
//! It may be expanded to other languages at some point but it's mostly been built to service a
//! very specific need and might never evolve past that.
//!
//! Use is fairly simple:
//!
//! ```rust
//! use go_away::{TypeMetadata, TypeRegistry};
//!
//! // First, derive TypeMetadata on some types:
//!
//! #[derive(TypeMetadata)]
//! struct MyType {
//!     my_field: String
//! }
//!
//! // Then you can register this type inside a `TypeRegistry`
//!
//! let mut registry = TypeRegistry::new();
//! MyType::metadata(&mut registry);
//!
//! // And get some go code to write out to a file:
//! let go_code = go_away::registry_to_output::<go_away::GoType>(&registry);
//! ```
//!
//! Note that the output go code does not contain any package definitions or required imports.
//! It's expected that any code that calls `go-away` will add this for itself.
//!

#![warn(missing_docs)]

mod alias;
mod metadata;
mod output;
mod registry;
mod type_id;

pub mod types;

pub use alias::TypeAlias;
pub use metadata::TypeMetadata;
pub use output::{GoType, TypeScriptType};
pub use registry::TypeRegistry;
pub use type_id::TypeId;

pub use go_away_derive::TypeMetadata;

use registry::Type;
use std::fmt::Display;

/// Generates code for all the types in the TypeRegistry
///
/// You should provide `go_away::GoType` or `go_away::TypeScriptType` as a generic
/// parameter with turbofish syntax to decide which format to output.
///
/// Note that this is a WIP API and is likely to be ditched/changed in future releases.
pub fn registry_to_output<'a, Format>(registry: &'a TypeRegistry) -> String
where
    Format: From<&'a Type> + Display,
{
    use std::fmt::Write;

    let mut output = String::new();
    for id in registry.structs.iter().rev() {
        let ty = registry.types.get(id).unwrap();
        write!(&mut output, "{}", Format::from(ty)).unwrap();
    }
    for id in registry.unions.iter().rev() {
        let ty = registry.types.get(id).unwrap();
        write!(&mut output, "{}", Format::from(ty)).unwrap();
    }
    for id in registry.newtypes.iter().rev() {
        let ty = registry.types.get(id).unwrap();
        write!(&mut output, "{}", Format::from(ty)).unwrap();
    }
    for id in registry.enums.iter().rev() {
        let ty = registry.types.get(id).unwrap();
        write!(&mut output, "{}", Format::from(ty)).unwrap();
    }
    for id in registry.aliases.iter().rev() {
        let ty = registry.types.get(id).unwrap();
        write!(&mut output, "{}", Format::from(ty)).unwrap();
    }

    output
}

impl<'a> From<&'a registry::Type> for output::go::GoType<'a> {
    fn from(ty: &'a registry::Type) -> Self {
        match ty {
            registry::Type::Struct(inner) => output::go::GoType::Struct(inner),
            registry::Type::Enum(inner) => output::go::GoType::Enum(inner),
            registry::Type::Union(inner) => output::go::GoType::Union(inner),
            registry::Type::NewType(inner) => output::go::GoType::NewType(inner),
            registry::Type::Alias(inner) => output::go::GoType::Alias(inner),
        }
    }
}

impl<'a> From<&'a registry::Type> for output::typescript::TypeScriptType<'a> {
    fn from(ty: &'a registry::Type) -> Self {
        match ty {
            registry::Type::Struct(inner) => output::typescript::TypeScriptType::Struct(inner),
            registry::Type::Enum(inner) => output::typescript::TypeScriptType::Enum(inner),
            registry::Type::Union(inner) => output::typescript::TypeScriptType::Union(inner),
            registry::Type::NewType(inner) => output::typescript::TypeScriptType::NewType(inner),
            registry::Type::Alias(inner) => output::typescript::TypeScriptType::Alias(inner),
        }
    }
}
