use std::ops::{Deref, DerefMut};
use ref_thread_local::{ref_thread_local, RefThreadLocal, Ref};
use crate::dropin::config::Config;
use crate::{BackPack, InMemoryFile};
use std::cell::RefCell;

thread_local! {
    static TL_CONFIG: RefCell<Config> = RefCell::new(Config::default());
}

ref_thread_local! {
    static managed TL_BACKPACK: BackPack<'static, 'static> = BackPack::new(InMemoryFile::unnamed()).expect("failed to open backpack");
}

pub(crate) fn with_config<T>(f: impl FnOnce(std::cell::Ref<Config>) -> T) -> T {
    TL_CONFIG.with(|config| {
        f(config.borrow())
    })
}

pub(crate) fn get_backpack<'a>() -> Ref<'a, BackPack<'static, 'static>> {
    TL_BACKPACK.borrow()
}

/// All code passed in the closure will execute with a default
pub fn backpack<T: Send>(f: impl Send + FnOnce() -> T) -> T {
    backpack_with_config(Config::thread_local(), f)
}

pub fn backpack_with_config<T: Send>(config: impl AsRef<Config>, f: impl Send + FnOnce() -> T) -> T {
    let config = config.as_ref().clone();
    let mut res = None;

    rayon::scope(|s| {
        s.spawn(|_| {
            TL_CONFIG.with(|i| {
                *i.borrow_mut() = config;
            });
            res = Some(f());
        })
    });

    res.expect("thread paniced")

}




