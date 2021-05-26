mod metadata;
mod output;
mod registry;

pub mod types;

pub use metadata::TypeMetadata;
pub use registry::TypeRegistry;

pub use go_away_derive::TypeMetadata;

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

#[derive(PartialEq, Eq)]
pub enum FieldType {
    Optional(Box<FieldType>),
    List(Box<FieldType>),
    Map {
        key: Box<FieldType>,
        value: Box<FieldType>,
    },
    Named(TypeRef),
    Primitive(Primitive),
}

#[derive(PartialEq, Eq)]
pub enum Primitive {
    String,
    Float,
    Int,
    Bool,
}

#[derive(PartialEq, Eq)]
pub struct TypeRef {
    name: String,
}

impl TypeRef {
    fn name(&self) -> &str {
        &self.name
    }
}
