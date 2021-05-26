pub use super::{
    types::{Enum, Struct, Union},
    TypeRef,
};

#[derive(Default)]
pub struct TypeRegistry {
    pub(super) structs: Vec<Struct>,
    pub(super) enums: Vec<Enum>,
    pub(super) unions: Vec<Union>,
}

// TODO: these methods maybe need to take/return some sort of UUID that identifies the type...
impl TypeRegistry {
    pub fn new() -> Self {
        TypeRegistry::default()
    }

    // Ok, so structs come in these varieties
    // - normal structs (easy)
    // - enum structs (maybe I skip these for now?)
    // TODO: IDs?
    pub fn register_struct(&mut self, details: Struct) -> TypeRef {
        let name = details.name.clone();
        self.structs.push(details);
        TypeRef { name }
    }

    // Deals w/ newtypes
    pub fn register_newtype(&mut self) -> TypeRef {
        todo!()
    }

    // This deals w/ the simple enum case
    pub fn register_enum(&mut self, details: Enum) -> TypeRef {
        let name = details.name.clone();
        self.enums.push(details);
        TypeRef { name }
    }

    // Ok, so if this deals w/ union types (i.e. enums w/ data)
    pub fn register_union(&mut self, details: Union) -> TypeRef {
        let name = details.name.clone();
        self.unions.push(details);
        TypeRef { name }
    }
}
