use std::{
    cell::{RefCell, RefMut},
    fs::File,
    io::BufWriter,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Once,
    },
    thread,
    time::SystemTime,
};

static mut GLOBAL_PROFILER: Option<Arc<RefCell<Profiler>>> = None;
static mut INIT: Once = Once::new();

thread_local!(static THREAD_PROFILER: RefCell<Option<ThreadProfiler>> = RefCell::new(None));

#[derive(Copy, Clone)]
struct ThreadId(usize);

struct ThreadInfo {
    name: String,
}

struct ThreadProfiler {
    id: ThreadId,
    tx: Sender<Sample>,
}

impl ThreadProfiler {
    fn push_sample(&self, name: String, time_start: SystemTime, time_end: SystemTime) {
        let sample = Sample {
            tid: self.id,
            name,
            time_start,
            time_end,
        };
        self.tx.send(sample).ok();
    }
}

struct Sample {
    tid: ThreadId,
    name: String,
    time_start: SystemTime,
    time_end: SystemTime,
}

pub struct Profiler {
    rx: Receiver<Sample>,
    tx: Sender<Sample>,
    threads: Vec<ThreadInfo>,
}

impl Profiler {
    fn get_and_init_once() -> &'static Option<Arc<RefCell<Profiler>>> {
        unsafe {
            INIT.call_once(|| {
                GLOBAL_PROFILER = Some(Arc::new(RefCell::new(Profiler::new())));
            });
            &GLOBAL_PROFILER
        }
    }
    fn new() -> Profiler {
        let (tx, rx) = channel();

        Profiler {
            rx,
            tx,
            threads: Vec::new(),
        }
    }
    pub fn get_mut<'a>() -> RefMut<'a, Profiler> {
        let profiler = Profiler::get_and_init_once();
        profiler.as_ref().unwrap().borrow_mut()
    }
    pub fn register_thread(&mut self) {
        let id = ThreadId(self.threads.len());
        let name = match thread::current().name() {
            Some(s) => s.to_string(),
            None => format!("<unknown-{}>", id.0),
        };

        self.threads.push(ThreadInfo { name });

        THREAD_PROFILER.with(|profiler| {
            assert!(profiler.borrow().is_none());

            let thread_profiler = ThreadProfiler {
                id,
                tx: self.tx.clone(),
            };

            *profiler.borrow_mut() = Some(thread_profiler);
        });
    }

    pub fn write_profile_file(&self, filename: &str) {
        let start_time = SystemTime::now();
        let mut data = Vec::new();

        while let Ok(sample) = self.rx.try_recv() {
            if sample.time_start > start_time {
                break;
            }

            let thread_id = self.threads[sample.tid.0].name.as_str();
            if let Ok(time_start) = sample.time_start.duration_since(SystemTime::UNIX_EPOCH) {
                data.push(serde_json::json!({
                    "pid": 0,
                    "tid": thread_id,
                    "name": sample.name,
                    "ph": "B",
                    "ts": time_start.as_millis() as u64
                }));
            };
            if let Ok(time_end) = sample.time_end.duration_since(SystemTime::UNIX_EPOCH) {
                data.push(serde_json::json!({
                    "pid": 0,
                    "tid": thread_id,
                    "ph": "E",
                    "ts": time_end.as_millis() as u64
                }));
            }
        }

        let f = BufWriter::new(File::create(filename).unwrap());
        serde_json::to_writer(f, &data).unwrap();
    }
}

pub struct ScopedProfile {
    name: String,
    time_start: SystemTime,
}

impl ScopedProfile {
    pub fn new(name: String) -> ScopedProfile {
        let time_start = SystemTime::now();
        ScopedProfile { name, time_start }
    }
}

impl Drop for ScopedProfile {
    fn drop(&mut self) {
        let time_end = SystemTime::now();

        THREAD_PROFILER.with(|profiler| match *profiler.borrow() {
            Some(ref profiler) => {
                profiler.push_sample(self.name.clone(), self.time_start, time_end);
            }
            None => {
                eprintln!(
                    "Trying to call ScopedProfile {} on unregistered thread!",
                    self.name
                );
            }
        });
    }
}
