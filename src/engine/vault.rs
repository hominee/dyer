//! A Lock-Free and Thread-Safe reference-counting pointer data container.
//!
//! Similar to [Arc]<[Mutex]>,
//! it not only provides shared owership of a value,
//! protects the data from mutation, but also **NOT BLOCKING** the thread and can send between
//! thread safely.
//!
//! [Clone::clone] return an `Vault<U>` instance which points to the heap-allocated source data
//! with reference counter increaced by 1.
//! when borrowed as mutable, it is sealed as [Vaulted],
//! only one mutable reference are allowed.
//!
//! it only get dropped when all clones dropped
//!
//! [Arc]: std::sync::Arc
//! [Mutex]: std::sync::Mutex
//!
use std::ptr::NonNull;
use std::sync::atomic::{self, AtomicBool, AtomicUsize, Ordering};

#[test]
fn test_unit() {
    let mut u = Vault::new(0);
    {
        *(u.as_mut()) = 1;
        assert_eq!(u.as_ref(), &1);
        assert_eq!(u.sealed.get_mut(), &false);
        let mut up = u.as_mut();
        *up += 1;
        assert_eq!(*up, 2);
    }
    assert_eq!(*u, 2);
    assert_eq!(u.sealed.get_mut(), &false);
}

/// A Lock-Free and Thread-Safe reference-counting data container
#[derive(std::fmt::Debug)]
pub struct Vault<U> {
    sealed: AtomicBool,
    inner: NonNull<Inner<U>>,
}

unsafe impl<U> Send for Vault<U> {}

struct Inner<U> {
    cn: AtomicUsize,
    data: U,
}

impl<U> Clone for Vault<U> {
    fn clone(&self) -> Self {
        self.inner().cn.fetch_add(1, Ordering::Relaxed);
        Self::from_inner(self.inner)
    }
}

impl<U> std::ops::Deref for Vault<U> {
    type Target = U;

    fn deref(&self) -> &Self::Target {
        &self.inner().data
    }
}

impl<U> std::ops::DerefMut for Vault<U> {
    fn deref_mut(&mut self) -> &mut U {
        unsafe { &mut self.inner.as_mut().data }
    }
}

impl<U: Default> Default for Vault<U> {
    fn default() -> Self {
        Vault::new(U::default())
    }
}

impl<U> Drop for Vault<U> {
    fn drop(&mut self) {
        if self.inner().cn.fetch_sub(1, Ordering::Release) != 1 {
            //println!("droping vault: {:?}, Not Dropped", self.inner().cn);
            return;
        }
        //println!("droping vault: {:?}, Dropped", self.inner().cn);
        atomic::fence(Ordering::Acquire);
        //self.inner().cn.load(Ordering::Acquire);
        self.drop_slow()
    }
}

impl<U> Vault<U> {
    pub fn new(d: U) -> Self {
        let dm: Box<_> = Box::new(Inner {
            cn: AtomicUsize::new(1),
            data: d,
        });
        Self::from_inner(Box::leak(dm).into())
    }

    pub fn take(&mut self) -> U
    where
        U: Default,
    {
        self.replace(U::default())
    }

    pub fn replace(&mut self, src: U) -> U {
        std::mem::replace(&mut **self, src)
    }

    #[allow(dead_code)]
    pub fn swap(&mut self, dst: &mut U) {
        std::mem::swap(&mut **self, dst);
    }

    #[allow(dead_code)]
    pub fn map<R, F>(&mut self, mut f: F) -> R
    where
        F: FnMut(&mut U) -> R,
    {
        unsafe { f(&mut self.inner.as_mut().data) }
    }

    #[allow(dead_code)]
    fn from_ptr(ptr: *mut Inner<U>) -> Self {
        unsafe { Self::from_inner(NonNull::new_unchecked(ptr)) }
    }

    fn inner(&self) -> &Inner<U> {
        unsafe { self.inner.as_ref() }
    }

    pub fn as_mut(&mut self) -> Vaulted<U> {
        while self.sealed.load(Ordering::Acquire) {}
        self.sealed.store(true, Ordering::Release);
        Vaulted(self)
    }

    #[allow(dead_code)]
    pub fn as_ref(&self) -> &U {
        while self.sealed.load(Ordering::Relaxed) {}
        unsafe { &self.inner.as_ref().data }
    }

    #[allow(dead_code)]
    pub(crate) fn update<F, R>(&mut self, mut f: F) -> R
    where
        F: FnMut(&mut U) -> R,
    {
        while self.sealed.load(Ordering::Acquire) {}
        f(&mut *self)
    }

    #[allow(dead_code)]
    pub(crate) fn try_unseal(&mut self) {
        if self.sealed.load(Ordering::Acquire) {
            self.sealed.store(false, Ordering::Release);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn unseal(&mut self) {
        while self.sealed.load(Ordering::Acquire) {
            self.sealed.store(false, Ordering::Release);
        }
    }

    unsafe fn get_mut_unchecked(this: &mut Self) -> &mut U {
        &mut (*this.inner.as_ptr()).data
    }

    fn drop_slow(&mut self) {
        unsafe { std::ptr::drop_in_place(Self::get_mut_unchecked(self)) };
    }

    fn from_inner(ptr: NonNull<Inner<U>>) -> Self {
        Self {
            sealed: AtomicBool::new(false),
            inner: ptr,
        }
    }
}

/// Serve as a `scoped lock` of [Vault], when it is dropped(falls out of scope), [Vault] will be
/// Available
pub struct Vaulted<'a, U>(&'a mut Vault<U>);

impl<'a, U> Vaulted<'a, U> {
    #[allow(dead_code)]
    fn new(d: &'a mut Vault<U>) -> Self {
        Vaulted(d)
    }
}

impl<U> std::ops::Deref for Vaulted<'_, U> {
    type Target = U;

    fn deref(&self) -> &Self::Target {
        &self.0.inner().data
    }
}

impl<U> std::ops::DerefMut for Vaulted<'_, U> {
    fn deref_mut(&mut self) -> &mut U {
        unsafe { &mut self.0.inner.as_mut().data }
    }
}

impl<U> Drop for Vaulted<'_, U> {
    fn drop(&mut self) {
        while !self.0.sealed.load(Ordering::Acquire) {}
        self.0.sealed.store(false, Ordering::Release);
    }
}
