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
