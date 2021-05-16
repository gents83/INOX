//! Thread id module

#[cfg(windows)]
///Raw thread id type, which is simple `u32`
pub type RawId = u32;

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
pub type RawId = libc::pthread_t;

#[cfg(any(target_os = "linux", target_os = "android"))]
///Raw thread id as `pid_t` which is signed integer
///
///Can be accessed via `gettid` on Linux and Android
pub type RawId = libc::pid_t;

#[cfg(target_os = "freebsd")]
///Raw thread id signed integer
///
///Can be accessed via `pthread_threadid_np` on freebsd
pub type RawId = libc::c_int;

#[cfg(target_os = "netbsd")]
///Raw thread id unsigned integer
///
///Can be accessed via `_lwp_self` on netbsd
pub type RawId = libc::c_uint;

#[cfg(any(target_os = "macos", target_os = "ios"))]
///Raw thread id as unsigned 64 bit integer.
///
///Can be accessed via `pthread_threadid_np` on mac
pub type RawId = u64;

#[cfg(all(not(unix), not(windows)))]
///Raw thread id type, which is dummy on this platform
pub type RawId = u8;

#[cfg(windows)]
#[inline]
///Access id using `GetCurrentThreadId`
pub fn get_raw_id() -> RawId {
    extern "system" {
        pub fn GetCurrentThreadId() -> RawId;
    }

    unsafe { GetCurrentThreadId() }
}

#[cfg(target_os = "freebsd")]
#[inline]
///Accesses id using `pthread_threadid_np`
pub fn get_raw_id() -> RawId {
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
pub fn get_raw_id() -> RawId {
    extern "C" {
        fn _lwp_self() -> libc::c_uint;
    }

    //According to documentation it cannot fail
    unsafe { _lwp_self() }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[inline]
///Accesses id using `pthread_threadid_np`
pub fn get_raw_id() -> RawId {
    #[link(name = "pthread")]
    extern "C" {
        fn pthread_threadid_np(thread: libc::pthread_t, thread_id: *mut u64) -> libc::c_int;
    }
    let mut tid: u64 = 0;
    let err = unsafe { pthread_threadid_np(0, &mut tid) };
    assert_eq!(err, 0);
    tid
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[inline]
///Accesses id using `gettid`
pub fn get_raw_id() -> RawId {
    unsafe { libc::syscall(libc::SYS_gettid) as libc::pid_t }
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
pub fn get_raw_id() -> RawId {
    unsafe { libc::pthread_self() }
}

#[cfg(all(not(unix), not(windows)))]
#[inline]
///Returns zero id, as this platform has no concept of threads
pub fn get_raw_id() -> RawId {
    0
}

#[derive(Copy, Clone, Debug)]
///Thread identifier.
pub struct ThreadId {
    id: RawId,
}

impl ThreadId {
    #[inline]
    ///Gets current thread id
    pub fn current() -> Self {
        Self { id: get_raw_id() }
    }

    #[inline]
    ///Access Raw identifier.
    pub const fn as_raw(&self) -> RawId {
        self.id
    }
}

impl core::cmp::PartialEq<ThreadId> for ThreadId {
    #[cfg(any(
        windows,
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "netbsd",
        target_os = "freebsd"
    ))]
    #[inline]
    fn eq(&self, other: &ThreadId) -> bool {
        self.id == other.id
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
    fn eq(&self, other: &ThreadId) -> bool {
        #[link(name = "pthread")]
        extern "C" {
            pub fn pthread_equal(left: RawId, right: RawId) -> libc::c_int;
        }

        unsafe { pthread_equal(self.id, other.id) != 0 }
    }
}

impl core::cmp::Eq for ThreadId {}

impl core::hash::Hash for ThreadId {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl core::fmt::Display for ThreadId {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.id, f)
    }
}

impl core::fmt::LowerHex for ThreadId {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::LowerHex::fmt(&self.id, f)
    }
}

impl core::fmt::UpperHex for ThreadId {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::UpperHex::fmt(&self.id, f)
    }
}

impl core::fmt::Octal for ThreadId {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Octal::fmt(&self.id, f)
    }
}
