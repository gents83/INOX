use std::{collections::HashMap, env, ffi::{CString, OsString}, mem, path::{Path, PathBuf}, ptr, slice, sync::{self, Arc, Mutex, mpsc::{self, Receiver, Sender}}, thread};
use std::os::windows::ffi::{OsStrExt, OsStringExt};

use crate::{watcher::*, ctypes::*};
use super::errors::*;
use super::externs::*;
use super::types::*;

const BUFFER_SIZE: usize = 2048 * std::mem::size_of::<c_char>();
const THREAD_WAIT_INTERVAL: u32 = 100;

#[derive(Clone)]
struct ReadData {
    dir: PathBuf,          // directory that is being watched
    complete_sem: HANDLE,
}

struct WatchHandles {
    dir_handle: HANDLE,
    complete_sem: HANDLE,
}
struct WatchRequest {
    event_fn: Arc<Mutex<dyn EventFn>>,
    buffer: [u8; BUFFER_SIZE as usize],
    handle: HANDLE,
    data: ReadData,
}

struct FileWatcherServer {
    rx: Receiver<WatcherRequest>,
    event_fn: Arc<Mutex<dyn EventFn>>,
    semaphore: HANDLE,
    watches: HashMap<PathBuf, WatchHandles>,
}

pub struct FileWatcherImpl {
    tx: Sender<WatcherRequest>,
    semaphore: HANDLE,
}


impl FileWatcherServer {
    fn start(event_fn: Arc<Mutex<dyn EventFn>>, semaphore: HANDLE) -> Sender<WatcherRequest> {
        let (action_tx, action_rx): (Sender<WatcherRequest>, Receiver<WatcherRequest>) = mpsc::channel();
        let wakeup_semaphore = semaphore as u64;
        thread::spawn(move || {
            let server = FileWatcherServer{
                rx: action_rx,
                event_fn,
                semaphore: wakeup_semaphore as HANDLE,
                watches: HashMap::new(),
            };
            server.run();
        });
        action_tx
    }

    fn add_watch(&mut self, path:PathBuf) -> PathBuf {        
        if !path.is_dir() && !path.is_file() {
            eprintln!("Unable to create a FileWatcher on a p thatath is neither a folder or a file: {}", path.to_str().unwrap());
        }
        let dir:PathBuf = if path.is_dir() {
            path.clone()
        }
        else {
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
            eprintln!("Path {} cannot be opened or not found", dir.to_str().unwrap());
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
            complete_sem: semaphore,
        };
        let watch_handles = WatchHandles {
            dir_handle: handle,
            complete_sem: semaphore,
        };
        self.watches.insert(path.clone(), watch_handles);
        process_folder(&rd, self.event_fn.clone(), handle);

        path
    }
    
    fn remove_watch(&mut self, path: PathBuf) {
        if let Some(ws) = self.watches.remove(&path) {
            stop_watch(&ws);
        }
    }

    fn run(mut self) {
        loop {
            let mut stopped = false;
            while let Ok(request) = self.rx.try_recv() {
                match request {
                    WatcherRequest::Watch(path) => {
                        let _res = self.add_watch(path);
                    }
                    WatcherRequest::Unwatch(path) => {
                        self.remove_watch(path);
                    }
                    WatcherRequest::Stop => {
                        stopped = true;
                        for ws in self.watches.values() {
                            stop_watch(ws);
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
                WaitForSingleObjectEx(self.semaphore, THREAD_WAIT_INTERVAL, TRUE);
            }
        }

        unsafe {
            CloseHandle(self.semaphore);
        }
    }
}



impl FileWatcherImpl {
    pub fn new<F: EventFn>(event_func: F) -> Result<Self> {          
        let event_fn = Arc::new(Mutex::new(event_func));     
        
        let semaphore = unsafe { CreateSemaphoreW(ptr::null_mut(), 0, 1, ptr::null_mut()) };
        if semaphore.is_null() || semaphore == INVALID_HANDLE_VALUE {
            eprintln!("Failed to create a semaphore for file watcher");
            return Err(String::from("Failed to create a semaphore for file watcher"));
        }

        Ok(Self {
            tx: FileWatcherServer::start(event_fn, semaphore),
            semaphore,
        })
    }

    pub fn watch(&mut self, path: &Path) {        
        let pb = self.get_absolute_path(path);
        self.send_action_require_ack(WatcherRequest::Watch(pb.clone()), &pb);
    }

    pub fn unwatch(&mut self, path: &Path) {
        let pb = self.get_absolute_path(path);
        let _res = self.tx.send(WatcherRequest::Unwatch(pb));
        self.wakeup_server();
    }

    fn get_absolute_path(&self, path: &Path) -> PathBuf {        
        let pb = if path.is_absolute() {
            path.to_owned()
        } else {
            let p = env::current_dir().unwrap();
            p.join(path)
        };
        if !pb.is_dir() && !pb.is_file() {
            eprintln!("Requesting to watch a path that is neither a file nor a directory {}", path.to_str().unwrap());
        }
        pb
    }
    
    fn wakeup_server(&mut self) {
        unsafe {
            ReleaseSemaphore(self.semaphore, 1, ptr::null_mut());
        }
    }

    fn send_action_require_ack(&mut self, action: WatcherRequest, pb: &PathBuf) {
        let _res = self.tx.send(action);
        self.wakeup_server();
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



fn stop_watch(ws: &WatchHandles) {
    unsafe {
        let cio = CancelIo(ws.dir_handle);
        let ch = CloseHandle(ws.dir_handle);
        // have to wait for it, otherwise we leak the memory allocated for there read request
        if cio != 0 && ch != 0 {
            WaitForSingleObjectEx(ws.complete_sem, INFINITE, TRUE);
        }
        CloseHandle(ws.complete_sem);
    }
}

fn process_folder(rd: &ReadData, event_fn: Arc<Mutex<dyn EventFn>>, handle: HANDLE) {
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
        
    let mut overlapped: Box<OVERLAPPED> = unsafe { Box::new(mem::zeroed()) };
    let req_buf = request.buffer.as_mut_ptr() as *mut c_void;
    let request_p = Box::into_raw(request) as *mut c_void;
    overlapped.hEvent = request_p;

    let res = unsafe { ReadDirectoryChangesW(
        handle,
        req_buf,
        BUFFER_SIZE as _,
        TRUE,
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

    process_folder(&request.data, request.event_fn.clone(), request.handle);

    let mut cur_offset: *const u8 = request.buffer.as_ptr();
    let mut cur_entry = cur_offset as *const FILE_NOTIFY_INFORMATION;    
    loop {
        let entry = *cur_entry;
        let encoded_path: &[u16] = slice::from_raw_parts(entry.FileName.as_ptr(), entry.FileNameLength as usize / 2);
        let path = request.data.dir.join(PathBuf::from(OsString::from_wide(encoded_path)));
        let event_fn = |res| send_event(&request.event_fn, res);

        if entry.Action == FILE_ACTION_RENAMED_OLD_NAME {
            event_fn(FileEvent::RenamedFrom(path));
        } else {
            match entry.Action {
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
        
        if entry.NextEntryOffset == 0 {
            break;
        }
        cur_offset = cur_offset.offset(entry.NextEntryOffset as isize);
        cur_entry = cur_offset as *const FILE_NOTIFY_INFORMATION;
    }
}