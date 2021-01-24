use std::{collections::HashMap, env, error::Error, ffi::{CString, OsString}, mem, path::{Path, PathBuf}, ptr, slice, sync::{self, Arc, Mutex, mpsc::{self, Receiver, Sender}}, thread};
use std::os::windows::ffi::{OsStrExt, OsStringExt};

use crate::{watcher::*, ctypes::*};
use super::errors::*;
use super::externs::*;
use super::types::*;

const BUFFER_SIZE: usize = 16384;

#[derive(Clone)]
struct ReadData {
    dir: PathBuf,          // directory that is being watched
    file: Option<PathBuf>, // if a file is being watched, this is its full path
    complete_sem: HANDLE,
    is_recursive: bool,
}
struct WatchState {
    dir_handle: HANDLE,
    complete_sem: HANDLE,
}

struct FileWatcherServer {
    rx: Receiver<WatcherRequest>,
    cmd_tx: Sender<PathBuf>,
    meta_tx: Sender<MetaEvent>,
    event_fn: Arc<Mutex<dyn EventFn>>,
    wakeup_sem: HANDLE,
    watches: HashMap<PathBuf, WatchState>,
}

pub struct FileWatcherImpl {
    tx: Sender<WatcherRequest>,
    cmd_rx: Receiver<PathBuf>,
    wakeup_sem: HANDLE,
}

struct WatchRequest {
    event_fn: Arc<Mutex<dyn EventFn>>,
    buffer: [u8; BUFFER_SIZE as usize],
    handle: HANDLE,
    data: ReadData,
}

impl FileWatcherServer {
    fn start(event_fn: Arc<Mutex<dyn EventFn>>, meta_tx: Sender<MetaEvent>, cmd_tx: Sender<PathBuf>, semaphore: HANDLE) -> Sender<WatcherRequest> {
        let (action_tx, action_rx): (Sender<WatcherRequest>, Receiver<WatcherRequest>) = mpsc::channel();
        let sem = semaphore as u64;
        thread::spawn(move || {
            let wakeup_sem = sem as HANDLE;
            let server = FileWatcherServer{
                rx: action_rx,
                meta_tx,
                cmd_tx,
                event_fn,
                wakeup_sem,
                watches: HashMap::new(),
            };
            server.run();
        });
        action_tx
    }

    fn add_watch(&mut self, path:PathBuf, is_recursive: bool) -> PathBuf {        
        if !path.is_dir() && !path.is_file() {
            eprintln!("Unable to create a FileWatcher on a p thatath is neither a folder or a file: {}", path.to_str().unwrap());
        }
        let mut is_folder = false;
        let dir:PathBuf = if path.is_dir() {
            is_folder = true;
            path.clone()
        }
        else {
            is_folder = false;
            path.parent().unwrap().to_path_buf()
        };    
        let folder_name: Vec<u16> = dir
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();    
        let handle:HANDLE = unsafe {
            CreateFileW(folder_name.as_ptr(), 
                FILE_LIST_DIRECTORY,    
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                ptr::null_mut(),
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED,
                ptr::null_mut(),
            )
        };
         
        if handle == INVALID_HANDLE_VALUE {
            if !is_folder {
                eprintln!("Path {} cannot be opened while trying to watch a single file", dir.to_str().unwrap());
            }
            else {
                eprintln!("Path {} cannot be opened or not found", dir.to_str().unwrap());
            }
            return dir;
        }

        let semaphore = unsafe { CreateSemaphoreW(ptr::null_mut(), 0, 1, ptr::null_mut()) };
        if semaphore.is_null() || semaphore == INVALID_HANDLE_VALUE {
            eprintln!("Failed to create a semaphore for file watcher on Path {}", dir.to_str().unwrap());
            unsafe { CloseHandle(handle) };
            return dir;
        }
        let rd = ReadData {
            dir,
            file: if is_folder { None } else { Some(path.clone()) },
            complete_sem: semaphore,
            is_recursive,
        };
        let ws = WatchState {
            dir_handle: handle,
            complete_sem: semaphore,
        };

        self.watches.insert(path.clone(), ws);
        start_read(&rd, self.event_fn.clone(), handle);

        path
    }
    
    fn remove_watch(&mut self, path: PathBuf) {
        if let Some(ws) = self.watches.remove(&path) {
            stop_watch(&ws, &self.meta_tx);
        }
    }

