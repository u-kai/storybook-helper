use clap::Parser;

use crate::{all_file_path, is_tsx, parser::TSXContent, StoryBookContent};

#[derive(Parser)]
pub struct Cli {
    #[clap(default_value = "src")]
    root: String,
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }
    pub fn run(&self) -> Result<(), std::io::Error> {
        let files = all_file_path(&self.root)?;
        files
            .into_iter()
            .filter(|path| is_tsx(path))
            .try_for_each(|path| {
                let content = TSXContent::from_file(&path)?;
                let component = content.to_component();
                let storybook = StoryBookContent::new(
                    format!("Example/{}", component.name.as_str()),
                    component,
                );
                println!("{}", storybook.to_file_content());
                Ok(())
            })
    }
}
