use std::collections::HashMap;

pub(super) struct Component {
    pub name: String,
    props: Props,
}

struct Props {
    name: String,
    inner: HashMap<Key, Type>,
}
impl Props {
    fn expand_str(&self) -> String {
        let mut props = String::new();
        for (key, ty) in &self.inner {
            props.push_str(&format!("{}: {},", key.0, ty.0));
        }
        format!("{{ {} }}", props)
    }
}
struct Key(String);
struct Type(String);
