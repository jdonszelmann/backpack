use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};

pub enum GuardedBytes<'a> {
    NoGuard(&'a [u8]),
    Guard(MappedRwLockReadGuard<'a, Vec<u8>>),
}

impl Debug for GuardedBytes<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "&{:?}", self.deref())
    }
}

impl<T: AsRef<[u8]>> PartialEq<T> for GuardedBytes<'_> {
    fn eq(&self, other: &T) -> bool {
        other.as_ref().eq(self.deref())
    }
}

impl<'a> From<&'a RwLock<Vec<u8>>> for GuardedBytes<'a> {
    fn from(g: &'a RwLock<Vec<u8>>) -> Self {
        Self::Guard(RwLockReadGuard::map(g.read(), |v| v))
    }
}

impl<'a> From<&'a [u8]> for GuardedBytes<'a> {
    fn from(r: &'a [u8]) -> Self {
        Self::NoGuard(r)
    }
}

impl Deref for GuardedBytes<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            GuardedBytes::NoGuard(v) => v,
            GuardedBytes::Guard(v) => {
                v.as_slice()
            }
        }
    }
}