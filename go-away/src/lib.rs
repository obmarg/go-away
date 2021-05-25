mod inspect;
mod output;
mod types;

pub use inspect::Inspect;

// TODO: maps and lists?
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

pub enum Primitive {
    String,
    Float,
    Int,
    Bool,
}

pub struct TypeRef {}

impl TypeRef {
    fn name(&self) -> String {
        todo!()
    }
}

pub struct TypeRegistry {}

// TODO: these methods maybe need to take/return some sort of UUID that identifies the type...
impl TypeRegistry {
    // Ok, so structs come in these varieties
    // - normal structs (easy)
    // - enum structs (maybe I skip these for now?)
    pub fn register_struct(&mut self) -> TypeRef {
        todo!()
    }

    // Deals w/ newtypes
    pub fn register_newtype(&mut self) -> TypeRef {
        todo!()
    }

    // This deals w/ the simple enum case
    pub fn register_enum(&mut self) -> TypeRef {
        todo!()
    }

    // Ok, so if this deals w/ union types (i.e. enums w/ data)
    pub fn register_union(&mut self) -> TypeRef {
        todo!()
    }
}
