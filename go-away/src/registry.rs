pub use std::collections::HashMap;

pub use super::{
    types::{Alias, Enum, NewType, Struct, TypeRef, Union},
    TypeId,
};

/// A registry of type details.
///
/// Can be populated by passing in to `TypeMetadata::metadata` and then used
/// to output types in other languages.
#[derive(Default)]
pub struct TypeRegistry {
    /// The master list of types
    pub(super) types: HashMap<TypeId, Type>,

    /// An ordered list of struct IDs, used to output in order.
    pub(super) structs: Vec<TypeId>,
    /// An ordered list of enum IDs, used to output in order.
    pub(super) enums: Vec<TypeId>,
    /// An ordered list of union IDs, used to output in order.
    pub(super) unions: Vec<TypeId>,
    /// An ordered list of newtype IDs, used to output in order.
    pub(super) newtypes: Vec<TypeId>,
    /// An ordered list of alias IDs
    pub(super) aliases: Vec<TypeId>,
}

impl TypeRegistry {
    /// Construct a new TypeRegistry
    pub fn new() -> Self {
        TypeRegistry::default()
    }

    /// Registers an `Alias` in the `TypeRegistry`
    ///
    /// Users should use `crate::TypeAlias::register_alias` rather than calling this directly.
    pub fn register_alias(&mut self, id: TypeId, details: Alias) -> TypeRef {
        if !self.types.contains_key(&id) {
            self.aliases.push(id.clone());
        }
        self.register_type(id, Type::Alias(details))
    }

    /// Register a `Struct`
    ///
    /// Users should use `crate::TypeMetadata::metadata` rather than calling this directly.
    pub fn register_struct(&mut self, id: TypeId, details: Struct) -> TypeRef {
        if !self.types.contains_key(&id) {
            self.structs.push(id.clone());
        }
        self.register_type(id, Type::Struct(details))
    }

    /// Register a `NewType`
    ///
    /// Users should use `crate::TypeMetadata::metadata` rather than calling this directly.
    pub fn register_newtype(&mut self, id: TypeId, details: NewType) -> TypeRef {
        if !self.types.contains_key(&id) {
            self.newtypes.push(id.clone());
        }
        self.register_type(id, Type::NewType(details))
    }

    /// Register an `Enum`
    ///
    /// Users should use `crate::TypeMetadata::metadata` rather than calling this directly.
    pub fn register_enum(&mut self, id: TypeId, details: Enum) -> TypeRef {
        if !self.types.contains_key(&id) {
            self.enums.push(id.clone());
        }
        self.register_type(id, Type::Enum(details))
    }

    /// Register a `Uninon`
    ///
    /// Users should use `crate::TypeMetadata::metadata` rather than calling this directly.
    pub fn register_union(&mut self, id: TypeId, details: Union) -> TypeRef {
        if !self.types.contains_key(&id) {
            self.unions.push(id.clone());
        }
        self.register_type(id, Type::Union(details))
    }

    fn register_type(&mut self, id: TypeId, ty: Type) -> TypeRef {
        if self.types.contains_key(&id) {
            match self.types.get(&id) {
                Some(existing) if ty.same_kind(&existing) => return existing.type_ref(),
                other => panic!("Type register mismatch: {:?} vs {:?}", ty, other),
            }
        }

        let type_ref = ty.type_ref();
        self.types.insert(id, ty);

        type_ref
    }
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(super) enum Type {
    Struct(Struct),
    Enum(Enum),
    Union(Union),
    NewType(NewType),
    Alias(Alias),
}

impl Type {
    fn type_ref(&self) -> TypeRef {
        match self {
            Type::Struct(st) => TypeRef {
                name: st.name.clone(),
            },
            Type::Enum(en) => TypeRef {
                name: en.name.clone(),
            },
            Type::Union(un) => TypeRef {
                name: un.name.clone(),
            },
            Type::NewType(nt) => TypeRef {
                name: nt.name.clone(),
            },
            Type::Alias(nt) => TypeRef {
                name: nt.name.clone(),
            },
        }
    }

    #[allow(clippy::match_like_matches_macro)]
    fn same_kind(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Struct(_), Type::Struct(_)) => true,
            (Type::Enum(_), Type::Enum(_)) => true,
            (Type::Union(_), Type::Union(_)) => true,
            (Type::NewType(_), Type::NewType(_)) => true,
            _ => false,
        }
    }
}
