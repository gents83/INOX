#![allow(improper_ctypes_definitions)]

use nrg_dynamic_library::Library;
use nrg_platform::{get_raw_thread_id, RawThreadId};
use std::{
    cell::RefCell,
    collections::HashMap,
    convert::TryInto,
    fs::File,
    io::BufWriter,
    process,
    sync::{
        atomic::{AtomicBool, AtomicU64},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self},
    time::{SystemTime, UNIX_EPOCH},
    u64,
};

pub type GlobalProfiler = Arc<Profiler>;
pub static mut NRG_PROFILER_LIB: Option<Library> = None;

pub const GET_PROFILER_FUNCTION_NAME: &str = "get_profiler";
pub type PfnGetProfiler = ::std::option::Option<unsafe extern "C" fn() -> GlobalProfiler>;

pub const CREATE_PROFILER_FUNCTION_NAME: &str = "create_profiler";
pub type PfnCreateProfiler = ::std::option::Option<unsafe extern "C" fn()>;

pub static mut GLOBAL_PROFILER: Option<GlobalProfiler> = None;
thread_local!(pub static THREAD_PROFILER: RefCell<Option<Arc<ThreadProfiler>>> = RefCell::new(None));

#[no_mangle]
pub extern "C" fn get_profiler() -> GlobalProfiler {
    unsafe { GLOBAL_PROFILER.as_ref().unwrap().clone() }
}

#[no_mangle]
pub extern "C" fn create_profiler() {
    unsafe {
        GLOBAL_PROFILER.replace(Arc::new(Profiler::new()));
        if let Some(profiler) = &GLOBAL_PROFILER {
            profiler.current_thread_profiler();
        }
    }
}

struct ThreadInfo {
    index: usize,
    name: String,
    profiler: Arc<ThreadProfiler>,
}

pub struct ThreadProfiler {
    id: RawThreadId,
    tx: Sender<Sample>,
}
unsafe impl Sync for ThreadProfiler {}
unsafe impl Send for ThreadProfiler {}

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
    tid: RawThreadId,
    category: String,
    name: String,
    time_start: f64,
    time_end: f64,
}

struct LockedData {
    threads: HashMap<RawThreadId, ThreadInfo>,
}
impl Default for LockedData {
    fn default() -> Self {
        Self {
            threads: HashMap::new(),
        }
    }
}

#[repr(C)]
pub struct Profiler {
    is_started: AtomicBool,
    time_start: AtomicU64,
    rx: Receiver<Sample>,
    tx: Sender<Sample>,
    locked_data: Mutex<LockedData>,
}
unsafe impl Sync for Profiler {}
unsafe impl Send for Profiler {}

impl Profiler {
    fn new() -> Profiler {
        let (tx, rx) = channel();

        Profiler {
            is_started: AtomicBool::new(false),
            time_start: AtomicU64::new(0),
            rx,
            tx,
            locked_data: Mutex::new(LockedData::default()),
        }
    }
    pub fn is_started(&self) -> bool {
        self.is_started.load(std::sync::atomic::Ordering::SeqCst)
    }
    pub fn get_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros()
            .try_into()
            .unwrap()
    }
    pub fn start(&self) {
        self.is_started
            .swap(true, std::sync::atomic::Ordering::SeqCst);
        self.time_start
            .swap(Profiler::get_time(), std::sync::atomic::Ordering::SeqCst);
        println!("Starting profiler");
    }
    pub fn stop(&self) {
        self.is_started
            .swap(false, std::sync::atomic::Ordering::SeqCst);
        let current_time = Profiler::get_time();
        let start_time = self.time_start.load(std::sync::atomic::Ordering::SeqCst);
        println!(
            "Stopping profiler for a total duration of {:.3}",
            (current_time - start_time) as f64 / 1000. / 1000.
        );
    }
    pub fn get_elapsed_time(&self) -> f64 {
        let current_time = Profiler::get_time();
        let start_time = self.time_start.load(std::sync::atomic::Ordering::SeqCst);
        (current_time - start_time) as _
    }
    pub fn current_thread_profiler(&self) -> Arc<ThreadProfiler> {
        let id = get_raw_thread_id();
        let name = String::from(thread::current().name().unwrap_or("main"));
        let mut locked_data = self.locked_data.lock().unwrap();
        let index = locked_data.threads.len();
        let thread_entry = locked_data.threads.entry(id).or_insert_with(|| ThreadInfo {
            index,
            name,
            profiler: Arc::new(ThreadProfiler {
                id,
                tx: self.tx.clone(),
            }),
        });
        thread_entry.profiler.clone()
    }

    pub fn write_profile_file(&self) {
        let end_time = self.get_elapsed_time();
        let mut thread_data = HashMap::new();
        let locked_data = self.locked_data.lock().unwrap();
        let mut threads: Vec<(&RawThreadId, &ThreadInfo)> = locked_data.threads.iter().collect();
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
                if let Some(thread) = locked_data.threads.get(&id) {
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
        let time_start = profiler.get_elapsed_time();
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
        let time_end = self.profiler.get_elapsed_time();

        THREAD_PROFILER.with(|profiler| {
            if profiler.borrow().is_none() {
                let thread_profiler = get_profiler().current_thread_profiler();
                *profiler.borrow_mut() = Some(thread_profiler);
            }
            profiler.borrow().as_ref().unwrap().push_sample(
                self.category.clone(),
                self.name.clone(),
                self.time_start,
                time_end,
            );
        });
    }
}
