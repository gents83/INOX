use std::{path::PathBuf, sync::mpsc::{self, Receiver}};
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
    _file_watcher: FileWatcherImpl,
}
unsafe impl Send for FileWatcher {}
unsafe impl Sync for FileWatcher {}


impl FileWatcher {
    pub fn new(path: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel();
        let mut w = FileWatcherImpl::new(move |res| tx.send(res).unwrap() ).unwrap();
        w.watch(path.as_ref());
        Self {
            rx,
            _file_watcher: w,
        }
    }

    pub fn read_events(&self) -> &Receiver<FileEvent> {
        &self.rx
    }
}