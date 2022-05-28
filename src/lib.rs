use std::fmt;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

/// Like an `AtomicRefCell` but no overhead of runtime checks in release mode.
pub struct MaybeAtomicRefCell<T: ?Sized> {
    #[cfg(any(debug_assertions, feature = "safe"))]
    inner: atomic_refcell::AtomicRefCell<T>,
    #[cfg(not(any(debug_assertions, feature = "safe")))]
    inner: std::cell::UnsafeCell<T>,
}

impl<T> MaybeAtomicRefCell<T> {
    /// Creates a new `MaybeAtomicRefCell` containing `value`.
    #[inline]
    pub const fn new(value: T) -> MaybeAtomicRefCell<T> {
        MaybeAtomicRefCell {
            #[cfg(any(debug_assertions, feature = "safe"))]
            inner: atomic_refcell::AtomicRefCell::new(value),
            #[cfg(not(any(debug_assertions, feature = "safe")))]
            inner: std::cell::UnsafeCell::new(value),
        }
    }

    /// Consumes the `MaybeAtomicRefCell`, returning the wrapped value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}

impl<T: ?Sized> MaybeAtomicRefCell<T> {
    /// Immutably borrows the wrapped value. Performs runtime checks in debug mode, but not in
    /// release mode (hence `unsafe`).
    #[inline]
    pub unsafe fn borrow(&self) -> MaybeAtomicRef<T> {
        #[cfg(any(debug_assertions, feature = "safe"))]
        return MaybeAtomicRef {
            inner: self.inner.borrow(),
        };
        #[cfg(not(any(debug_assertions, feature = "safe")))]
        #[allow(unused_unsafe)]
        MaybeAtomicRef {
            inner: unsafe { &*self.inner.get() },
        }
    }

    /// Mutably borrows the wrapped value. Performs runtime checks in debug mode, but not in
    /// release mode (hence `unsafe`).
    #[inline]
    pub unsafe fn borrow_mut(&self) -> MaybeAtomicRefMut<T> {
        #[cfg(any(debug_assertions, feature = "safe"))]
        return MaybeAtomicRefMut {
            inner: self.inner.borrow_mut(),
        };
        #[cfg(not(any(debug_assertions, feature = "safe")))]
        #[allow(unused_unsafe)]
        MaybeAtomicRefMut {
            inner: unsafe { &mut *self.inner.get() },
        }
    }

    /// Returns a raw pointer to the underlying data in this cell.
    ///
    /// External synchronization is needed to avoid data races when dereferencing
    /// the pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut T {
        #[cfg(any(debug_assertions, feature = "safe"))]
        return self.inner.as_ptr();
        #[cfg(not(any(debug_assertions, feature = "safe")))]
        self.inner.get()
    }

    /// Returns a mutable reference to the wrapped value.
    ///
    /// No runtime checks take place (unless debug assertions are enabled)
    /// because this call borrows `MaybeAtomicRefCell` mutably at compile-time.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
}

unsafe impl<T: ?Sized + Send> Send for MaybeAtomicRefCell<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for MaybeAtomicRefCell<T> {}

impl<T: Default> Default for MaybeAtomicRefCell<T> {
    #[inline]
    fn default() -> MaybeAtomicRefCell<T> {
        MaybeAtomicRefCell::new(Default::default())
    }
}

impl<T> From<T> for MaybeAtomicRefCell<T> {
    fn from(t: T) -> MaybeAtomicRefCell<T> {
        MaybeAtomicRefCell::new(t)
    }
}

pub struct MaybeAtomicRef<'b, T: ?Sized> {
    #[cfg(any(debug_assertions, feature = "safe"))]
    inner: atomic_refcell::AtomicRef<'b, T>,
    #[cfg(not(any(debug_assertions, feature = "safe")))]
    inner: &'b T,
}

impl<'b, T: ?Sized> Deref for MaybeAtomicRef<'b, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[cfg(any(debug_assertions, feature = "safe"))]
        return self.inner.deref();
        #[cfg(not(any(debug_assertions, feature = "safe")))]
        self.inner
    }
}

pub struct MaybeAtomicRefMut<'b, T: ?Sized> {
    #[cfg(any(debug_assertions, feature = "safe"))]
    inner: atomic_refcell::AtomicRefMut<'b, T>,
    #[cfg(not(any(debug_assertions, feature = "safe")))]
    inner: &'b mut T,
}

impl<'b, T: ?Sized> Deref for MaybeAtomicRefMut<'b, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[cfg(any(debug_assertions, feature = "safe"))]
        return self.inner.deref();
        #[cfg(not(any(debug_assertions, feature = "safe")))]
        self.inner
    }
}

impl<'b, T: ?Sized> DerefMut for MaybeAtomicRefMut<'b, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[cfg(any(debug_assertions, feature = "safe"))]
        return self.inner.deref_mut();
        #[cfg(not(any(debug_assertions, feature = "safe")))]
        self.inner
    }
}

impl<'b, T: ?Sized + Debug + 'b> Debug for MaybeAtomicRef<'b, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'b, T: ?Sized + Debug + 'b> Debug for MaybeAtomicRefMut<'b, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T: ?Sized + Debug> Debug for MaybeAtomicRefCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MaybeAtomicRefCell {{ ... }}")
    }
}

#[cfg(test)]
mod tests {
    use crate::MaybeAtomicRefCell;

    #[test]
    fn it_works() {
        let cell = MaybeAtomicRefCell::new(5);

        unsafe {
            {
                assert_eq!(*cell.borrow(), 5);
                let _borrow1 = cell.borrow();
                let _borrow2 = cell.borrow();
            }

            *cell.borrow_mut() += 1;

            {
                let _borrow1 = cell.borrow();
                let _borrow2 = cell.borrow();
                assert_eq!(*cell.borrow(), 6);
            }

            // Is Send.
            std::thread::spawn(move || {
                let mine = cell;
                mine.borrow();

                let inner = mine.into_inner();
                assert_eq!(inner, 6);
            })
            .join()
            .unwrap();
        }
    }

    #[test]
    #[cfg_attr(any(debug_assertions, feature = "safe"), should_panic)]
    fn it_panics_mut_mut() {
        let cell = MaybeAtomicRefCell::new(5);
        unsafe {
            let _borrow1 = cell.borrow_mut();
            let _borrow2 = cell.borrow_mut();
        }
    }

    #[test]
    #[cfg_attr(any(debug_assertions, feature = "safe"), should_panic)]
    fn it_panics_mut_ref() {
        let cell = MaybeAtomicRefCell::new(5);
        unsafe {
            let _borrow1 = cell.borrow_mut();
            let _borrow2 = cell.borrow();
        }
    }

    #[test]
    #[cfg_attr(any(debug_assertions, feature = "safe"), should_panic)]
    fn it_panics_ref_mut() {
        let cell = MaybeAtomicRefCell::new(5);
        unsafe {
            let _borrow1 = cell.borrow();
            let _borrow2 = cell.borrow_mut();
        }
    }
}
