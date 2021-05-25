pub struct Struct {
    name: String,
    fields: Vec<Field>,
}

pub struct Field {
    name: String,
    serialized_name: String,
    ty: super::FieldType,
}

pub struct NewType {
    name: String,
    inner: super::FieldType,
}

pub struct Enum {
    name: String,
    variants: Vec<EnumVariant>,
}

pub struct EnumVariant {
    name: String,
    serialized_name: String,
}

pub struct Union {
    name: String,
    representation: UnionRepresentation,
    variants: Vec<UnionVariant>,
}

pub struct UnionVariant {
    name: String,
    serialized_name: String,
    inner_type: super::TypeRef,
}

pub enum UnionRepresentation {
    AdjacentlyTagged,
    InternallyTagged,
    ExternallyTagged,
}
