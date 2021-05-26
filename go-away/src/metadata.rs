use crate::Primitive;

use super::{FieldType, TypeRegistry};

// TODO: Sort of liking `TypeMetadata` for this?
// Like derive(TypeMetadata) seems good for some reason.
// Not sure.  Think about it.
// Reflection is also (sort of) a reasonable name
pub trait TypeMetadata {
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

impl TypeMetadata for bool {
    fn metadata(_: &mut TypeRegistry) -> FieldType {
        FieldType::Primitive(Primitive::Bool)
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
