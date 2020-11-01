use super::platform_impl::platform::symbol as platform;
use std::marker;
use std::ops;
use std::fmt;

pub struct Symbol<'lib, T: 'lib> {
    inner: platform::Symbol<T>,
    pd: marker::PhantomData<&'lib T>
}

impl<'lib, T> Symbol<'lib, T> {
    pub unsafe fn into_raw(self) -> platform::Symbol<T> {
        self.inner
    }

    pub unsafe fn from_raw<L>(sym: platform::Symbol<T>, _: &'lib L) -> Symbol<'lib, T> {
        Symbol {
            inner: sym,
            pd: marker::PhantomData
        }
    }
}

impl<'lib, T> Clone for Symbol<'lib, T> {
    fn clone(&self) -> Symbol<'lib, T> {
        Symbol {
            inner: self.inner.clone(),
            pd: marker::PhantomData
        }
    }
}

// FIXME: implement FnOnce for callable stuff instead.
impl<'lib, T> ops::Deref for Symbol<'lib, T> {
    type Target = T;
    fn deref(&self) -> &T {
        ops::Deref::deref(&self.inner)
    }
}

impl<'lib, T> fmt::Debug for Symbol<'lib, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

unsafe impl<'lib, T: Send> Send for Symbol<'lib, T> {}
unsafe impl<'lib, T: Sync> Sync for Symbol<'lib, T> {}
