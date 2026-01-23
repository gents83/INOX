use super::platform_impl::platform::watcher::FileWatcherImpl;
use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
};

pub enum WatcherRequest {
    Watch(PathBuf),
    Unwatch(PathBuf),
    Stop,
}

pub enum MetaEvent {
    SingleWatchComplete,
    WatcherAwakened,
}

#[derive(Debug)]
pub enum FileEvent {
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
    rx: Receiver<String>,
    file_watcher: FileWatcherImpl,
    filepath: PathBuf,
    filename: PathBuf,
}
unsafe impl Send for FileWatcher {}
unsafe impl Sync for FileWatcher {}

impl FileWatcher {
    pub fn new(path: PathBuf) -> Self {
        let filepath = Path::new(path.to_str().unwrap()).canonicalize().unwrap();
        let filename: PathBuf = PathBuf::from(filepath.file_name().unwrap());
        let (tx, rx) = mpsc::channel();
        #[cfg(not(target_os = "android"))]
        #[cfg(not(target_os = "ios"))]
        let w = FileWatcherImpl::new(move |res: FileEvent| tx.send(format!("{res:?}")).unwrap())
            .unwrap();
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let w = FileWatcherImpl::new(
            filepath.clone(),
            move |res: &str| tx.send(res.to_string()).unwrap(),
        )
        .unwrap();
        Self {
            rx,
            file_watcher: w,
            filepath,
            filename,
        }
    }

    #[inline]
    pub fn stop(&mut self) {
        self.file_watcher.unwatch(self.filepath.as_path());
    }

    #[inline]
    pub fn get_path(&self) -> PathBuf {
        self.filepath.clone()
    }
    #[inline]
    pub fn get_filename(&self) -> PathBuf {
        self.filename.clone()
    }

    #[inline]
    pub fn read_events(&self) -> &Receiver<String> {
        &self.rx
    }
}
