use super::{TypeMetadata, TypeRegistry};

/// A trait for types that can be registered as aliases.
///
/// Users shouldn't usually need to impl this - a blanket impl is provided
/// for all types that impl `TypeMetadata`.
pub trait TypeAlias {
    /// Registers this type as a type alias.
    ///
    /// Note that this should not be used on types which have `TypeMetadata`
    /// derived on them - it's only really meant for use on actual rust type
    /// aliases.
    fn register_as_alias(name: &str, registry: &mut TypeRegistry);
}

impl<T> TypeAlias for T
where
    T: TypeMetadata + 'static,
{
    fn register_as_alias(name: &str, registry: &mut TypeRegistry) {
        let inner = Self::metadata(registry);
        registry.register_alias(
            crate::TypeId::for_type::<Self>(),
            crate::types::Alias {
                name: name.to_string(),
                inner,
            },
        );
    }
}
