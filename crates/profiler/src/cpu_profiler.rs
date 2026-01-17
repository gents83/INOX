#![allow(improper_ctypes_definitions)]

use inox_platform::{get_raw_thread_id, RawThreadId};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io::BufWriter,
    process,
    sync::{
        atomic::AtomicBool,
        mpsc::{channel, Receiver, Sender},
        Arc, LazyLock, Mutex, RwLock,
    },
    thread,
    time::Instant,
};

pub type GlobalCpuProfiler = Arc<CpuProfiler>;

pub static GLOBAL_CPU_PROFILER: LazyLock<GlobalCpuProfiler> =
    LazyLock::new(|| Arc::new(CpuProfiler::new()));

thread_local!(pub static THREAD_PROFILER: RefCell<Option<Arc<ThreadProfiler>>> = const { RefCell::new(None) });

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
    pub fn push_sample(&self, category: String, name: String, time_start: f64, time_end: f64) {
        let sample = Sample {
            tid: self.id,
            category,
            name,
            time_start,
            time_end,
        };
        self.tx.send(sample).ok();
    }
    pub fn push_sample_with_tid(
        &self,
        tid: RawThreadId,
        category: String,
        name: String,
        time_start: f64,
        time_end: f64,
    ) {
        let sample = Sample {
            tid,
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

#[derive(Default)]
struct LockedData {
    threads: HashMap<RawThreadId, ThreadInfo>,
}

#[repr(C)]
pub struct CpuProfiler {
    is_started: AtomicBool,
    time_start: RwLock<Instant>,
    rx: Receiver<Sample>,
    tx: Sender<Sample>,
    locked_data: Mutex<LockedData>,
}
unsafe impl Sync for CpuProfiler {}
unsafe impl Send for CpuProfiler {}

impl CpuProfiler {
    fn new() -> CpuProfiler {
        let (tx, rx) = channel();

        CpuProfiler {
            is_started: AtomicBool::new(false),
            time_start: RwLock::new(Instant::now()),
            rx,
            tx,
            locked_data: Mutex::new(LockedData::default()),
        }
    }
    pub fn is_started(&self) -> bool {
        self.is_started.load(std::sync::atomic::Ordering::SeqCst)
    }
    pub fn start(&self) {
        self.is_started
            .swap(true, std::sync::atomic::Ordering::SeqCst);
        *self.time_start.write().unwrap() = Instant::now();
        inox_log::debug_log!("Starting profiler");
    }
    pub fn stop(&self) {
        self.is_started
            .swap(false, std::sync::atomic::Ordering::SeqCst);
        let start_time = *self.time_start.read().unwrap();
        inox_log::debug_log!(
            "Stopping profiler for a total duration of {:.3}",
            start_time.elapsed().as_secs_f64()
        );
    }
    pub fn get_elapsed_time(&self) -> f64 {
        let start_time = *self.time_start.read().unwrap();
        start_time.elapsed().as_micros() as _
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

    pub fn log(&self, msg: &str) {
        let thread_id = get_raw_thread_id();
        let name = {
            let locked_data = self.locked_data.lock().unwrap();
            if let Some(thread_data) = locked_data.threads.get(&thread_id) {
                thread_data.name.clone()
            } else if let Some(name) = thread::current().name() {
                name.to_string()
            } else {
                format!("Thread {thread_id}")
            }
        };
        println!("[{name}]: {msg}");
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
            thread_data.entry(sample.tid).or_default().push(sample);
        }

        let mut data = Vec::new();
        for (id, vec) in thread_data.iter_mut() {
            vec.sort_by(|a, b| a.time_start.partial_cmp(&b.time_start).unwrap());

            for sample in vec.iter() {
                if let Some(thread) = locked_data.threads.get(id) {
                    let thread_id = thread.name.as_str();
                    data.push(serde_json::json!({
                        "pid": process::id(),
                        "id": thread.index,
                        "tid": thread_id,
                        "cat": sample.category,
                        "name": sample.name,
                        "ph": "X",
                        "ts": sample.time_start,
                        "dur": sample.time_end - sample.time_start,
                    }));
                } else {
                    let thread_name = if *id == u64::MAX { "GPU" } else { "Unknown" };
                    data.push(serde_json::json!({
                        "pid": process::id(),
                        "id": id,
                        "tid": thread_name,
                        "cat": sample.category,
                        "name": sample.name,
                        "ph": "X",
                        "ts": sample.time_start,
                        "dur": sample.time_end - sample.time_start,
                    }));
                }
            }
        }

        let profile_file_name = "app.inox_profile";

        let f = BufWriter::new(File::create(profile_file_name).unwrap());
        serde_json::to_writer(f, &data).unwrap();

        println!("Profile file {profile_file_name} written");
    }
}
