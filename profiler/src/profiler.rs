#![allow(improper_ctypes_definitions)]

use std::{
    collections::HashMap,
    convert::TryInto,
    fs::File,
    io::BufWriter,
    process,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, RwLock,
    },
    thread::{self, ThreadId},
    time::Instant,
    u64,
};

pub type GlobalProfiler = Arc<RwLock<Profiler>>;
pub static mut NRG_PROFILER_LIB: Option<nrg_platform::Library> = None;

pub const GET_PROFILER_FUNCTION_NAME: &str = "get_profiler";
pub type PfnGetProfiler = ::std::option::Option<unsafe extern "C" fn() -> GlobalProfiler>;

pub const CREATE_PROFILER_FUNCTION_NAME: &str = "create_profiler";
pub type PfnCreateProfiler = ::std::option::Option<unsafe extern "C" fn()>;

pub static mut GLOBAL_PROFILER: Option<GlobalProfiler> = None;

#[no_mangle]
pub extern "C" fn get_profiler() -> GlobalProfiler {
    unsafe { GLOBAL_PROFILER.as_ref().unwrap().clone() }
}

#[no_mangle]
pub extern "C" fn create_profiler() {
    unsafe {
        GLOBAL_PROFILER.replace(Arc::new(RwLock::new(Profiler::new())));
    }
    let main = "main\0";
    register_thread(main.as_ptr());
}

fn register_thread(name: *const u8) {
    unsafe {
        let profiler = GLOBAL_PROFILER.as_ref().unwrap();
        if name == std::ptr::null() {
            profiler.write().unwrap().register_thread(None);
        } else if let Ok(str) = std::ffi::CStr::from_ptr(name as *const i8).to_str() {
            profiler.write().unwrap().register_thread(Some(str));
        } else {
            profiler.write().unwrap().register_thread(None);
        }
    }
}

#[repr(C)]
struct ThreadInfo {
    name: String,
    index: usize,
    profiler: ThreadProfiler,
}

struct ThreadProfiler {
    id: ThreadId,
    tx: Sender<Sample>,
}

impl ThreadProfiler {
    fn push_sample(&self, category: String, name: String, time_start: f64, time_end: f64) {
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
    time_start: f64,
    time_end: f64,
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
    pub fn is_started(&self) -> bool {
        self.started
    }
    pub fn start(&mut self) {
        let _ = self.rx.try_recv();
        self.started = true;
        self.time_start = Instant::now();
        println!("Starting profiler");
    }
    pub fn stop(&mut self) {
        self.started = false;
        println!(
            "Stopping profiler for a total duration of {:.3}",
            self.time_start.elapsed().as_secs_f64()
        );
    }
    pub fn get_elapsed_time(&self) -> f64 {
        let elapsed = self.time_start.elapsed();
        let micros: u64 = elapsed.as_micros().try_into().unwrap();
        micros as f64
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

        let index = self.threads.len();
        self.threads.entry(id).or_insert(ThreadInfo {
            name,
            index,
            profiler: ThreadProfiler {
                id,
                tx: self.tx.clone(),
            },
        });
    }
    pub fn add_sample_for_thread(&mut self, category: &str, name: &str, start: f64, end: f64) {
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
        let end_time = self.get_elapsed_time();
        let mut thread_data = HashMap::new();
        let mut threads: Vec<(&ThreadId, &ThreadInfo)> = self.threads.iter().collect();
        threads.sort_by(|&a, &b| a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase()));
        for (&id, t) in threads.iter() {
            thread_data.insert(
                id,
                vec![Sample {
                    tid: id,
                    category: t.name.clone(),
                    name: t.name.clone(),
                    time_start: 0.,
                    time_end: end_time,
                }],
            );
        }

        while let Ok(sample) = self.rx.try_recv() {
            if let Some(vec) = thread_data.get_mut(&sample.tid) {
                vec.push(sample);
            } else {
                panic!("Invalid thread id {:?}", sample.tid);
            }
        }

        let mut data = Vec::new();
        for (id, vec) in thread_data.iter_mut() {
            vec.sort_by(|a, b| a.time_start.partial_cmp(&b.time_start).unwrap());

            for sample in vec.iter() {
                if let Some(thread) = self.threads.get(&id) {
                    let thread_id = thread.name.as_str();
                    data.push(serde_json::json!({
                        "pid": process::id(),
                        "id": thread.index,
                        "tid": thread_id,
                        "cat": thread_id,
                        "name": sample.name,
                        "ph": "B",
                        "ts": sample.time_start,
                    }));
                    data.push(serde_json::json!({
                        "pid": process::id(),
                        "id": thread.index,
                        "tid": thread_id,
                        "cat": thread_id,
                        "name": sample.name,
                        "ph": "E",
                        "ts": sample.time_end,
                    }));
                } else {
                    panic!("Invalid thread id {:?}", sample.tid);
                }
            }
        }

        let profile_file_name = "app.nrg_profile";

        let f = BufWriter::new(File::create(profile_file_name).unwrap());
        serde_json::to_writer(f, &data).unwrap();

        println!("Profile file {} written", profile_file_name);
    }
}

pub struct ScopedProfile {
    profiler: GlobalProfiler,
    category: String,
    name: String,
    time_start: f64,
}

impl ScopedProfile {
    pub fn new(profiler: GlobalProfiler, category: &str, name: &str) -> Self {
        let time_start = profiler.read().unwrap().get_elapsed_time();
        Self {
            profiler,
            category: category.to_string(),
            name: name.to_string(),
            time_start,
        }
    }
}

impl Drop for ScopedProfile {
    fn drop(&mut self) {
        let time_end = self.profiler.read().unwrap().get_elapsed_time();
        self.profiler.write().unwrap().add_sample_for_thread(
            self.category.as_str(),
            self.name.as_str(),
            self.time_start,
            time_end,
        );
    }
}
