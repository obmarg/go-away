pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

pub struct Field {
    pub name: String,
    pub serialized_name: String,
    pub ty: super::FieldType,
}

pub struct NewType {
    pub name: String,
    pub inner: super::FieldType,
}

pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

pub struct EnumVariant {
    pub name: String,
    pub serialized_name: String,
}

pub struct Union {
    pub name: String,
    pub representation: UnionRepresentation,
    pub variants: Vec<UnionVariant>,
}

pub struct UnionVariant {
    pub name: Option<String>,
    pub ty: super::FieldType,
    pub serialized_name: String,
}

pub enum UnionRepresentation {
    AdjacentlyTagged { tag: String, content: String },
    InternallyTagged,
    ExternallyTagged,
    Untagged,
}
