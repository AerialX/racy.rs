#![no_std]

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
// TODO require T: Send? a pointer is just an integer so it's not like it *has* to be, but consider how this might be used in practice?
pub struct SharedMut<T: ?Sized>(pub *mut T);

unsafe impl<T: ?Sized> Sync for SharedMut<T> { }
unsafe impl<T: ?Sized> Send for SharedMut<T> { }

impl<T: ?Sized> SharedMut<T> {
    pub const fn new(ptr: *mut T) -> Self {
        Self(ptr)
    }

    pub const fn get(&self) -> *mut T {
        self.0
    }
}

impl<T: ?Sized> Deref for SharedMut<T> {
    type Target = *mut T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ?Sized> DerefMut for SharedMut<T> {
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
    const DEFAULT: Self = RacyCell { inner: UnsafeCell::new(T::DEFAULT) };
}

impl<T: Sized> RacyCell<T> {
    #[inline]
    pub const fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
        }
    }
}

impl<T: ?Sized> RacyCell<T> {
    #[inline]
    pub const fn inner(&self) -> &UnsafeCell<T> {
        &self.inner
    }

    #[inline]
    pub const fn get(&self) -> *mut T {
        self.inner.get()
    }

    #[inline]
    pub unsafe fn get_ref(&self) -> &T {
        &*self.get()
    }

    #[inline]
    pub unsafe fn get_mut(&self) -> &mut T {
        &mut *self.get()
    }
}
