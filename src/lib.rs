use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

pub fn all_file_path(root: impl AsRef<Path>) -> Result<Vec<PathBuf>, std::io::Error> {
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
    use std::path::{Path, PathBuf};

    fn make_test_dir(dir_name: &str) {
        let sub_dir_name = format!("{}/sub_dir", dir_name);
        std::fs::create_dir_all(&sub_dir_name).unwrap();
        std::fs::write(format!("{}/main.rs", dir_name), "fn main() {}").unwrap();
        std::fs::write(format!("{}/lib.rs", dir_name), "fn lib() {}").unwrap();
        std::fs::write(format!("{}/main.py", sub_dir_name), "def main(): pass").unwrap();
    }
    fn remove_dir(dir_name: &str) {
        let path: &Path = dir_name.as_ref();
        if path.exists() {
            std::fs::remove_dir_all(dir_name).unwrap();
        }
    }
    #[test]
    fn test_all_file_path() {
        let dir_name = "test_all_file_path";
        make_test_dir(dir_name);
        let files = super::all_file_path(dir_name).unwrap();
        remove_dir(dir_name);
        assert_eq!(files.len(), 3);
        assert!(files.contains(&PathBuf::from(format!("{}/main.rs", dir_name))),);
        assert!(files.contains(&PathBuf::from(format!("{}/lib.rs", dir_name))),);
        assert!(files.contains(&PathBuf::from(format!("{}/sub_dir/main.py", dir_name))),);
    }
}
