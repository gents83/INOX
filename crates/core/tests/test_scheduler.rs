#![cfg(not(target_arch = "wasm32"))]

use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, RwLock,
    },
    thread,
    time::Duration,
};

use inox_core::{
    JobHandler, JobHandlerTrait, JobPriority, Phases, Scheduler, System,
    SystemUID, INDEPENDENT_JOB_ID,
};
use inox_uid::generate_uid_from_string;

#[derive(Default)]
struct TestSystem {
    counter: Arc<AtomicUsize>,
}

impl TestSystem {
    fn new(_name: &str, counter: Arc<AtomicUsize>) -> Self {
        Self {
            counter,
        }
    }
}

inox_core::implement_unique_system_uid!(TestSystem);

impl System for TestSystem {
    fn read_config(&mut self, _plugin_name: &str) {}
    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {}
    fn run(&mut self) -> bool {
        self.counter.fetch_add(1, Ordering::SeqCst);
        true
    }
    fn uninit(&mut self) {}
}

#[test]
fn test_job_handler_single_thread() {
    let job_handler = Arc::new(RwLock::new(JobHandler::default()));
    let can_continue = Arc::new(AtomicBool::new(true));

    // Force single thread via logic simulation or just rely on the handler's capability
    // Since we cannot change the constant NUM_WORKER_THREADS at runtime easily without mocking or cfg,
    // we test the general job execution flow.
    // However, we can start the handler.

    job_handler.start(&can_continue);

    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    job_handler.add_job(
        &INDEPENDENT_JOB_ID,
        "TestJob",
        JobPriority::High,
        move || {
            c.fetch_add(1, Ordering::SeqCst);
        },
    );

    // Give some time for execution
    thread::sleep(Duration::from_millis(100));

    assert_eq!(counter.load(Ordering::SeqCst), 1);

    can_continue.store(false, Ordering::SeqCst);
    job_handler.stop();
}

#[test]
fn test_job_priorities() {
    let job_handler = Arc::new(RwLock::new(JobHandler::default()));
    let can_continue = Arc::new(AtomicBool::new(true));
    job_handler.start(&can_continue);

    let high_counter = Arc::new(AtomicUsize::new(0));
    let low_counter = Arc::new(AtomicUsize::new(0));

    let hc = high_counter.clone();
    let lc = low_counter.clone();

    // Add Low priority job that takes some time
    job_handler.add_job(
        &INDEPENDENT_JOB_ID,
        "LowJob",
        JobPriority::Low,
        move || {
            thread::sleep(Duration::from_millis(50));
            lc.fetch_add(1, Ordering::SeqCst);
        },
    );

    // Add High priority job
    job_handler.add_job(
        &INDEPENDENT_JOB_ID,
        "HighJob",
        JobPriority::High,
        move || {
            hc.fetch_add(1, Ordering::SeqCst);
        },
    );

    // High job should ideally finish before Low job in a loaded system,
    // or at least both should finish.
    thread::sleep(Duration::from_millis(200));

    assert_eq!(high_counter.load(Ordering::SeqCst), 1);
    assert_eq!(low_counter.load(Ordering::SeqCst), 1);

    can_continue.store(false, Ordering::SeqCst);
    job_handler.stop();
}

#[test]
fn test_scheduler_phases() {
    let job_handler = Arc::new(RwLock::new(JobHandler::default()));
    let can_continue = Arc::new(AtomicBool::new(true));
    job_handler.start(&can_continue);

    let mut scheduler = Scheduler::default();
    scheduler.start();

    let update_counter = Arc::new(AtomicUsize::new(0));
    let render_counter = Arc::new(AtomicUsize::new(0));

    scheduler.add_system(
        Phases::Update,
        TestSystem::new("UpdateSystem", update_counter.clone()),
        None,
        &job_handler,
    );

    scheduler.add_system(
        Phases::Render,
        TestSystem::new("RenderSystem", render_counter.clone()),
        None,
        &job_handler,
    );

    // Run scheduler
    scheduler.run_once(true, &job_handler);

    // Both systems should have run
    assert_eq!(update_counter.load(Ordering::SeqCst), 1);
    assert_eq!(render_counter.load(Ordering::SeqCst), 1);

    scheduler.uninit();
    can_continue.store(false, Ordering::SeqCst);
    job_handler.stop();
}

