use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn delete_file(filepath: PathBuf) {
    let res = std::fs::remove_file(filepath.clone());
    if res.is_err() {
        super::platform_impl::platform::utils::delete_file(filepath.clone());
        let _res = std::fs::remove_file(filepath);
    }
}

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
        });
}

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
