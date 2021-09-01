use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn is_folder_empty(path: &Path) -> bool {
    let mut is_empty = true;
    if let Ok(dir) = std::fs::read_dir(path) {
        dir.for_each(|entry| {
            if let Ok(dir_entry) = entry {
                let path = dir_entry.path();
                is_empty &= !path.is_dir();
            }
        });
    }
    is_empty
}

pub fn for_each_file_in<F>(root: &Path, mut func: F)
where
    F: FnMut(&Path),
{
    if let Ok(dir) = std::fs::read_dir(root) {
        dir.for_each(|entry| {
            if let Ok(dir_entry) = entry {
                let path = dir_entry.path();
                if path.is_file() {
                    func(path.as_path());
                }
            }
        });
    }
}

pub fn for_each_folder_in<F>(root: &Path, mut func: F)
where
    F: FnMut(&Path),
{
    if let Ok(dir) = std::fs::read_dir(root) {
        dir.for_each(|entry| {
            if let Ok(dir_entry) = entry {
                let path = dir_entry.path();
                if path.is_dir() {
                    func(path.as_path());
                }
            }
        });
    }
}

#[inline]
pub fn convert_from_local_path(parent_folder: &Path, relative_path: &Path) -> PathBuf {
    let mut pathbuf = parent_folder.to_path_buf();
    let data_folder = pathbuf
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let string = relative_path.to_str().unwrap().to_string();
    if string.contains(parent_folder.to_str().unwrap()) {
        pathbuf = relative_path.canonicalize().unwrap()
    } else if string.contains(data_folder.as_str()) {
        pathbuf = relative_path.to_path_buf()
    } else if let Ok(result_path) = pathbuf.join(relative_path).canonicalize() {
        pathbuf = result_path;
    } else {
        eprintln!("Unable to find file: {:?}{:?}", pathbuf, relative_path);
    }
    pathbuf
}

pub fn convert_in_local_path(original_path: &Path, base_path: &Path) -> PathBuf {
    let path = original_path.to_str().unwrap().to_string();
    let path = path.replace(
        PathBuf::from(base_path)
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
        "",
    );
    let mut path = path.replace("\\", "/");
    if path.starts_with('/') {
        path.remove(0);
    }
    PathBuf::from(path)
}

#[inline]
pub fn delete_file(filepath: PathBuf) {
    let _res = std::fs::remove_file(filepath);
}

#[inline]
pub fn copy_with_random_name(src_path: PathBuf, target_path: PathBuf, name: &str, extension: &str) {
    let default_pdb_name = format!("{}{}", name, extension);
    let locked_path = src_path.join(default_pdb_name);

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    let new_name = format!("{}_{}.{}", name, secs, extension);
    let new_pdb_path = target_path.join(new_name);

    let _res = ::std::fs::rename(locked_path, new_pdb_path);
}

#[inline]
pub fn copy_all_files_with_extension(src_path: PathBuf, target_path: PathBuf, extension: &str) {
    let files = fs::read_dir(src_path).unwrap();
    files
        .filter_map(Result::ok)
        .filter(|f| match f.path().extension() {
            Some(file) => file == extension,
            _ => false,
        })
        .for_each(|f| {
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros();
            let new_name = format!(
                "{}_{}",
                secs,
                f.path().file_name().unwrap().to_str().unwrap()
            );
            let new_path = target_path.join(new_name);
            let _res = fs::rename(f.path(), new_path);
            let _res = std::fs::remove_file(f.path());
        });
}
#[inline]
pub fn link_library(name: &str) {
    println!("cargo:rustc-link-lib=dylib={}", name);
}

#[inline]
pub fn remove_files_containing_with_ext(folder: PathBuf, name: &str, extension: &str) {
    if !folder.exists() {
        return;
    }
    for entry in ::std::fs::read_dir(folder).unwrap().flatten() {
        let path = entry.path();
        if !path.is_dir() && path.extension().is_some() {
            let str_path = String::from(path.to_str().unwrap());
            if extension.contains(path.extension().unwrap().to_str().unwrap())
                && str_path.contains(name)
            {
                let _res = ::std::fs::remove_file(path);
            }
        }
    }
}
