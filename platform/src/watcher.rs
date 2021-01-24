use std::{path::PathBuf, sync::mpsc::{self, Receiver, Sender}};
use super::platform_impl::platform::watcher::FileWatcherImpl;


pub enum WatcherRequest {
    Watch(PathBuf, bool),
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
    watcher_impl: FileWatcherImpl,
}
unsafe impl Send for FileWatcher {}
unsafe impl Sync for FileWatcher {}


impl FileWatcher {
    pub fn new(path: PathBuf, is_recursive: bool) -> Self {
        let (tx, rx) = mpsc::channel();
        let mut w = FileWatcherImpl::new(move |res| tx.send(res).unwrap() ).unwrap();
        w.watch(path.as_ref(), is_recursive);
        Self {
            rx,
            watcher_impl: w,
        }
    }

    pub fn read_events(&self) -> &Receiver<FileEvent> {
        &self.rx
    }
}