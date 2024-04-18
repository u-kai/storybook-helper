use std::collections::HashMap;

pub(super) struct Component {
    pub name: String,
    props: NamedProps,
}

struct NamedProps {
    pub name: String,
    inner: Props,
}

impl NamedProps {
    fn new(name: impl Into<String>, inner: Props) -> Self {
        Self {
            name: name.into(),
            inner,
        }
    }
    fn expand_str(&self) -> String {
        self.inner.to_str()
    }
}

pub(super) struct Props {
    inner: HashMap<Key, Type>,
}

impl Props {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
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
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub(super) struct Key(String);

pub(super) enum Type {
    Number,
    String,
    Boolean,
    Array(Box<Type>),
    Object(Props),
    Original(String),
}

impl Type {
    fn to_str(&self) -> String {
        match self {
            Type::Number => "number".to_string(),
            Type::String => "string".to_string(),
            Type::Boolean => "boolean".to_string(),
            Type::Array(ty) => format!("{}[]", ty.to_str()),
            Type::Object(props) => props.to_str(),
            Type::Original(s) => s.clone(),
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_expand_str() {
        let props = Props {
            inner: vec![
                (Key("timeOut".to_string()), Type::Number),
                (Key("errorMessage".to_string()), Type::String),
            ]
            .into_iter()
            .collect(),
        };
        assert!(
            props.to_str() == "{ timeOut: number,errorMessage: string, }"
                || props.to_str() == "{ errorMessage: string,timeOut: number, }"
        );
    }
}
