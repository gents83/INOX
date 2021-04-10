#![allow(improper_ctypes_definitions)]

use std::{
    cell::Cell,
    collections::HashMap,
    convert::TryInto,
    fs::File,
    io::BufWriter,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Mutex,
    },
    thread::{self, ThreadId},
    time::Instant,
    u64,
};

pub static mut NRG_PROFILER_LIB: Option<nrg_platform::Library> = None;

pub const CREATE_PROFILER_FUNCTION_NAME: &str = "create_profiler";
pub type PFNCreateProfiler = ::std::option::Option<unsafe extern "C" fn()>;
pub const REGISTER_THREAD_FUNCTION_NAME: &str = "register_thread";
pub type PFNRegisterThread = ::std::option::Option<unsafe extern "C" fn(*const u8)>;
pub const GET_ELAPSED_TIME_FUNCTION_NAME: &str = "get_elapsed_time";
pub type PFNGetElapsedTime = ::std::option::Option<unsafe extern "C" fn() -> u64>;
pub const ADD_SAMPLE_FOR_THREAD_FUNCTION_NAME: &str = "add_sample_for_thread";
pub type PFNAddSampleForThread = ::std::option::Option<unsafe extern "C" fn(&str, u64, u64)>;
pub const WRITE_PROFILE_FILE_FUNCTION_NAME: &str = "write_profile_file";
pub type PFNWriteProfileFile = ::std::option::Option<unsafe extern "C" fn()>;

pub static GLOBAL_PROFILER: GlobalProfiler = GlobalProfiler(Cell::new(None));

#[repr(C)]
pub struct GlobalProfiler(Cell<Option<Mutex<Profiler>>>);
unsafe impl Sync for GlobalProfiler {}
unsafe impl Send for GlobalProfiler {}

#[no_mangle]
pub extern "C" fn create_profiler() {
    GLOBAL_PROFILER.0.set(Some(Mutex::new(Profiler::new())));
    let main = "main\0";
    register_thread(main.as_ptr());
}

#[no_mangle]
pub extern "C" fn register_thread(name: *const u8) {
    unsafe {
        match *GLOBAL_PROFILER.0.as_ptr() {
            Some(ref x) => {
                if name == std::ptr::null() {
                    x.lock().unwrap().register_thread(None);
                } else if let Ok(str) = std::ffi::CStr::from_ptr(name as *const i8).to_str() {
                    x.lock().unwrap().register_thread(Some(str));
                } else {
                    x.lock().unwrap().register_thread(None);
                }
            }
            None => panic!("Trying to register_thread on an uninitialized static global variable"),
        }
    }
}

