use std::{collections::BTreeMap, path::Path};

use crate::parser::ComponentPartsParser;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Component {
    pub name: String,
    props: Props,
}
impl Component {
    pub fn new(name: impl Into<String>, props: Props) -> Self {
        Self {
            name: name.into(),
            props,
        }
    }
    pub fn props_str(&self) -> String {
        match &self.props {
            Props::Named(props) => props.name.clone(),
            Props::Expand(props) => props.to_str(),
        }
    }
    pub fn expand_str(&self) -> String {
        match &self.props {
            Props::Named(props) => props.expand_str(),
            Props::Expand(props) => props.to_str(),
        }
    }
    pub fn fill_sample(&self) -> String {
        match &self.props {
            Props::Named(props) => props.inner.fill_sample(),
            Props::Expand(props) => props.fill_sample(),
        }
    }
}
pub(crate) struct TSXContent(pub String);

impl TSXContent {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self(content))
    }
    pub fn to_component(&self) -> Option<Component> {
        let mut parser = ComponentPartsParser::new(self);
        parser.search_component()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Props {
    Named(NamedProps),
    Expand(ObjectType),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct NamedProps {
    pub name: String,
    inner: ObjectType,
}

impl NamedProps {
    pub fn new(name: impl Into<String>, inner: ObjectType) -> Self {
        Self {
            name: name.into(),
            inner,
        }
    }
    pub fn expand_str(&self) -> String {
        self.inner.to_str()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ObjectType {
    inner: BTreeMap<Key, Type>,
}

impl ObjectType {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }
    pub fn insert(&mut self, key: Key, ty: Type) {
        self.inner.insert(key, ty);
    }
    fn to_str(&self) -> String {
        let mut props = String::new();
        for (key, ty) in &self.inner {
            props.push_str(&format!("{}: {},", key.0, ty.to_str()));
        }
        format!("{{ {} }}", props)
    }
    fn fill_sample(&self) -> String {
        let mut props = String::new();
        for (key, ty) in &self.inner {
            props.push_str(&format!("{}: {},", key.0, ty.sample()));
        }
        format!("{{ {} }}", props)
    }
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd, Ord, Eq)]
pub(super) struct Key(pub String);

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Type {
    Number,
    String,
    Boolean,
    Array(Box<Type>),
    Object(ObjectType),
    Named(String),
}

/// <Type> ::= <PrimitiveType> | <ObjectType> | <AliasType> | <GenericType> | <UnionType> | <IntersectionType> | <LiteralType> | <ArrayType>
///

#[derive(Debug, Clone, PartialEq)]
pub(super) enum TSType {
    Primitive(PrimitiveType),
    Object(ObjectTypes),
    Alias(AliasType),
    Generic(GenericType),
    Union(UnionType),
    Intersection(IntersectionType),
    Literal(String),
    Array(Box<TSType>),
}
impl TSType {
    fn to_str(&self) -> String {
        match self {
            TSType::Primitive(p) => match p {
                PrimitiveType::String => "string".to_string(),
                PrimitiveType::Number => "number".to_string(),
                PrimitiveType::Boolean => "boolean".to_string(),
                PrimitiveType::Undefined => "undefined".to_string(),
                PrimitiveType::Null => "null".to_string(),
                PrimitiveType::Any => "any".to_string(),
                PrimitiveType::Void => "void".to_string(),
                PrimitiveType::Unknown => "unknown".to_string(),
                PrimitiveType::Never => "never".to_string(),
                PrimitiveType::Object => "object".to_string(),
            },
            TSType::Object(obj) => obj.to_str(),
            TSType::Alias(alias) => {
                format!("{} = {}", alias.name, alias.ty.as_ref().unwrap().to_str())
            }
            TSType::Generic(generic) => format!("{}<{}>", generic.name, generic.ty_list.to_str()),
            TSType::Union(union) => union.to_str(),
            TSType::Intersection(inter) => inter.to_str(),
            TSType::Literal(lit) => lit.clone(),
            TSType::Array(ty) => format!("{}[]", ty.to_str()),
        }
    }
    fn sample(&self) -> String {
        match self {
            TSType::Primitive(p) => match p {
                PrimitiveType::String => "\"\"".to_string(),
                PrimitiveType::Number => "0".to_string(),
                PrimitiveType::Boolean => "false".to_string(),
                PrimitiveType::Undefined => "undefined".to_string(),
                PrimitiveType::Null => "null".to_string(),
                PrimitiveType::Any => "null".to_string(),
                PrimitiveType::Void => "null".to_string(),
                PrimitiveType::Unknown => "null".to_string(),
                PrimitiveType::Never => "null".to_string(),
                PrimitiveType::Object => "null".to_string(),
            },
            TSType::Object(obj) => obj.inner.fill_sample(),
            TSType::Alias(alias) => alias.ty.as_ref().unwrap().sample(),
            TSType::Generic(generic) => generic.ty_list.inner[0].sample(),
            TSType::Union(union) => union.sample(),
            TSType::Intersection(inter) => inter.sample(),
            TSType::Literal(lit) => lit.clone(),
            TSType::Array(ty) => format!("[{}]", ty.sample()),
        }
    }
}

/// <PrimitiveType> ::= "string" | "number" | "boolean" | "undefined" | "null" | "any" | "void" | "unknown" | "never" | "object"
///
#[derive(Debug, Clone, PartialEq)]
pub(super) enum PrimitiveType {
    String,
    Number,
    Boolean,
    Undefined,
    Null,
    Any,
    Void,
    Unknown,
    Never,
    Object,
}

/// <ObjectType> ::= "{" <PropertyList> "}"

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ObjectTypes {
    inner: PropertyList,
}
impl ObjectTypes {
    pub fn new() -> Self {
        Self {
            inner: PropertyList::new(),
        }
    }
    pub fn push(&mut self, prop: Property) {
        self.inner.push(prop);
    }
    fn to_str(&self) -> String {
        self.inner.to_str()
    }
}

/// <PropertyList> ::= <Property> | <Property> "," <PropertyList>
#[derive(Debug, Clone, PartialEq)]
pub(super) struct PropertyList {
    inner: Vec<Property>,
}
impl PropertyList {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }
    pub fn push(&mut self, prop: Property) {
        self.inner.push(prop);
    }
    fn to_str(&self) -> String {
        let mut props = String::new();
        for prop in &self.inner {
            props.push_str(&format!("{},", prop.to_str()));
        }
        format!("{{ {} }}", props)
    }
    fn fill_sample(&self) -> String {
        let mut props = String::new();
        for prop in &self.inner {
            props.push_str(&format!("{},", prop.sample()));
        }
        format!("{{ {} }}", props)
    }
}

/// <Property> ::= <Identifier> ":" <Type>
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Property {
    ident: Identifier,
    ty: TSType,
}
impl Property {
    pub fn new(ident: Identifier, ty: TSType) -> Self {
        Self { ident, ty }
    }
    fn to_str(&self) -> String {
        format!("{}: {}", self.ident.0, self.ty.to_str())
    }
    fn sample(&self) -> String {
        format!("{}: {}", self.ident.0, self.ty.sample())
    }
}

/// <AliasType> ::= <Identifier> "=" <Type>
#[derive(Debug, Clone, PartialEq)]
pub(super) struct AliasType {
    pub name: String,
    // 見つけられない場合があるのでOption
    ty: Option<Box<TSType>>,
}
impl AliasType {
    pub fn new(name: impl Into<String>, ty: Option<Box<TSType>>) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
    fn to_str(&self) -> String {
        match &self.ty {
            Some(ty) => format!("{} = {}", self.name, ty.to_str()),
            None => self.name.clone(),
        }
    }
}
// <GenericType> ::= <Identifier> "<" <TypeList> ">"
#[derive(Debug, Clone, PartialEq)]
pub(super) struct GenericType {
    pub name: String,
    ty_list: TypeList,
}

impl GenericType {
    pub fn new(name: impl Into<String>, ty_list: TypeList) -> Self {
        Self {
            name: name.into(),
            ty_list,
        }
    }
    fn to_str(&self) -> String {
        format!("{}<{}>", self.name, self.ty_list.to_str())
    }
    fn sample(&self) -> String {
        format!("{}<{}>", self.name, self.ty_list.inner[0].sample())
    }
}

/// <TypeList> ::= <Type> | <Type> "," <TypeList>
#[derive(Debug, Clone, PartialEq)]
pub(super) struct TypeList {
    inner: Vec<TSType>,
}

impl TypeList {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }
    pub fn push(&mut self, ty: TSType) {
        self.inner.push(ty);
    }
    fn to_str(&self) -> String {
        let mut types = String::new();
        for ty in &self.inner {
            types.push_str(&format!("{},", ty.to_str()));
        }
        types
    }
}

