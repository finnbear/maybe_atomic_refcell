use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::{cmp, fmt};

/// Like an `AtomicRefCell` but no overhead of runtime checks in release mode.
pub struct MaybeAtomicRefCell<T: ?Sized> {
    #[cfg(debug_assertions)]
    inner: atomic_refcell::AtomicRefCell<T>,
    #[cfg(not(debug_assertions))]
    inner: std::cell::UnsafeCell<T>,
}

impl<T> MaybeAtomicRefCell<T> {
    /// Creates a new `MaybeAtomicRefCell` containing `value`.
    #[inline]
    pub const fn new(value: T) -> MaybeAtomicRefCell<T> {
        MaybeAtomicRefCell {
            #[cfg(debug_assertions)]
            inner: atomic_refcell::AtomicRefCell::new(value),
            #[cfg(not(debug_assertions))]
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
    /// release mode.
    #[inline]
    pub fn borrow(&self) -> MaybeAtomicRef<T> {
        #[cfg(debug_assertions)]
        return MaybeAtomicRef {
            inner: self.inner.borrow(),
        };
        #[cfg(not(debug_assertions))]
        MaybeAtomicRef {
            inner: unsafe { &*self.inner.get() },
        }
    }

    /// Mutably borrows the wrapped value. Performs runtime checks in debug mode, but not in
    /// release mode.
    #[inline]
    pub fn borrow_mut(&self) -> MaybeAtomicRefMut<T> {
        #[cfg(debug_assertions)]
        return MaybeAtomicRefMut {
            inner: self.inner.borrow_mut(),
        };
        #[cfg(not(debug_assertions))]
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
        #[cfg(debug_assertions)]
        return self.inner.as_ptr();
        #[cfg(not(debug_assertions))]
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

impl<T: Clone> Clone for MaybeAtomicRefCell<T> {
    #[inline]
    fn clone(&self) -> MaybeAtomicRefCell<T> {
        MaybeAtomicRefCell::new(self.borrow().clone())
    }
}

impl<T: Default> Default for MaybeAtomicRefCell<T> {
    #[inline]
    fn default() -> MaybeAtomicRefCell<T> {
        MaybeAtomicRefCell::new(Default::default())
    }
}

impl<T: ?Sized + PartialEq> PartialEq for MaybeAtomicRefCell<T> {
    #[inline]
    fn eq(&self, other: &MaybeAtomicRefCell<T>) -> bool {
        *self.borrow() == *other.borrow()
    }
}

impl<T: ?Sized + Eq> Eq for MaybeAtomicRefCell<T> {}

impl<T: ?Sized + PartialOrd> PartialOrd for MaybeAtomicRefCell<T> {
    #[inline]
    fn partial_cmp(&self, other: &MaybeAtomicRefCell<T>) -> Option<cmp::Ordering> {
        self.borrow().partial_cmp(&*other.borrow())
    }
}

impl<T: ?Sized + Ord> Ord for MaybeAtomicRefCell<T> {
    #[inline]
    fn cmp(&self, other: &MaybeAtomicRefCell<T>) -> cmp::Ordering {
        self.borrow().cmp(&*other.borrow())
    }
}

impl<T> From<T> for MaybeAtomicRefCell<T> {
    fn from(t: T) -> MaybeAtomicRefCell<T> {
        MaybeAtomicRefCell::new(t)
    }
}

pub struct MaybeAtomicRef<'b, T: ?Sized> {
    #[cfg(debug_assertions)]
    inner: atomic_refcell::AtomicRef<'b, T>,
    #[cfg(not(debug_assertions))]
    inner: &'b T,
}

impl<'b, T: ?Sized> Deref for MaybeAtomicRef<'b, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[cfg(debug_assertions)]
        return self.inner.deref();
        #[cfg(not(debug_assertions))]
        self.inner
    }
}

pub struct MaybeAtomicRefMut<'b, T: ?Sized> {
    #[cfg(debug_assertions)]
    inner: atomic_refcell::AtomicRefMut<'b, T>,
    #[cfg(not(debug_assertions))]
    inner: &'b mut T,
}

impl<'b, T: ?Sized> Deref for MaybeAtomicRefMut<'b, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[cfg(debug_assertions)]
        return self.inner.deref();
        #[cfg(not(debug_assertions))]
        self.inner
    }
}

impl<'b, T: ?Sized> DerefMut for MaybeAtomicRefMut<'b, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[cfg(debug_assertions)]
        return self.inner.deref_mut();
        #[cfg(not(debug_assertions))]
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

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn it_panics_mut_mut() {
        let cell = MaybeAtomicRefCell::new(5);
        let _borrow1 = cell.borrow_mut();
        let _borrow2 = cell.borrow_mut();
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn it_panics_mut_ref() {
        let cell = MaybeAtomicRefCell::new(5);
        let _borrow1 = cell.borrow_mut();
        let _borrow2 = cell.borrow();
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn it_panics_ref_mut() {
        let cell = MaybeAtomicRefCell::new(5);
        let _borrow1 = cell.borrow();
        let _borrow2 = cell.borrow_mut();
    }
}
