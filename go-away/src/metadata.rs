use crate::{
    types::{FieldType, Primitive},
};

use super::TypeRegistry;

/// Exposes metadata about a type that can be used to generate other
/// versions of that type in other languages.
///
/// This is usually intended to be derived rather than manually implemented.
pub trait TypeMetadata {
    /// Populates `TypeRegistry` with this type and any of it's
    /// contained types and returns a `FieldType`
    fn metadata(registry: &mut TypeRegistry) -> FieldType;
}

impl<T> TypeMetadata for Vec<T>
where
    T: TypeMetadata,
{
    fn metadata(registry: &mut TypeRegistry) -> FieldType {
        FieldType::List(Box::new(T::metadata(registry)))
    }
}

impl<T> TypeMetadata for Option<T>
where
    T: TypeMetadata,
{
    fn metadata(registry: &mut TypeRegistry) -> FieldType {
        FieldType::Optional(Box::new(T::metadata(registry)))
    }
}

impl<K, V> TypeMetadata for std::collections::HashMap<K, V>
where
    K: TypeMetadata,
    V: TypeMetadata,
{
    fn metadata(registry: &mut TypeRegistry) -> FieldType {
        let key = Box::new(K::metadata(registry));
        let value = Box::new(V::metadata(registry));

        FieldType::Map { key, value }
    }
}

impl TypeMetadata for String {
    fn metadata(_: &mut TypeRegistry) -> FieldType {
        FieldType::Primitive(Primitive::String)
    }
}

impl TypeMetadata for str {
    fn metadata(_: &mut TypeRegistry) -> FieldType {
        FieldType::Primitive(Primitive::String)
    }
}

impl TypeMetadata for bool {
    fn metadata(_: &mut TypeRegistry) -> FieldType {
        FieldType::Primitive(Primitive::Bool)
    }
}

#[cfg(feature = "uuid")]
impl TypeMetadata for uuid::Uuid {
    fn metadata(_: &mut TypeRegistry) -> FieldType {
        FieldType::Primitive(Primitive::String)
    }
}

#[cfg(feature = "chrono")]
impl<Tz: chrono::offset::TimeZone> TypeMetadata for chrono::DateTime<Tz> {
    fn metadata(_: &mut TypeRegistry) -> FieldType {
        FieldType::Primitive(Primitive::Time)
    }
}

macro_rules! metadata_for_int {
    () => {};
    ($this:ty, $($tail:tt)*) => {
        metadata_for_int!($this);
        metadata_for_int!{$($tail)*}
    };
    ($int:ty) => {
        impl TypeMetadata for $int {
            fn metadata(_: &mut TypeRegistry) -> FieldType {
                FieldType::Primitive(Primitive::Int)
            }
        }
    };
}

metadata_for_int! {i8, i16, i32, i64, i128, u8, u16, u32, u64, u128}

macro_rules! metadata_for_float {
    () => {};
    ($this:ty, $($tail:tt)*) => {
        metadata_for_float!($this);
        metadata_for_float!{$($tail)*}
    };
    ($fl:ty) => {
        impl TypeMetadata for $fl {
            fn metadata(_: &mut TypeRegistry) -> FieldType {
                FieldType::Primitive(Primitive::Float)
            }
        }
    };
}

metadata_for_float! {f32, f64}