/// <UnionType> ::= <Type> "|" <Type>
#[derive(Debug, Clone, PartialEq)]
pub(super) struct UnionType {
    inner: Vec<TSType>,
}

impl UnionType {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }
    pub fn push(&mut self, ty: TSType) {
        self.inner.push(ty);
    }
    fn to_str(&self) -> String {
        let mut types = String::new();
        for ty in &self.inner {
            types.push_str(&format!("{} | ", ty.to_str()));
        }
        types.pop();
        types.pop();
        types
    }
    fn sample(&self) -> String {
        self.inner[0].sample()
    }
}

// <IntersectionType> ::= <Type> "&" <Type>
#[derive(Debug, Clone, PartialEq)]
pub(super) struct IntersectionType {
    inner: Vec<TSType>,
}

impl IntersectionType {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }
    pub fn push(&mut self, ty: TSType) {
        self.inner.push(ty);
    }
    fn to_str(&self) -> String {
        let mut types = String::new();
        for ty in &self.inner {
            types.push_str(&format!("{} & ", ty.to_str()));
        }
        types.pop();
        types.pop();
        types
    }
    fn sample(&self) -> String {
        self.inner[0].sample()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ArrayType {
    pub ty: Box<TSType>,
}
impl ArrayType {
    pub fn new(ty: Box<TSType>) -> Self {
        Self { ty }
    }
    fn to_str(&self) -> String {
        format!("{}[]", self.ty.to_str())
    }
    fn sample(&self) -> String {
        format!("[{}]", self.ty.sample())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Identifier(pub String);
impl Identifier {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl Type {
    fn to_str(&self) -> String {
        match self {
            Type::Number => "number".to_string(),
            Type::String => "string".to_string(),
            Type::Boolean => "boolean".to_string(),
            Type::Array(ty) => format!("{}[]", ty.to_str()),
            Type::Object(props) => props.to_str(),
            Type::Named(s) => s.clone(),
        }
    }
    fn sample(&self) -> String {
        match self {
            Type::Number => "0".to_string(),
            Type::String => "\"\"".to_string(),
            Type::Boolean => "false".to_string(),
            Type::Array(ty) => format!("[{}]", ty.sample()),
            Type::Object(props) => props.fill_sample(),
            Type::Named(s) => "null".to_string(),
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_expand_str() {
        let props = ObjectType {
            inner: vec![
                (Key("timeOut".to_string()), Type::Number),
                (Key("errorMessage".to_string()), Type::String),
            ]
            .into_iter()
            .collect(),
        };
        assert!(props.to_str() == "{ errorMessage: string,timeOut: number, }");
    }
}
