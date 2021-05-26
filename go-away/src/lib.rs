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
//! use go_away::TypeMetadata;
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
//! MyType::metadata(&mut registy);
//!
//! // And get some go code to write out to a file:
//! let go_code = go_away::registry_to_output(registry);
//! ```
//!
//! Note that the output go code does not contain any package definitions or required imports.
//! It's expected that any code that calls `go-away` will add this for itself.
//!

#![warn(missing_docs)]

mod metadata;
mod output;
mod registry;

pub mod types;

pub use metadata::TypeMetadata;
pub use registry::TypeRegistry;

pub use go_away_derive::TypeMetadata;

/// Generates go ocde for all the types in the TypeRegistry
///
/// Note that this is a WIP API and is likely to be ditched/changed in future releases.
pub fn registry_to_output(registry: TypeRegistry) -> String {
    use std::fmt::Write;

    let mut output = String::new();
    for st in registry.structs {
        write!(&mut output, "{}", output::GoType::Struct(st)).unwrap();
    }
    for en in registry.enums {
        write!(&mut output, "{}", output::GoType::Enum(en)).unwrap();
    }
    for un in registry.unions {
        write!(&mut output, "{}", output::GoType::Union(un)).unwrap();
    }
    for nt in registry.newtypes {
        write!(&mut output, "{}", output::GoType::NewType(nt)).unwrap();
    }

    output
}