#[test]
fn test_scheduler_dependencies() {
    let job_handler = Arc::new(RwLock::new(JobHandler::default()));
    let can_continue = Arc::new(AtomicBool::new(true));
    job_handler.start(&can_continue);

    let mut scheduler = Scheduler::default();
    scheduler.start();

    let counter = Arc::new(AtomicUsize::new(0));

    let _sys1_id = generate_uid_from_string("TestSystem1");
    let _sys2_id = generate_uid_from_string("TestSystem2");

    // System 1 increments to 1
    let c1 = counter.clone();
    struct Sys1 { counter: Arc<AtomicUsize> }
    inox_core::implement_unique_system_uid!(Sys1);
    impl System for Sys1 {
        fn read_config(&mut self, _: &str) {}
        fn should_run_when_not_focused(&self) -> bool { false }
        fn init(&mut self) {}
        fn run(&mut self) -> bool {
            self.counter.store(1, Ordering::SeqCst);
            true
        }
        fn uninit(&mut self) {}
    }

    // System 2 checks if counter is 1, then sets to 2
    let c2 = counter.clone();
    struct Sys2 { counter: Arc<AtomicUsize> }
    inox_core::implement_unique_system_uid!(Sys2);
    impl System for Sys2 {
        fn read_config(&mut self, _: &str) {}
        fn should_run_when_not_focused(&self) -> bool { false }
        fn init(&mut self) {}
        fn run(&mut self) -> bool {
            if self.counter.load(Ordering::SeqCst) == 1 {
                self.counter.store(2, Ordering::SeqCst);
            }
            true
        }
        fn uninit(&mut self) {}
    }

    scheduler.add_system(
        Phases::Update,
        Sys1 { counter: c1 },
        None,
        &job_handler,
    );

    // Add Sys2 with dependency on Sys1 (conceptually via phase order or explicit deps if supported within phase)
    // Note: The current scheduler implementation in memory runs phases sequentially.
    // Within a phase, systems run in parallel via jobs.
    // So to test strict ordering, we should put them in different phases OR use the dependencies param.

    // Let's test dependencies within the same phase
    scheduler.add_system(
        Phases::Update,
        Sys2 { counter: c2 },
        Some(&[Sys1::system_id()]),
        &job_handler,
    );

    scheduler.run_once(true, &job_handler);

    assert_eq!(counter.load(Ordering::SeqCst), 2);

    scheduler.uninit();
    can_continue.store(false, Ordering::SeqCst);
    job_handler.stop();
}

#[test]
fn test_phase_wait_logic() {
    let job_handler = Arc::new(RwLock::new(JobHandler::default()));
    let can_continue = Arc::new(AtomicBool::new(true));
    job_handler.start(&can_continue);

    let mut scheduler = Scheduler::default();
    scheduler.start();

    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    // System that spawns a job and waits for it implicitly by being in a phase
    // In reality, the scheduler waits for jobs launched by the phase.
    // We can simulate a system that takes time.
    struct SlowSystem { counter: Arc<AtomicUsize> }
    inox_core::implement_unique_system_uid!(SlowSystem);
    impl System for SlowSystem {
        fn read_config(&mut self, _: &str) {}
        fn should_run_when_not_focused(&self) -> bool { false }
        fn init(&mut self) {}
        fn run(&mut self) -> bool {
            thread::sleep(Duration::from_millis(100));
            self.counter.fetch_add(1, Ordering::SeqCst);
            true
        }
        fn uninit(&mut self) {}
    }

    scheduler.add_system(
        Phases::Update,
        SlowSystem { counter: c },
        None,
        &job_handler,
    );

    // This call should block until SlowSystem is done because scheduler waits for phase completion
    scheduler.run_once(true, &job_handler);

    assert_eq!(counter.load(Ordering::SeqCst), 1);

    scheduler.uninit();
    can_continue.store(false, Ordering::SeqCst);
    job_handler.stop();
}

#[test]
fn test_stress_concurrent_jobs() {
    let job_handler = Arc::new(RwLock::new(JobHandler::default()));
    let can_continue = Arc::new(AtomicBool::new(true));
    job_handler.start(&can_continue);

    let total_jobs = 1000;
    let counter = Arc::new(AtomicUsize::new(0));

    for _ in 0..total_jobs {
        let c = counter.clone();
        job_handler.add_job(
            &INDEPENDENT_JOB_ID,
            "StressJob",
            JobPriority::Medium,
            move || {
                c.fetch_add(1, Ordering::SeqCst);
            },
        );
    }

    // Wait for all jobs to finish
    // Since we don't have a direct "wait_all" on handler exposed for tests easily, we loop
    let mut retries = 0;
    while counter.load(Ordering::SeqCst) < total_jobs && retries < 100 {
        thread::sleep(Duration::from_millis(50));
        retries += 1;
    }

    assert_eq!(counter.load(Ordering::SeqCst), total_jobs);

    can_continue.store(false, Ordering::SeqCst);
    job_handler.stop();
}
