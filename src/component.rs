use std::collections::BTreeMap;

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

pub(super) enum Props {
    Named(NamedProps),
    Expand(ExpandProps),
}

pub(super) struct NamedProps {
    pub name: String,
    inner: ExpandProps,
}

impl NamedProps {
    pub fn new(name: impl Into<String>, inner: ExpandProps) -> Self {
        Self {
            name: name.into(),
            inner,
        }
    }
    pub fn expand_str(&self) -> String {
        self.inner.to_str()
    }
}

pub(super) struct ExpandProps {
    inner: BTreeMap<Key, Type>,
}

impl ExpandProps {
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

pub(super) enum Type {
    Number,
    String,
    Boolean,
    Array(Box<Type>),
    Object(ExpandProps),
    Named(String),
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
        let props = ExpandProps {
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
