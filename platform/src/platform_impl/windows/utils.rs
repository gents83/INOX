use std::mem::size_of;
use std::{fs::OpenOptions, path::PathBuf};
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::prelude::*;

use super::externs::*;
use super::types::*;


pub fn delete_file(filepath: PathBuf) {
    let mut opts = OpenOptions::new();
    opts.access_mode(DELETE | FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES);
    opts.custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT);
    
    let metadata = filepath.metadata().unwrap();
    let file = opts.open(filepath.clone()).unwrap();
    let mut perms = metadata.permissions();
    perms.set_readonly(false);
    let _res = file.set_permissions(perms);

    let mut info = FILE_DISPOSITION_INFO {
        DeleteFile: TRUE,
    };
    unsafe {
        SetFileInformationByHandle(
            file.as_raw_handle(),
            FILE_INFO_BY_HANDLE_CLASS::FileDispositionInfo,
            &mut info as *mut FILE_DISPOSITION_INFO as LPVOID,
            size_of::<FILE_DISPOSITION_INFO>() as u32,
        )
    };

    let _res = file.set_permissions(metadata.permissions());
    drop(file);
}