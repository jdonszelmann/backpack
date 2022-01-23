use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use parking_lot::MappedRwLockReadGuard;

pub enum MaybeRef<'a, T: ?Sized> {
    Regular(&'a T),
    Ref(MappedRwLockReadGuard<'a, T>),
}

impl<T: Debug + ?Sized> Debug for MaybeRef<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "&{:?}", self.deref())
    }
}



impl<'a, T: ?Sized> From<&'a T> for MaybeRef<'a, T> {
    fn from(r: &'a T) -> Self {
        Self::Regular(r)
    }
}

impl<'a, T: ?Sized> From<MappedRwLockReadGuard<'a, T>> for MaybeRef<'a, T> {
    fn from(r: MappedRwLockReadGuard<'a, T>) -> Self {
        Self::Ref(r)
    }
}

impl<T: ?Sized> Deref for MaybeRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeRef::Regular(r) => r,
            MaybeRef::Ref(r) => {
                r.deref()
            }
        }
    }
}