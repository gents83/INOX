use std::{path::{Path, PathBuf}, sync::mpsc::{self, Receiver}};
use super::platform_impl::platform::watcher::FileWatcherImpl;


pub enum WatcherRequest {
    Watch(PathBuf),
    Unwatch(PathBuf),
    Stop,
}

pub enum MetaEvent {
    SingleWatchComplete,
    WatcherAwakened,
}

pub enum FileEvent{
    RenamedFrom(PathBuf),
    RenamedTo(PathBuf),
    Created(PathBuf),
    Deleted(PathBuf),
    Modified(PathBuf),
}

pub type Result<T> = std::result::Result<T, String>;

pub trait EventFn: 'static + Fn(FileEvent) + Send {}

impl<F> EventFn for F where F: 'static + Fn(FileEvent) + Send {}

pub struct FileWatcher {
    rx: Receiver<FileEvent>,
    file_watcher: FileWatcherImpl,
    filepath: PathBuf,
    filename: PathBuf,
}
unsafe impl Send for FileWatcher {}
unsafe impl Sync for FileWatcher {}


impl FileWatcher {
    pub fn new(path: PathBuf) -> Self {
        let filepath = Path::new(path.to_str().unwrap()).canonicalize().unwrap();
        let filename:PathBuf = PathBuf::from(filepath.file_name().unwrap());
        let (tx, rx) = mpsc::channel();
        let mut w = FileWatcherImpl::new(move |res| tx.send(res).unwrap() ).unwrap();
        w.watch(path.as_ref());
        Self {
            rx,
            file_watcher: w,
            filepath: filepath.to_path_buf(),
            filename: filename,
        }
    }

    pub fn stop(&mut self) {
        self.file_watcher.unwatch(self.filepath.as_path());
    }

    pub fn get_path(&self) -> PathBuf {
        self.filepath.clone()
    }

    pub fn get_filename(&self) -> PathBuf {
        self.filename.clone()
    }

    pub fn read_events(&self) -> &Receiver<FileEvent> {
        &self.rx
    }
}