#[no_mangle]
pub extern "C" fn add_sample_for_thread(name: &str, start: u64, end: u64) {
    unsafe {
        match *GLOBAL_PROFILER.0.as_ptr() {
            Some(ref x) => {
                x.lock().unwrap().add_sample_for_thread(name, start, end);
            }
            None => {
                panic!("Trying to add_sample_for_thread on an uninitialized static global variable")
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn get_elapsed_time() -> u64 {
    unsafe {
        match *GLOBAL_PROFILER.0.as_ptr() {
            Some(ref x) => x.lock().unwrap().get_elapsed_time(),
            None => {
                panic!("Trying to write_profile_file on an uninitialized static global variable")
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn write_profile_file() {
    unsafe {
        match *GLOBAL_PROFILER.0.as_ptr() {
            Some(ref x) => {
                let _profile_file = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "app.nrg_profile");
                x.lock().unwrap().write_profile_file("app.nrg_profile");
            }
            None => {
                panic!("Trying to write_profile_file on an uninitialized static global variable")
            }
        }
    }
}

#[repr(C)]
struct ThreadInfo {
    name: String,
    profiler: ThreadProfiler,
}

struct ThreadProfiler {
    id: ThreadId,
    tx: Sender<Sample>,
}

impl ThreadProfiler {
    fn push_sample(&self, name: String, time_start: u64, time_end: u64) {
        let sample = Sample {
            tid: self.id,
            name,
            time_start,
            time_end,
        };
        self.tx.send(sample).ok();
    }
}

#[repr(C)]
struct Sample {
    tid: ThreadId,
    name: String,
    time_start: u64,
    time_end: u64,
}

#[repr(C)]
pub struct Profiler {
    time_start: Instant,
    rx: Receiver<Sample>,
    tx: Sender<Sample>,
    threads: HashMap<ThreadId, ThreadInfo>,
}
unsafe impl Sync for Profiler {}
unsafe impl Send for Profiler {}

impl Profiler {
    fn new() -> Profiler {
        let (tx, rx) = channel();

        Profiler {
            time_start: Instant::now(),
            rx,
            tx,
            threads: HashMap::new(),
        }
    }
    pub fn get_elapsed_time(&self) -> u64 {
        let elapsed = self.time_start.elapsed();
        elapsed.as_millis().try_into().unwrap()
    }

    pub fn register_thread(&mut self, optional_name: Option<&str>) {
        let id = thread::current().id();

        let name = {
            if let Some(name) = optional_name {
                name.to_string()
            } else if let Some(s) = thread::current().name() {
                s.to_string()
            } else {
                format!("<unknown-{:?}>", id)
            }
        };

        self.threads.entry(id).or_insert(ThreadInfo {
            name,
            profiler: ThreadProfiler {
                id,
                tx: self.tx.clone(),
            },
        });
    }
    pub fn add_sample_for_thread(&mut self, name: &str, start: u64, end: u64) {
        let id = thread::current().id();
        if let Some(thread) = self.threads.get(&id) {
            thread.profiler.push_sample(String::from(name), start, end);
        } else {
            panic!("Invalid thread id {:?}", id);
        }
    }

    pub fn write_profile_file(&self, filename: &str) {
        let start_time = self.get_elapsed_time();
        let mut data = Vec::new();

        while let Ok(sample) = self.rx.try_recv() {
            if sample.time_start > start_time {
                break;
            }

            if let Some(thread) = self.threads.get(&sample.tid) {
                let thread_id = thread.name.as_str();
                data.push(serde_json::json!({
                    "pid": 0,
                    "tid": thread_id,
                    "name": sample.name,
                    "ph": "B",
                    "ts": sample.time_start
                }));
                data.push(serde_json::json!({
                    "pid": 0,
                    "tid": thread_id,
                    "ph": "E",
                    "ts": sample.time_end
                }));
            } else {
                panic!("Invalid thread id {:?}", sample.tid);
            }
        }

        let f = BufWriter::new(File::create(filename).unwrap());
        serde_json::to_writer(f, &data).unwrap();
    }
}

pub struct ScopedProfile {
    name: String,
    time_start: u64,
}

impl ScopedProfile {
    pub fn new(name: String) -> ScopedProfile {
        let mut time: u64 = 0;
        unsafe {
            if let Some(get_elapsed_fn) = NRG_PROFILER_LIB
                .as_ref()
                .unwrap()
                .get::<PFNGetElapsedTime>(GET_ELAPSED_TIME_FUNCTION_NAME)
            {
                time = get_elapsed_fn.unwrap()();
            }
        }
        ScopedProfile {
            name,
            time_start: time,
        }
    }
}

impl Drop for ScopedProfile {
    fn drop(&mut self) {
        unsafe {
            if let Some(add_sample_fn) = NRG_PROFILER_LIB
                .as_ref()
                .unwrap()
                .get::<PFNAddSampleForThread>(ADD_SAMPLE_FOR_THREAD_FUNCTION_NAME)
            {
                if let Some(get_elapsed_fn) = NRG_PROFILER_LIB
                    .as_ref()
                    .unwrap()
                    .get::<PFNGetElapsedTime>(GET_ELAPSED_TIME_FUNCTION_NAME)
                {
                    add_sample_fn.unwrap()(
                        self.name.as_str(),
                        self.time_start,
                        get_elapsed_fn.unwrap()(),
                    );
                }
            }
        }
    }
}
