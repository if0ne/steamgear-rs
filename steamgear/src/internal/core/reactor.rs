use std::{sync::LazyLock, task::Waker};

use dashmap::DashMap;
use steamgear_sys as sys;

pub(crate) static REACTOR: LazyLock<Reactor> = LazyLock::new(Reactor::new);

#[derive(Debug)]
pub struct Reactor {
    pub(crate) tasks: DashMap<sys::SteamAPICall_t, Waker>,
}

impl Reactor {
    pub(crate) fn new() -> Self {
        Self {
            tasks: Default::default(),
        }
    }

    pub(crate) fn add(&self, call: sys::SteamAPICall_t, waker: Waker) {
        self.tasks.entry(call).or_insert(waker);
    }

    pub(crate) fn wake(&self, call: sys::SteamAPICall_t) {
        let Some((_, task)) = self.tasks.remove(&call) else {
            return;
        };

        task.wake();
    }

    pub(crate) fn remove(&self, call: sys::SteamAPICall_t) {
        self.tasks.remove(&call);
    }
}
