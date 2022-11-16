use super::data_classes::{DataClass, NewTypeClass};
use crate::{
    output::prelude::*,
    types::{self, FieldType},
};

pub enum KotlinStruct<'a> {
    Normal(DataClass<'a>),
    NewType(NewTypeClass<'a>),
}

impl<'a> KotlinStruct<'a> {
    pub fn new(name: &'a str) -> KotlinStruct {
        KotlinStruct::Normal(DataClass::new(name))
    }

    pub fn newtype(name: &'a str, ty: &'a FieldType) -> Self {
        KotlinStruct::NewType(
            NewTypeClass::new(name, ty.kotlin_type(), ty.serializer())
                .with_default_string(ty.default_str()),
        )
    }

    pub fn with_fields(mut self, new_fields: &'a [types::Field]) -> Self {
        let KotlinStruct::Normal(data_class) = &mut self else {
            panic!("Called with_fields on a newtype");
        };
        data_class.add_fields(new_fields.iter().map(Into::into));
        self
    }
}

impl fmt::Display for KotlinStruct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KotlinStruct::Normal(inner) => write!(f, "{inner}"),
            KotlinStruct::NewType(inner) => write!(f, "{inner}"),
        }
    }
}
