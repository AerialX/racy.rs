#![cfg_attr(not(test), no_std)]
#![cfg_attr(feature = "unstable", feature(const_raw_ptr_deref))]

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
// TODO require T: Send? a pointer is just an integer so it's not like it *has* to be, but consider how this might be used in practice?
pub struct SharedMut<T: ?Sized>(pub *mut T);

unsafe impl<T: ?Sized> Sync for SharedMut<T> { }
unsafe impl<T: ?Sized> Send for SharedMut<T> { }

impl<T: ?Sized> SharedMut<T> {
    #[inline]
    pub const fn new(ptr: *mut T) -> Self {
        Self(ptr)
    }

    #[inline]
    pub const fn get(&self) -> *mut T {
        self.0
    }
}

impl<T: Sized> From<*mut T> for SharedMut<T> {
    #[inline]
    fn from(ptr: *mut T) -> Self {
        Self::new(ptr)
    }
}

impl<T: ?Sized> Deref for SharedMut<T> {
    type Target = *mut T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ?Sized> DerefMut for SharedMut<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct RacyCell<T: ?Sized> {
    inner: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Sync for RacyCell<T> { }
unsafe impl<T: ?Sized + Send> Send for RacyCell<T> { }

#[cfg(feature = "const-default")]
impl<T: const_default::ConstDefault> const_default::ConstDefault for RacyCell<T> {
    const DEFAULT: Self = RacyCell::new(T::DEFAULT);
}

impl<T: Sized> RacyCell<T> {
    #[inline]
    pub const fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
        }
    }
}

impl<T: Sized> From<T> for RacyCell<T> {
    #[inline]
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

impl<T: ?Sized> RacyCell<T> {
    #[inline]
    pub const fn racy_inner(&self) -> &UnsafeCell<T> {
        &self.inner
    }

    #[inline]
    pub const fn racy_ptr(&self) -> *mut T {
        self.inner.get()
    }

    #[inline]
    #[cfg(feature = "unstable")]
    pub const unsafe fn racy_ref(&self) -> &T {
        &*self.racy_ptr()
    }

    #[inline]
    #[cfg(not(feature = "unstable"))]
    pub unsafe fn racy_ref(&self) -> &T {
        &*self.racy_ptr()
    }

    #[inline]
    pub unsafe fn racy_mut(&self) -> &mut T {
        &mut *self.racy_ptr()
    }
}

impl<T: ?Sized + Sync> Deref for RacyCell<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            self.racy_ref()
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct RecklessCell<T: ?Sized> {
    inner: RacyCell<T>,
}

unsafe impl<T: ?Sized> Sync for RecklessCell<T> { }
unsafe impl<T: ?Sized> Send for RecklessCell<T> { }

#[cfg(feature = "const-default")]
impl<T: const_default::ConstDefault> const_default::ConstDefault for RecklessCell<T> {
    const DEFAULT: Self = RecklessCell::new(T::DEFAULT);
}

impl<T: Sized> RecklessCell<T> {
    #[inline]
    pub const fn new(inner: T) -> Self {
        // TODO make this an unsafe fn?
        Self {
            inner: RacyCell::new(inner),
        }
    }
}

impl<T: Sized> From<T> for RecklessCell<T> {
    #[inline]
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

impl<T: ?Sized> RecklessCell<T> {
    #[inline]
    pub const fn reckless_inner(&self) -> &RacyCell<T> {
        &self.inner
    }

    #[inline]
    pub const fn reckless_cell(&self) -> &UnsafeCell<T> {
        self.inner.racy_inner()
    }

    #[inline]
    pub const fn reckless_get(&self) -> *mut T {
        self.inner.racy_ptr()
    }

    #[inline]
    #[cfg(feature = "unstable")]
    pub const fn reckless_ref(&self) -> &T {
        unsafe {
            self.inner.racy_ref()
        }
    }

    #[inline]
    #[cfg(not(feature = "unstable"))]
    pub fn reckless_ref(&self) -> &T {
        unsafe {
            self.inner.racy_ref()
        }
    }

    #[inline]
    pub unsafe fn reckless_mut(&self) -> &mut T {
        self.inner.racy_mut()
    }

    #[inline]
    pub fn reckless_get_mut(&mut self) -> &mut T {
        unsafe {
            self.reckless_mut()
        }
    }
}

impl<T: ?Sized> Deref for RecklessCell<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.reckless_ref()
    }
}

impl<T: ?Sized> DerefMut for RecklessCell<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.reckless_get_mut()
    }
}
