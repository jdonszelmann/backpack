use crate::dropin::config::Config;
use crate::{BackPack, InMemoryFile};
use std::cell::RefCell;
use lazy_static::lazy_static;
use elsa::sync::FrozenMap;
use std::thread::ThreadId;

thread_local! {
    static TL_CONFIG: RefCell<Config> = RefCell::new(Config::default());
}
lazy_static! {
    static ref TL_BACKPACKS: FrozenMap<ThreadId, Box<BackPack<'static, 'static>>> = FrozenMap::new();
}


pub(crate) fn with_config<T>(f: impl FnOnce(std::cell::Ref<Config>) -> T) -> T {
    TL_CONFIG.with(|config| {
        f(config.borrow())
    })
}

pub(crate) fn get_backpack() -> &'static BackPack<'static, 'static> {
    let res = TL_BACKPACKS.get(&std::thread::current().id());
    if let Some(i) = res {
        i
    } else {
        TL_BACKPACKS.insert(
            std::thread::current().id(),
            Box::new(BackPack::create(InMemoryFile::unnamed()).expect("failed to create backpack"))
        )
    }

    // TL_BACKPACK.with(|i| {
    //     if let Some(i) = i.get() {
    //         i
    //     } else {
    //         i.set(BackPack::create(InMemoryFile::unnamed()).expect("failed to create backpack"));
    //         get_backpack()
    //     }
    // })
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

            // TODO: remove thread local backpack here
        })
    });

    res.expect("thread paniced")

}




