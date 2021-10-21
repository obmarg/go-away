//! Defines the type model for go-away - a set of structs that describe
//! types and can be used to generate copies of them in other languages.

/// A struct.
///
/// This will be serialized as a JSON object.
#[derive(Debug)]
pub struct Struct {
    /// The name of the struct in Rust
    pub name: String,

    /// The structs fields.
    pub fields: Vec<Field>,
}

/// A field within a struct
#[derive(Debug)]
pub struct Field {
    /// The name of the field in rust.  If the field is un-named this will
    /// be a number.
    pub name: String,
    // TODO: Need both serialize & deserialize apparently
    /// The name of the field when serialized.
    pub serialized_name: String,
    /// The type of the field
    pub ty: FieldType,
}

/// A newtype struct (e.g. `struct SomeId(String)`)
///
/// These are usually represented as their inner type when serialized.
#[derive(Debug)]
pub struct NewType {
    /// The name of the struct in rust.
    pub name: String,

    /// The type contained within the newtype.
    pub inner: FieldType,
}

/// A type alias (e.g. `type SomeType = HashMap<String, String>;`)
///
/// These are usually represented as their inner type when serialized.
#[derive(Debug)]
pub struct Alias {
    /// The name of the type alias in rust.
    pub name: String,

    /// The type that is being aliased.
    pub inner: FieldType,
}

/// An enum - note that in go-away these do not contain data.
///
/// A Rust enum that's variants contain values will go to a `UnionType`
#[derive(Debug)]
pub struct Enum {
    /// The name of the enum
    pub name: String,
    /// The enums variants
    pub variants: Vec<EnumVariant>,
}

/// An enum variant - note that these are just names and are serialized
/// as strings.
#[derive(Debug)]
pub struct EnumVariant {
    /// The name of the variant in code.
    pub name: String,
    /// The name of the variant when serialized.
    pub serialized_name: String,
}

/// A union type - any rust enum that's variants contain data.
///
/// These will be serialzied differently depending on the UnionRepresentation.
#[derive(Debug)]
pub struct Union {
    /// The name of the union
    pub name: String,
    /// The representation of the union to use on the wire.
    pub representation: UnionRepresentation,
    /// The unions variants.
    pub variants: Vec<UnionVariant>,
}

#[derive(Debug, PartialEq, Eq)]
/// A variant of a union type
pub struct UnionVariant {
    /// The name of the variant if any
    pub name: Option<String>,
    /// The type inside the variant
    pub ty: FieldType,
    /// The name the variant will be serialized to
    pub serialized_name: String,
}

/// The serialized representation of the union type
///
/// See https://serde.rs/enum-representations.html for details
#[derive(Debug)]
pub enum UnionRepresentation {
    /// An adjacently tagged representation
    AdjacentlyTagged {
        /// The name of the tag field
        tag: String,
        /// The name of the content field
        content: String,
    },
    /// An internally tagged representation
    InternallyTagged {
        /// The name of the tag field
        tag: String,
    },
    /// An externally tagged representation
    ExternallyTagged,
    /// An untagged representation
    Untagged,
}

/// The type of a field.
#[derive(Debug, PartialEq, Eq)]
pub enum FieldType {
    /// A `Option<T>` field
    Optional(Box<FieldType>),
    /// a `Vec<T>` field
    List(Box<FieldType>),
    /// a `HashMap<K, V>` field
    Map {
        /// The type of the HashMaps keys
        key: Box<FieldType>,
        /// The type of the HashMaps values
        value: Box<FieldType>,
    },
    /// A field with a named type
    Named(TypeRef),
    /// A field with a primitive type
    Primitive(Primitive),
}

/// The primitive types
#[derive(Debug, PartialEq, Eq)]
pub enum Primitive {
    /// Strings
    String,
    /// Floating point numbers
    Float,
    /// Integers
    Int,
    /// Booleans
    Bool,
    /// Time
    Time,
}

/// A reference to a given named type
#[derive(Debug, PartialEq, Eq)]
pub struct TypeRef {
    pub(crate) name: String,
    // TODO: id: std::any::TypeId,
}

impl TypeRef {
    /// Gets the name of the referenced type
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}