    fn run(mut self) {
        loop {
            let mut stopped = false;
            while let Ok(request) = self.rx.try_recv() {
                match request {
                    WatcherRequest::Watch(path, is_recursive) => {
                        let res = self.add_watch(path, is_recursive);
                        let _ = self.cmd_tx.send(res);
                    }
                    WatcherRequest::Unwatch(path) => {
                        self.remove_watch(path);
                    }
                    WatcherRequest::Stop => {
                        stopped = true;
                        for ws in self.watches.values() {
                            stop_watch(ws, &self.meta_tx);
                        }
                        break;
                    }
                }
            }
            if stopped {
                break;
            }
            unsafe {
                // wait with alertable flag so that the completion routine fires
                let waitres = WaitForSingleObjectEx(self.wakeup_sem, 100, TRUE);
                if waitres == WAIT_OBJECT_0 {
                    let _ = self.meta_tx.send(MetaEvent::WatcherAwakened);
                }
            }
        }

        unsafe {
            CloseHandle(self.wakeup_sem);
        }
    }
}



impl FileWatcherImpl {
    pub fn new<F: EventFn>(event_func: F) -> Result<Self> {          
        let (meta_tx, _): (Sender<MetaEvent>, Receiver<MetaEvent>) = mpsc::channel();
        let event_fn = Arc::new(Mutex::new(event_func));  

        FileWatcherImpl::create(event_fn, meta_tx)
    }

    pub fn watch(&mut self, path: &Path, is_recursive: bool) {
        let pb = if path.is_absolute() {
            path.to_owned()
        } else {
            let p = env::current_dir().unwrap();
            p.join(path)
        };
        // path must exist and be either a file or directory
        if !pb.is_dir() && !pb.is_file() {
            eprintln!("Requesting to watch a path that is neither a file nor a directory {}", path.to_str().unwrap());
            return;
        }
        self.send_action_require_ack(WatcherRequest::Watch(pb.clone(), is_recursive), &pb);
    }
    
    fn wakeup_server(&mut self) {
        unsafe {
            ReleaseSemaphore(self.wakeup_sem, 1, ptr::null_mut());
        }
    }

    fn send_action_require_ack(&mut self, action: WatcherRequest, pb: &PathBuf) {
        let _res = self.tx.send(action);
        self.wakeup_server();

        let ack_pb = self.cmd_rx.recv().unwrap();
        if pb.as_path() != ack_pb.as_path() {
            eprintln!("Waiting ack for {:?} and received for {:?}", pb, ack_pb);
        }
    }

    pub fn unwatch(&mut self, path: &Path) {
        let pb = if path.is_absolute() {
            path.to_owned()
        } else {
            let p = env::current_dir().unwrap();
            p.join(path)
        };
        let res = self.tx.send(WatcherRequest::Unwatch(pb));
        self.wakeup_server();
    }

    fn create(event_fn: Arc<Mutex<dyn EventFn>>, meta_tx: Sender<MetaEvent>) -> Result<Self> {          
        let (cmd_tx, cmd_rx): (Sender<PathBuf>, Receiver<PathBuf>) = mpsc::channel();
        
        let wakeup_semaphore = unsafe { CreateSemaphoreW(ptr::null_mut(), 0, 1, ptr::null_mut()) };
        if wakeup_semaphore.is_null() || wakeup_semaphore == INVALID_HANDLE_VALUE {
            eprintln!("Failed to create a semaphore for file watcher");
            return Err(String::from("Failed to create a semaphore for file watcher"));
        }

        let action_tx = FileWatcherServer::start(event_fn, meta_tx, cmd_tx, wakeup_semaphore);

        Ok(Self {
            tx: action_tx,
            cmd_rx,
            wakeup_sem: wakeup_semaphore,
        })
    }
}

impl Drop for FileWatcherImpl {
    fn drop(&mut self) {
        let _ = self.tx.send(WatcherRequest::Stop);
        // better wake it up
        self.wakeup_server();
    }
}

unsafe impl Send for FileWatcherImpl {}
unsafe impl Sync for FileWatcherImpl {}



fn stop_watch(ws: &WatchState, meta_tx: &Sender<MetaEvent>) {
    unsafe {
        let cio = CancelIo(ws.dir_handle);
        let ch = CloseHandle(ws.dir_handle);
        // have to wait for it, otherwise we leak the memory allocated for there read request
        if cio != 0 && ch != 0 {
            WaitForSingleObjectEx(ws.complete_sem, INFINITE, TRUE);
        }
        CloseHandle(ws.complete_sem);
    }
    let _ = meta_tx.send(MetaEvent::SingleWatchComplete);
}

