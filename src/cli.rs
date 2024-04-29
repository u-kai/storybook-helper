use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use clap::Parser;

use crate::{all_file_path, component::TSXContent, is_tsx, to_stories_path, StoryBookContent};

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
        let path = Path::new(&self.root);
        if path.is_file() {
            return self.run_to_file(path);
        }
        let files = all_file_path(path)?;
        files
            .into_iter()
            .filter(|path| is_tsx(path))
            .try_for_each(|path| {
                self.run_to_file(&path)?;
                Ok(())
            })
    }
    fn run_to_file(&self, path: &Path) -> Result<(), std::io::Error> {
        let content = TSXContent::from_file(&path)?;
        let Some(component) = content.to_component() else {
            return Ok(());
        };
        let storybook =
            StoryBookContent::new(format!("Example/{}", component.name.as_str()), component);
        let mut file = File::create(to_stories_path(path)).unwrap();
        file.write_all(
            storybook
                .to_file_content(path.file_stem().unwrap().to_str().unwrap())
                .as_bytes(),
        )?;
        Ok(())
    }
}
