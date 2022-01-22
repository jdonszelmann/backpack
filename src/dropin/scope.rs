use std::cell::RefCell;
use crate::dropin::config::Config;
thread_local! {
    pub static BACKPACK_ENV: RefCell<BackpackEnv> = RefCell::new(BackpackEnv::new());
}

pub struct BackpackEnv {
    config: Config,
}

impl BackpackEnv {
    pub const fn new() -> Self {
        Self {
            config: Config::new()
        }
    }
}

fn backpack<T: Send>(f: impl Send + FnOnce() -> T) -> T {
    backpack_with_config(f, Config::default())
}

fn backpack_with_config<T: Send>(f: impl Send + FnOnce() -> T, config: Config) -> T {
    let mut res = None;

    rayon::scope(|s| {
        s.spawn(|_| {
            BACKPACK_ENV.with(|i| {
                i.borrow_mut().config = config;
            });
            res = Some(f());
        })
    });

    res.expect("thread paniced")
}




