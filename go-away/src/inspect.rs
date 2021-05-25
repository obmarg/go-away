use super::{FieldType, TypeRegistry};

pub trait Inspect {
    fn info(registry: &mut TypeRegistry) -> FieldType;
}

impl<T> Inspect for Vec<T>
where
    T: Inspect,
{
    fn info(registry: &mut TypeRegistry) -> FieldType {
        FieldType::List(Box::new(T::info(registry)))
    }
}

impl<T> Inspect for Option<T>
where
    T: Inspect,
{
    fn info(registry: &mut TypeRegistry) -> FieldType {
        FieldType::Optional(Box::new(T::info(registry)))
    }
}

impl<K, V> Inspect for std::collections::HashMap<K, V>
where
    K: Inspect,
    V: Inspect,
{
    fn info(registry: &mut TypeRegistry) -> FieldType {
        let key = Box::new(K::info(registry));
        let value = Box::new(V::info(registry));

        FieldType::Map { key, value }
    }
}
