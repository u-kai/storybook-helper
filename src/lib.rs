use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

use component::Component;
pub mod cli;
mod component;
mod lexer;
mod parser;
mod token;

struct StoryBookContent {
    title: String,
    component: Component,
}

impl StoryBookContent {
    fn new(title: impl Into<String>, component: Component) -> Self {
        Self {
            title: title.into(),
            component,
        }
    }
    fn import_component(&self) -> String {
        format!(
            r#"import {{ {} }} from "./{}";"#,
            self.component.name, self.component.name
        )
    }
    fn import_libraries(&self) -> &'static str {
        r#"import React from "react";
import { StoryFn } from "@storybook/react";"#
    }
    fn export_default(&self) -> String {
        format!(
            r#"export default {{
    title: "{}",
    component: {},
}};"#,
            self.title, self.component.name
        )
    }
    fn template(&self) -> String {
        format!(
            r#"const Template: StoryFn<{}> = (args) => (
  <{} {{...args}} />
);"#,
            self.component.props_str(),
            self.component.name
        )
    }
    fn primary_sample(&self) -> String {
        format!(
            r#"export const Primary = Template.bind({{}});

Primary.args = {};"#,
            self.component.fill_sample()
        )
    }
    fn to_file_content(&self) -> String {
        format!(
            "{}\n{}\n\n{}\n\n{}\n\n{}\n",
            self.import_libraries(),
            self.import_component(),
            self.export_default(),
            self.template(),
            self.primary_sample()
        )
    }
}

fn is_stories(path: impl AsRef<Path>) -> bool {
    let Some(Some(mut split)) = path
        .as_ref()
        .file_name()
        .map(|name| name.to_str().map(|name| name.split('.')))
    else {
        return false;
    };
    if let (Some(name), Some(ext)) = (split.next(), split.next()) {
        return ext == "stories" && name.len() > 0;
    };
    false
}

fn is_tsx(path: impl AsRef<Path>) -> bool {
    !is_stories(&path)
        && path
            .as_ref()
            .extension()
            .map(|ext| ext == "tsx")
            .unwrap_or(false)
}

fn all_file_path(root: impl AsRef<Path>) -> Result<Vec<PathBuf>, std::io::Error> {
    let Ok(root) = read_dir(&root) else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("not found path = {:?}", root.as_ref()),
        ));
    };
    root.filter_map(|entry| entry.ok())
        .filter_map(|entry| match entry.file_type() {
            Ok(file_type) => Some((file_type, entry.path())),
            Err(_) => None,
        })
        .try_fold(Vec::new(), |mut acc, (file_type, path)| {
            if !file_type.is_dir() {
                acc.push(path);
                return Ok(acc);
            }
            let Ok(mut files) = all_file_path(&path) else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("not found path = {:?}", path),
                ));
            };
            acc.append(&mut files);
            Ok(acc)
        })
}

#[cfg(test)]

mod tests {
    use crate::component::{Key, NamedProps, ObjectType, Props, Type};

    use super::*;
    use std::path::{Path, PathBuf};

    fn create_dir_all(dir_name: &str) {
        std::fs::create_dir_all(dir_name).unwrap();
    }
    fn create_file(file_name: &str, content: &str) {
        std::fs::write(file_name, content).unwrap();
    }
    fn remove_dir(dir_name: &str) {
        let path: &Path = dir_name.as_ref();
        if path.exists() {
            std::fs::remove_dir_all(dir_name).unwrap();
        }
    }
    #[test]
    fn test_make_storybook_content() {
        let mut props = ObjectType::new();
        props.insert(Key("timeOut".to_string()), Type::Number);
        props.insert(Key("errorMessage".to_string()), Type::String);

        let props = NamedProps::new("Props", props);
        let component = Component::new("ErrorAlert", Props::Named(props));

        let storybook_content = StoryBookContent::new("Sample/ErrorAlert", component);
        assert_eq!(
            storybook_content.to_file_content(),
            r#"import React from "react";
import { StoryFn } from "@storybook/react";
import { ErrorAlert } from "./ErrorAlert";

export default {
    title: "Sample/ErrorAlert",
    component: ErrorAlert,
};

const Template: StoryFn<Props> = (args) => (
  <ErrorAlert {...args} />
);

export const Primary = Template.bind({});

Primary.args = { errorMessage: "",timeOut: 0, };
"#
        );
    }
    #[test]
    fn test_is_stories() {
        let dir_name = "test_all_file_path";
        assert!(is_stories("test.stories.tsx"));
        assert!(is_stories(format!("{dir_name}/test.stories.tsx")));
        assert!(!is_stories("test.ts"));
        assert!(!is_stories("test.rs"));
        assert!(!is_stories("test.tsx"));
    }
    #[test]
    fn test_is_tsx() {
        let dir_name = "test_all_file_path2";
        assert!(is_tsx("test.tsx"));
        assert!(is_tsx(format!("{dir_name}/test.tsx")));
        assert!(!is_tsx("test.ts"));
        assert!(!is_tsx("test.rs"));
        assert!(!is_tsx("test.stories.tsx"));
    }
    #[test]
    fn test_all_file_path() {
        let dir_name = "test_all_file_path";
        create_dir_all(format!("{dir_name}/sub_dir").as_str());
        create_file(format!("{dir_name}/main.rs").as_str(), "");
        create_file(format!("{dir_name}/lib.rs").as_str(), "");
        create_file(format!("{dir_name}/sub_dir/main.py").as_str(), "");

        let files = super::all_file_path(dir_name).unwrap();
        remove_dir(dir_name);
        assert_eq!(files.len(), 3);
        assert!(files.contains(&PathBuf::from(format!("{}/main.rs", dir_name))),);
        assert!(files.contains(&PathBuf::from(format!("{}/lib.rs", dir_name))),);
        assert!(files.contains(&PathBuf::from(format!("{}/sub_dir/main.py", dir_name))),);
    }
}
