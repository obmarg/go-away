pub use super::types::{Enum, NewType, Struct, TypeRef, Union};

/// A registry of type details.
///
/// Can be populated by passing in to `TypeMetadata::metadata` and then used
/// to output types in other languages.
#[derive(Default)]
pub struct TypeRegistry {
    pub(super) structs: Vec<Struct>,
    pub(super) enums: Vec<Enum>,
    pub(super) unions: Vec<Union>,
    pub(super) newtypes: Vec<NewType>,
}

// TODO: these methods maybe need to take/return some sort of UUID that identifies the type...
impl TypeRegistry {
    /// Construct a new TypeRegistry
    pub fn new() -> Self {
        TypeRegistry::default()
    }

    /// Register a `Struct`
    pub fn register_struct(&mut self, details: Struct) -> TypeRef {
        let name = details.name.clone();
        self.structs.push(details);
        TypeRef { name }
    }

    /// Register a `NewType`
    pub fn register_newtype(&mut self, details: NewType) -> TypeRef {
        let name = details.name.clone();
        self.newtypes.push(details);
        TypeRef { name }
    }

    /// Register an `Enum`
    pub fn register_enum(&mut self, details: Enum) -> TypeRef {
        let name = details.name.clone();
        self.enums.push(details);
        TypeRef { name }
    }

    /// Register a `Uninon`
    pub fn register_union(&mut self, details: Union) -> TypeRef {
        let name = details.name.clone();
        self.unions.push(details);
        TypeRef { name }
    }
}