fn start_read(rd: &ReadData, event_fn: Arc<Mutex<dyn EventFn>>, handle: HANDLE) {
    let mut request = Box::new(WatchRequest {
        event_fn,
        handle,
        buffer: [0u8; BUFFER_SIZE as usize],
        data: rd.clone(),
    });
        
    let flags = FILE_NOTIFY_CHANGE_FILE_NAME
        | FILE_NOTIFY_CHANGE_DIR_NAME
        | FILE_NOTIFY_CHANGE_ATTRIBUTES
        | FILE_NOTIFY_CHANGE_SIZE
        | FILE_NOTIFY_CHANGE_LAST_WRITE
        | FILE_NOTIFY_CHANGE_CREATION
        | FILE_NOTIFY_CHANGE_SECURITY;

    let watch_subdir:bool = request.data.file.is_none() && request.data.is_recursive;
        
    let mut overlapped: Box<OVERLAPPED> = unsafe { Box::new(mem::zeroed()) };
    let req_buf = request.buffer.as_mut_ptr() as *mut c_void;
    let request_p = Box::into_raw(request) as *mut c_void;
    overlapped.hEvent = request_p;

    let res = unsafe { ReadDirectoryChangesW(
        handle,
        req_buf,
        BUFFER_SIZE as _,
        watch_subdir as BOOL,
        flags,
        &mut 0u32 as *mut u32,
        &mut *overlapped as *mut OVERLAPPED,
        Some(handle_event),
    )};
    
    if res == 0 {
        let request: Box<WatchRequest> = unsafe { mem::transmute(request_p) };
        eprintln!("Failed to create file watcher on Path {}", request.data.dir.to_str().unwrap());
        unsafe { ReleaseSemaphore(request.data.complete_sem, 1, ptr::null_mut()) };
    }
    else {
        mem::forget(overlapped);
    }
}


fn send_event(event_fn: &Mutex<dyn EventFn>, event_type: FileEvent) {
    if let Ok(guard) = event_fn.lock() {
        let f: &dyn EventFn = &*guard;
        f(event_type);
    }
}

unsafe extern "system" fn handle_event(error_code: u32,_bytes_written: u32, overlapped: LPOVERLAPPED) {
    let overlapped: Box<OVERLAPPED> = Box::from_raw(overlapped);
    let request: Box<WatchRequest> = Box::from_raw(overlapped.hEvent as *mut _);

    if error_code == ERROR_OPERATION_ABORTED {
        eprintln!("Watching operation aborted");
        ReleaseSemaphore(request.data.complete_sem, 1, ptr::null_mut());
        return;
    }

    // Process next request queued up as soon as possible
    start_read(&request.data, request.event_fn.clone(), request.handle);

    // The FILE_NOTIFY_INFORMATION struct has a variable length due to the variable length
    // string as its last member. Each struct contains an offset for getting the next entry in
    // the buffer.
    let mut cur_offset: *const u8 = request.buffer.as_ptr();
    let mut cur_entry = cur_offset as *const FILE_NOTIFY_INFORMATION;    
    loop {
        // filename length is size in bytes, so / 2
        let len = (*cur_entry).FileNameLength as usize / 2;
        let encoded_path: &[u16] = slice::from_raw_parts((*cur_entry).FileName.as_ptr(), len);
        // prepend root to get a full path
        let path = request.data.dir.join(PathBuf::from(OsString::from_wide(encoded_path)));

        let event_fn = |res| send_event(&request.event_fn, res);

        if (*cur_entry).Action == FILE_ACTION_RENAMED_OLD_NAME {
            event_fn(FileEvent::RenamedFrom(path));
        } else {
            match (*cur_entry).Action {
                FILE_ACTION_RENAMED_NEW_NAME => {
                    event_fn(FileEvent::RenamedTo(path));                        
                }
                FILE_ACTION_ADDED => {
                    event_fn(FileEvent::Created(path));                                
                }
                FILE_ACTION_REMOVED => {
                    event_fn(FileEvent::Deleted(path));        
                }
                FILE_ACTION_MODIFIED => {
                    event_fn(FileEvent::Modified(path));        
                }
                _ => (),
            };
        }            
        
        if (*cur_entry).NextEntryOffset == 0 {
            break;
        }
        cur_offset = cur_offset.offset((*cur_entry).NextEntryOffset as isize);
        cur_entry = cur_offset as *const FILE_NOTIFY_INFORMATION;
    }
}