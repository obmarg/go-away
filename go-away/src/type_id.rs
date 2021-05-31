pub use std::borrow::Cow;

/// A type identifier - used to deduplicate types in the output
#[derive(Hash, Debug, PartialEq, Eq, Clone)]
pub struct TypeId(TypeIdInner);

impl TypeId {
    /// Construct a `TypeId` for a given rust type
    pub fn for_type<T: 'static + ?Sized>() -> Self {
        TypeId(TypeIdInner::Type(std::any::TypeId::of::<T>()))
    }

    /// Construct a `TypeId` for a variant of a rust enum.
    ///
    /// This needs specific support beacuse our output needs types that
    /// don't exist directly in rust.
    pub fn for_variant<T, S>(variant_name: S) -> Self
    where
        T: 'static + ?Sized,
        S: Into<Cow<'static, str>>,
    {
        TypeId(TypeIdInner::Variant {
            parent_enum: std::any::TypeId::of::<T>(),
            variant_name: variant_name.into(),
        })
    }
}

#[derive(Hash, Debug, PartialEq, Eq, Clone)]
pub(super) enum TypeIdInner {
    Type(std::any::TypeId),

    Variant {
        parent_enum: std::any::TypeId,
        variant_name: Cow<'static, str>,
    },
}
