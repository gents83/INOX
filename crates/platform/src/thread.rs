#[cfg(windows)]
///Raw thread id type, which is simple `u32`
pub type RawThreadId = u32;

#[cfg(all(
    unix,
    not(target_os = "linux"),
    not(target_os = "android"),
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "netbsd"),
    not(target_os = "freebsd")
))]
///Raw thread id type, which is opaque type, platform dependent
pub type RawThreadId = libc::pthread_t;

#[cfg(target_os = "linux")]
///Raw thread id as `pid_t` which is signed integer
///
///Can be accessed via `gettid` on Linux
pub type RawThreadId = libc::pid_t;

#[cfg(target_os = "android")]
///Raw thread id as `i32` which is signed integer
pub type RawThreadId = i32;

#[cfg(target_os = "freebsd")]
///Raw thread id signed integer
///
///Can be accessed via `pthread_threadid_np` on freebsd
pub type RawThreadId = libc::c_int;

#[cfg(target_os = "netbsd")]
///Raw thread id unsigned integer
///
///Can be accessed via `_lwp_self` on netbsd
pub type RawThreadId = libc::c_uint;

#[cfg(any(target_os = "macos", target_os = "ios"))]
///Raw thread id as unsigned 64 bit integer.
///
///Can be accessed via `pthread_threadid_np` on mac
pub type RawThreadId = u64;

#[cfg(all(not(unix), not(windows)))]
///Raw thread id type, which is dummy on this platform
pub type RawThreadId = u8;

#[cfg(windows)]
#[inline]
///Access id using `GetCurrentThreadId`
pub fn get_raw_thread_id() -> RawThreadId {
    extern "system" {
        pub fn GetCurrentThreadId() -> RawThreadId;
    }

    unsafe { GetCurrentThreadId() }
}

#[cfg(target_os = "freebsd")]
#[inline]
///Accesses id using `pthread_threadid_np`
pub fn get_raw_thread_id() -> RawThreadId {
    #[link(name = "pthread")]
    extern "C" {
        fn pthread_getthreadid_np() -> libc::c_int;
    }

    //According to documentation it cannot fail
    unsafe { pthread_getthreadid_np() }
}

#[cfg(target_os = "netbsd")]
#[inline]
///Accesses id using `_lwp_self`
pub fn get_raw_thread_id() -> RawThreadId {
    extern "C" {
        fn _lwp_self() -> libc::c_uint;
    }

    //According to documentation it cannot fail
    unsafe { _lwp_self() }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[inline]
///Accesses id using `pthread_threadid_np`
pub fn get_raw_thread_id() -> RawThreadId {
    #[link(name = "pthread")]
    extern "C" {
        fn pthread_threadid_np(thread: *mut core::ffi::c_void, thread_id: *mut u64) -> i32;
    }
    let mut tid: u64 = 0;
    let err = unsafe { pthread_threadid_np(0 as _, &mut tid) };
    assert_eq!(err, 0);
    tid
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[inline]
///Accesses id using `gettid`
pub fn get_raw_thread_id() -> RawThreadId {
    //unsafe { libc::syscall(libc::SYS_gettid) as libc::pid_t }
    0
}

#[cfg(all(
    unix,
    not(target_os = "linux"),
    not(target_os = "android"),
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "netbsd"),
    not(target_os = "freebsd")
))]
#[inline]
///Access id using `pthread_self`
pub fn get_raw_thread_id() -> RawThreadId {
    unsafe { libc::pthread_self() }
}

#[cfg(all(not(unix), not(windows)))]
#[inline]
///Returns zero id, as this platform has no concept of threads
pub fn get_raw_thread_id() -> RawThreadId {
    0
}
