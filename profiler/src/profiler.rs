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
pub type PfnCreateProfiler = ::std::option::Option<unsafe extern "C" fn()>;
pub const START_PROFILER_FUNCTION_NAME: &str = "start_profiler";
pub type PfnStartProfiler = ::std::option::Option<unsafe extern "C" fn()>;
pub const STOP_PROFILER_FUNCTION_NAME: &str = "stop_profiler";
pub type PfnStopProfiler = ::std::option::Option<unsafe extern "C" fn()>;
pub const REGISTER_THREAD_FUNCTION_NAME: &str = "register_thread";
pub type PfnRegisterThread = ::std::option::Option<unsafe extern "C" fn(*const u8)>;
pub const GET_ELAPSED_TIME_FUNCTION_NAME: &str = "get_elapsed_time";
pub type PfnGetElapsedTime = ::std::option::Option<unsafe extern "C" fn() -> u64>;
pub const ADD_SAMPLE_FOR_THREAD_FUNCTION_NAME: &str = "add_sample_for_thread";
pub type PfnAddSampleForThread = ::std::option::Option<unsafe extern "C" fn(&str, &str, u64, u64)>;
pub const WRITE_PROFILE_FILE_FUNCTION_NAME: &str = "write_profile_file";
pub type PfnWriteProfileFile = ::std::option::Option<unsafe extern "C" fn()>;

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
pub extern "C" fn start_profiler() {
    unsafe {
        match *GLOBAL_PROFILER.0.as_ptr() {
            Some(ref x) => {
                x.lock().unwrap().start();
                println!("Starting profiler");
            }
            None => panic!("Trying to start_profiler on an uninitialized static global variable"),
        }
    }
}

#[no_mangle]
pub extern "C" fn stop_profiler() {
    unsafe {
        match *GLOBAL_PROFILER.0.as_ptr() {
            Some(ref x) => {
                println!("Stopping profiler");
                x.lock().unwrap().stop();
            }
            None => panic!("Trying to stop_profiler on an uninitialized static global variable"),
        }
    }
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
pub extern "C" fn add_sample_for_thread(category: &str, name: &str, start: u64, end: u64) {
    unsafe {
        match *GLOBAL_PROFILER.0.as_ptr() {
            Some(ref x) => {
                x.lock()
                    .unwrap()
                    .add_sample_for_thread(category, name, start, end);
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
                x.lock().unwrap().write_profile_file();
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
    fn push_sample(&self, category: String, name: String, time_start: u64, time_end: u64) {
        let sample = Sample {
            tid: self.id,
            category,
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
    category: String,
    name: String,
    time_start: u64,
    time_end: u64,
}

#[repr(C)]
pub struct Profiler {
    started: bool,
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
            started: false,
            time_start: Instant::now(),
            rx,
            tx,
            threads: HashMap::new(),
        }
    }
    pub fn start(&mut self) {
        let _ = self.rx.try_recv();
        self.started = true;
        self.time_start = Instant::now();
    }
    pub fn stop(&mut self) {
        self.started = false;
        self.write_profile_file();
    }
    pub fn get_elapsed_time(&self) -> u64 {
        let elapsed = self.time_start.elapsed();
        elapsed.as_micros().try_into().unwrap()
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
    pub fn add_sample_for_thread(&mut self, category: &str, name: &str, start: u64, end: u64) {
        if !self.started {
            return;
        }
        let id = thread::current().id();
        if let Some(thread) = self.threads.get(&id) {
            thread
                .profiler
                .push_sample(String::from(category), String::from(name), start, end);
        } else {
            panic!("Invalid thread id {:?}", id);
        }
    }

    pub fn write_profile_file(&self) {
        let start_time = self.get_elapsed_time();
        let mut data = Vec::new();

        let index = 0;
        while let Ok(sample) = self.rx.try_recv() {
            if sample.time_start > start_time {
                break;
            }

            if let Some(thread) = self.threads.get(&sample.tid) {
                let thread_id = thread.name.as_str();
                data.push(serde_json::json!({
                    "pid": 0,
                    "id": index,
                    "cat": sample.category,
                    "tid": thread_id,
                    "name": sample.name,
                    "ph": "b",
                    "ts": sample.time_start,
                }));
                data.push(serde_json::json!({
                    "pid": 0,
                    "id": index,
                    "cat": sample.category,
                    "tid": thread_id,
                    "name": sample.name,
                    "ph": "e",
                    "ts": sample.time_end,
                }));
            } else {
                panic!("Invalid thread id {:?}", sample.tid);
            }
        }

        let f = BufWriter::new(File::create("app.nrg_profile").unwrap());
        serde_json::to_writer(f, &data).unwrap();
    }
}

pub struct ScopedProfile {
    category: String,
    name: String,
    time_start: u64,
}

impl ScopedProfile {
    pub fn new(category: &str, name: &str) -> ScopedProfile {
        let mut time: u64 = 0;
        unsafe {
            if let Some(get_elapsed_fn) = NRG_PROFILER_LIB
                .as_ref()
                .unwrap()
                .get::<PfnGetElapsedTime>(GET_ELAPSED_TIME_FUNCTION_NAME)
            {
                time = get_elapsed_fn.unwrap()();
            }
        }
        ScopedProfile {
            category: category.to_string(),
            name: name.to_string(),
            time_start: time,
        }
    }
}

impl Drop for ScopedProfile {
    fn drop(&mut self) {
        unsafe {
            let mut time_end = self.time_start;
            if let Some(get_elapsed_fn) = NRG_PROFILER_LIB
                .as_ref()
                .unwrap()
                .get::<PfnGetElapsedTime>(GET_ELAPSED_TIME_FUNCTION_NAME)
            {
                time_end = get_elapsed_fn.unwrap()();
            }
            if let Some(add_sample_fn) = NRG_PROFILER_LIB
                .as_ref()
                .unwrap()
                .get::<PfnAddSampleForThread>(ADD_SAMPLE_FOR_THREAD_FUNCTION_NAME)
            {
                add_sample_fn.unwrap()(
                    self.name.as_str(),
                    self.name.as_str(),
                    self.time_start,
                    time_end,
                );
            }
        }
    }
}
